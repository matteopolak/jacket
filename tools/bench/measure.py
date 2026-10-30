"""Run one process and measure it: wall time, user+system CPU time, peak resident set.

macOS and Linux read these from `wait4` (`ru_maxrss`); Windows from a Job Object (`winjob.py`). On every OS
"peak memory" is the peak resident memory (working set) of the largest single process in the command's
tree; CPU time is user plus system time of the whole tree. On Windows the job's peak commit charge,
which is larger, is kept in `Run.commit_mib`.
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
    """SIGKILL the process group led by `pid`: the process and any children (a linker, a helper) it left behind.
    On Windows, `taskkill /T` ends the process and its descendants."""
    if sys.platform == 'win32':
        subprocess.run(['taskkill', '/F', '/T', '/PID', str(pid)], capture_output=True)
        return
    try:
        os.killpg(pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass


def run_measured(command: list[str], cwd: Path | None = None, env: dict[str, str] | None = None,
                 timeout: float = 900, stdin_bytes: bytes | None = None) -> Run:
    """Run `command` to completion (or kill it after `timeout` seconds). Peak RSS and CPU time are the
    process's own, from wait4 (a Job Object on Windows): children such as a linker it spawns are included in CPU time but only
    the largest single process counts for RSS. The command runs in a process group of its own, which is
    killed on timeout and again once the command exits, so no child outlives the run."""
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err, tempfile.TemporaryFile() as feed:
        stdin = DEVNULL
        if stdin_bytes is not None:
            feed.write(stdin_bytes)
            feed.seek(0)
            stdin = feed
        if sys.platform == 'win32':
            return run_on_windows(command, cwd, env, stdin, out, err, timeout)
        start = time.perf_counter()
        process = Popen(command, cwd=cwd, env=env, stdin=stdin, stdout=out, stderr=err,
                         start_new_session=True)
        deadline = start + timeout
        timed_out = False
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
        kill_group(process.pid)
        process.returncode = os.waitstatus_to_exitcode(status)  # reaped here, not by Popen
        out.seek(0)
        err.seek(0)
        return Run(wall=wall, cpu=usage.ru_utime + usage.ru_stime, rss_mib=usage.ru_maxrss / RSS_UNIT,
                   returncode=process.returncode, stdout=out.read().decode(errors='replace'),
                   stderr=err.read().decode(errors='replace'), timed_out=timed_out)


def run_on_windows(command, cwd, env, stdin, out, err, timeout) -> Run:
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
    out.seek(0)
    err.seek(0)
    return Run(wall=measured.wall, cpu=measured.cpu, rss_mib=measured.rss_mib, returncode=measured.returncode,
               stdout=out.read().decode(errors='replace'), stderr=err.read().decode(errors='replace'),
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
