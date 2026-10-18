"""Subprocess runs that assert on the outcome."""

import subprocess
from collections.abc import Mapping, Sequence
from pathlib import Path

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT


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
