"""What every developer-tools check shares: the repo root, the compiler environment and a checked run.

Importing this module puts `tools/` on `sys.path` (the checks run directly, which only adds this
folder), so a check imports `toolkit` and its siblings after it.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from toolkit.hashing import sha256_file  # noqa: E402,F401
from toolkit.process import run_expect  # noqa: E402
from toolkit.repo import ROOT, stdlib_env  # noqa: E402,F401

ENV = stdlib_env()
JAIC = ROOT / "target/debug/jaic"


def run_checked(command, *, label, env=ENV, timeout=90, **expectations):
    """`toolkit.process.run_expect` with the checks' defaults: run in the repo root with `ENV`, 90 seconds."""
    return run_expect(command, label=label, env=env, timeout=timeout, **expectations)
