"""Where the repository and its build products are."""

import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

__all__ = ["ROOT", "cargo_target_dir", "stdlib_env"]


def cargo_target_dir(*, resolve: bool = False) -> Path:
    """`$CARGO_TARGET_DIR` when set, else `<repo>/target`."""
    path = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    return path.resolve() if resolve else path


def stdlib_env(**extra: str) -> dict[str, str]:
    """The current environment with `JAIC_STDLIB` pointing at the checkout's `stdlib/` (plus `extra`)."""
    return dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), **extra)
