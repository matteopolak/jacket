"""Where the repository and its build products are."""

import functools
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

__all__ = ["ROOT", "cargo_target_dir", "jaifmt_path", "stdlib_env"]


@functools.cache
def _metadata_target_dir() -> Path:
    """Cargo's own answer (`.cargo/config.toml` and `build.target-dir` are honoured); `<repo>/target` when cargo cannot say."""
    try:
        out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
                             cwd=ROOT, capture_output=True, text=True, check=True)
        return Path(json.loads(out.stdout)["target_directory"])
    except (OSError, subprocess.CalledProcessError, ValueError, KeyError):
        return ROOT / "target"


def cargo_target_dir(*, resolve: bool = False) -> Path:
    """The directory cargo builds into: `$CARGO_TARGET_DIR`, else cargo's configured target directory, else `<repo>/target`."""
    env = os.environ.get("CARGO_TARGET_DIR")
    path = Path(env) if env else _metadata_target_dir()
    return path.resolve() if resolve else path


def jaifmt_path() -> Path:
    """The one place the repository's own jaifmt is built and read: `<cargo target directory>/jaifmt`."""
    return cargo_target_dir() / ("jaifmt.exe" if sys.platform == "win32" else "jaifmt")


def stdlib_env(**extra: str) -> dict[str, str]:
    """The current environment with `JAIC_STDLIB` pointing at the checkout's `stdlib/` (plus `extra`)."""
    return dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), **extra)
