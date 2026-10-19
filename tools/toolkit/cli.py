"""Command-line arguments the scripts under tools/ share."""

import argparse
import sys
from pathlib import Path

# tools/, for folder-qualified imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import cargo_target_dir

__all__ = ["add_jaic_argument", "default_jaic"]


def default_jaic(profile: str = "debug") -> Path:
    """`<cargo target dir>/<profile>/jaic` (with `.exe` on Windows)."""
    return cargo_target_dir() / profile / ("jaic.exe" if sys.platform == "win32" else "jaic")


def add_jaic_argument(
    parser: argparse.ArgumentParser,
    *,
    required: bool = False,
    profile: str = "debug",
    type=Path,
    help: str | None = None,
) -> None:
    """Add `--jaic`. Optional unless `required`, defaulting to `default_jaic(profile)`; `type` converts it."""
    parser.add_argument(
        "--jaic",
        type=type,
        required=required,
        default=None if required else type(default_jaic(profile)),
        help=help or ("the compiler to test" if required else f"the compiler to test (default: {profile} build in the target dir)"),
    )
