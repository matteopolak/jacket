"""Select an own source build in this checkout or its launch-assigned Cargo target.

The explicit CARGO_TARGET_DIR is the worker's authorized build storage. This does
not search peer/external build trees or fall back to a supplied compiler binary.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import os
from pathlib import Path
from toolkit.repo import ROOT



def assigned_target():
    return Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target/compiler")).resolve()


def default_compiler():
    return assigned_target() / "debug/jaic"


def checked_compiler(candidate, parser):
    compiler = candidate.resolve(strict=True)
    if not (compiler.is_relative_to((ROOT / "target").resolve()) or compiler.is_relative_to(assigned_target())):
        parser.error("compiler must be an own source build under this checkout's target or its assigned CARGO_TARGET_DIR")
    return compiler
