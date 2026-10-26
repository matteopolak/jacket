"""What every developer-tools check shares: the repo root, the compiler environment and a checked run.

Importing this module puts `tools/` on `sys.path` (the checks run directly, which only adds this
folder), so a check imports `toolkit` and its siblings after it.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from toolkit.hashing import sha256_bytes, sha256_file  # noqa: E402,F401
from toolkit.process import run_expect  # noqa: E402
from repo_output import output_dir  # noqa: E402,F401
from toolkit.repo import ROOT, cargo_binary, cargo_target_dir, jaifmt_path, stdlib_env  # noqa: E402,F401

ENV = stdlib_env()


def __getattr__(name):
    """`JAIC` and `JAIFMT` ask cargo where its target directory is, which can wait on a cargo lock, so
    they are resolved on first use rather than by every import of this module."""
    if name == "JAIC":
        return cargo_binary("jaic")
    if name == "JAIFMT":
        return jaifmt_path()
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def run_checked(command, *, label, env=ENV, timeout=90, **expectations):
    """`toolkit.process.run_expect` with the checks' defaults: run in the repo root with `ENV`, 90 seconds."""
    return run_expect(command, label=label, env=env, timeout=timeout, **expectations)


def labeled_runner(label, *, expected="", stdout=False, **defaults):
    """A `run(command, expected=..., **overrides)` for one check: `run_checked` with this label and these
    defaults, returning the process's stdout when `stdout` is set and the finished process otherwise."""

    def run(command, expected=expected, **overrides):
        result = run_checked(command, label=label, expected=expected, **{**defaults, **overrides})
        return result.stdout if stdout else result

    return run
