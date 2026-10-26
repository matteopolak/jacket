"""Select an own source build in this checkout or its launch-assigned Cargo target.

The explicit CARGO_TARGET_DIR is the worker's authorized build storage. This does
not search peer/external build trees or fall back to a supplied compiler binary.
"""

from toolkit.repo import cargo_target_dir
from repo_output import output_dir


def assigned_target():
    return cargo_target_dir().resolve()


def checked_compiler(candidate, parser):
    compiler = candidate.resolve(strict=True)
    if not (compiler.is_relative_to(output_dir().resolve()) or compiler.is_relative_to(assigned_target())):
        parser.error("compiler must be an own source build under this checkout's target or its assigned CARGO_TARGET_DIR")
    return compiler
