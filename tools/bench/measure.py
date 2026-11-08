"""Run one process and measure it: wall time, user+system CPU time, peak resident set.

macOS and Linux read wait4 accounting for the command and children it waited for. Windows uses a
Job Object's descendant accounting. RSS is the peak of the largest measured process, not a sum;
Windows peak commit charge is kept in `Run.commit_mib`. POSIX cleanup is limited to the private
process group: descendants that create another group/session escape it.
"""
from __future__ import annotations

import os
import signal
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
import subprocess
from subprocess import DEVNULL, Popen

POLL_SECONDS = 0.005
# CreateProcess's error for an image without a relocation table (a fixed base address) in an AppContainer,
# where Windows always forces ASLR.
ERROR_ILLEGAL_DLL_RELOCATION = 623


class SandboxError(RuntimeError):
    """A command cannot run in the sandbox at all, so the whole run stops."""
# ru_maxrss is in bytes on macOS and KiB elsewhere.
RSS_UNIT = 2**20 if sys.platform == 'darwin' else 2**10


@dataclass
class Run:
    wall: float
    cpu: float
    rss_mib: float
    returncode: int
    stdout: str
    stderr: str
    timed_out: bool = False
    commit_mib: float | None = None  # Windows only: peak commit charge of the whole tree

    @property
    def ok(self) -> bool:
        return self.returncode == 0 and not self.timed_out

    def failure(self) -> str:
        """One line saying why the run failed (the last lines of stderr, or stdout)."""
        if self.timed_out:
            return f'timed out after {self.wall:.0f} s'
        # The first error says why; jaic's `-timings` lines come last and say nothing.
        lines = [line.strip() for line in (self.stderr or self.stdout).splitlines()
                 if line.strip() and not line.startswith('jaic-timing:')]
        errors = [line for line in lines if 'error' in line.lower()]
        shown = errors[:2] or lines[-2:]
        return f'exit {self.returncode}: ' + ' | '.join(line[:160] for line in shown)


def kill_group(pid: int) -> None:
    """SIGKILL members of the POSIX process group led by `pid`, including children still in it.
    This does not reach descendants that create another group/session. On Windows this fallback
    uses `taskkill /T`; `run_measured` instead owns a Job Object."""
    if sys.platform == 'win32':
        subprocess.run(['taskkill', '/F', '/T', '/PID', str(pid)], capture_output=True)
        return
    try:
        os.killpg(pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass


def run_measured(command: list[str], cwd: Path | None = None, env: dict[str, str] | None = None,
                 timeout: float = 900, stdin_bytes: bytes | None = None, output=None,
                 limits: dict[int, int] | None = None) -> Run:
    """Run `command` to completion or its timeout (RSS is the largest measured process).
    POSIX wait4 accounting includes waited-for children; Windows accounts for the Job. A private POSIX group is killed on timeout,
    interruption and exit; a descendant starting a new group/session is outside this boundary.
    Windows uses a Job Object for descendant ownership. With `output=log_file`, stdout and stderr
    stream to that binary file instead of being retained in the result strings. `limits` maps `resource`
    limits to values (`{resource.RLIMIT_CPU: 180}`), set in the child before it runs; POSIX only."""
    with tempfile.TemporaryFile() as captured_out, tempfile.TemporaryFile() as captured_err, tempfile.TemporaryFile() as feed:
        out = captured_out if output is None else output
        err = captured_err if output is None else output
        stdin = DEVNULL
        if stdin_bytes is not None:
            feed.write(stdin_bytes)
            feed.seek(0)
            stdin = feed
        if sys.platform == 'win32':
            return run_on_windows(command, cwd, env, stdin, out, err, timeout, capture=output is None)
        start = time.perf_counter()
        process = Popen(command, cwd=cwd, env=env, stdin=stdin, stdout=out, stderr=err,
                         start_new_session=True, preexec_fn=apply_limits(limits) if limits else None)
        deadline = start + timeout
        timed_out = False
        try:
            while True:
                pid, status, usage = os.wait4(process.pid, os.WNOHANG)
                if pid:
                    break
                if time.perf_counter() > deadline:
                    kill_group(process.pid)
                    _, status, usage = os.wait4(process.pid, 0)
                    timed_out = True
                    break
                time.sleep(POLL_SECONDS)
            wall = time.perf_counter() - start
            process.returncode = os.waitstatus_to_exitcode(status)  # reaped here, not by Popen
        finally:
            kill_group(process.pid)
            if process.returncode is None:
                process.wait()
        stdout = stderr = ''
        if output is None:
            out.seek(0)
            err.seek(0)
            stdout, stderr = out.read().decode(errors='replace'), err.read().decode(errors='replace')
        return Run(wall=wall, cpu=usage.ru_utime + usage.ru_stime, rss_mib=usage.ru_maxrss / RSS_UNIT,
                   returncode=process.returncode, stdout=stdout, stderr=stderr, timed_out=timed_out)


def apply_limits(limits: dict[int, int]):
    """A `preexec_fn` that sets each `resource` limit (soft and hard) in the child."""
    def apply() -> None:
        import resource
        for kind, value in limits.items():
            resource.setrlimit(kind, (value, value))
    return apply


def run_on_windows(command, cwd, env, stdin, out, err, timeout, *, capture=True) -> Run:
    """The Windows half of `run_measured`. A command wrapped by `Sandbox.wrap` runs in its AppContainer."""
    from bench import winjob

    sandbox = getattr(command, 'sandbox', None)
    plain = list(command)
    sid = sandbox.container_sid() if sandbox else None
    null = None
    if stdin is DEVNULL:
        stdin = null = winjob.open_null()
    try:
        measured = winjob.run(plain, cwd, env, stdin, out, err, timeout, sid)
    except OSError as error:
        if sid and error.winerror == ERROR_ILLEGAL_DLL_RELOCATION:
            raise SandboxError(
                f'{plain[0]} cannot run in the AppContainer sandbox: it has no relocation table (a fixed base '
                'address), and Windows forces ASLR on every AppContainer process. Run in Windows Sandbox '
                '(`main.py wsb`) or, without isolation, with --no-sandbox.') from error
        raise
    finally:
        if null:
            null.close()
    stdout = stderr = ''
    if capture:
        out.seek(0)
        err.seek(0)
        stdout, stderr = out.read().decode(errors='replace'), err.read().decode(errors='replace')
    return Run(wall=measured.wall, cpu=measured.cpu, rss_mib=measured.rss_mib, returncode=measured.returncode,
               stdout=stdout, stderr=stderr,
               timed_out=measured.timed_out, commit_mib=measured.commit_mib)


def warm_file_cache(*paths: Path) -> int:
    """Read every file under `paths` once so the OS file cache holds it; returns the bytes read."""
    total = 0
    for path in paths:
        files = [path] if path.is_file() else sorted(p for p in path.rglob('*') if p.is_file())
        for file in files:
            try:
                total += len(file.read_bytes())
            except OSError:
                continue
    return total
