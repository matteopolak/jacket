"""Subprocess runs that assert on the outcome."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

from toolkit.repo import ROOT

__all__ = ["kill_tree", "run", "run_expect", "run_tree"]

# Popen arguments that put a child in its own process group (Windows) or session (POSIX), so that
# `kill_tree` reaches what the child started.
NEW_GROUP: dict = (
    {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt" else {"start_new_session": True}
)


def kill_tree(process: subprocess.Popen) -> None:
    """Kill `process` and its descendants. The process must have been started with `NEW_GROUP`."""
    if os.name == "nt":
        subprocess.run(["taskkill", "/T", "/F", "/PID", str(process.pid)], capture_output=True)
        return
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        # Nothing left in the group (macOS answers EPERM for a group of one unreaped leader).
        process.kill()


def run_tree(
    argv: Sequence[str],
    *,
    timeout: float,
    check: bool = False,
    capture_output: bool = False,
    input: str | None = None,
    **kwargs,
) -> subprocess.CompletedProcess:
    """`subprocess.run` that, on `TimeoutExpired`, kills the child's whole process tree (not only the
    child) before re-raising. Other keyword arguments go to `Popen`."""
    if capture_output:
        kwargs["stdout"] = kwargs["stderr"] = subprocess.PIPE
    if input is not None:
        kwargs["stdin"] = subprocess.PIPE
    with subprocess.Popen(argv, **NEW_GROUP, **kwargs) as process:
        try:
            stdout, stderr = process.communicate(input, timeout=timeout)
        except subprocess.TimeoutExpired:
            kill_tree(process)
            process.communicate()
            raise
    if check and process.returncode:
        raise subprocess.CalledProcessError(process.returncode, argv, stdout, stderr)
    return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)


def run(
    command: Sequence[object],
    *,
    check: bool = True,
    capture: bool = False,
    echo: bool = True,
    **kwargs,
) -> subprocess.CompletedProcess:
    """Run `command` (items are `str()`-ed). `capture` collects stdout and stderr as text, `echo` (default) prints
    `+ command` to stderr first, and `check` raises `CalledProcessError` on a non-zero status. Other
    keyword arguments (`cwd`, `env`, `timeout`, ...) go to `subprocess.run`. With a `timeout` the child runs in
    its own process group (see `run_tree`) so a timeout also stops its descendants."""
    argv = [str(item) for item in command]
    if echo:
        print("+ " + " ".join(argv), file=sys.stderr, flush=True)
    if capture:
        kwargs.setdefault("text", True)
        kwargs.setdefault("capture_output", True)
    if kwargs.get("timeout") is not None:
        return run_tree(argv, check=check, **kwargs)
    return subprocess.run(argv, check=check, **kwargs)


def run_expect(
    command: Sequence[object],
    *,
    label: str,
    expected: str | None = "",
    code: int = 0,
    stderr: str | None = "",
    env: Mapping[str, str] | None = None,
    cwd: str | Path = ROOT,
    timeout: float = 90,
) -> subprocess.CompletedProcess:
    """Run `command` (items are `str()`-ed) and raise `RuntimeError` unless it behaved as expected.

    `expected` is the exact stdout (None skips the check), `code` the exit status and `stderr` the exact
    stderr (None skips the check). `label` prefixes the failure message. Returns the completed process.
    """
    result = run_tree(
        [str(item) for item in command], cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout
    )
    if (
        result.returncode != code
        or (expected is not None and result.stdout != expected)
        or (stderr is not None and result.stderr != stderr)
    ):
        diagnostic = (result.stderr or result.stdout).splitlines()[:2]
        raise RuntimeError(
            f"{label}: {[str(item) for item in command]}; status={result.returncode}; diagnostic={diagnostic}"
        )
    return result
