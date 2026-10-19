"""Subprocess runs that assert on the outcome."""

import subprocess
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

# tools/, for folder-qualified imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT

__all__ = ["run", "run_expect"]


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
    keyword arguments (`cwd`, `env`, `timeout`, ...) go to `subprocess.run`."""
    argv = [str(item) for item in command]
    if echo:
        print("+ " + " ".join(argv), file=sys.stderr, flush=True)
    if capture:
        kwargs.setdefault("text", True)
        kwargs.setdefault("capture_output", True)
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
    result = subprocess.run(
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
