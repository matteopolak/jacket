"""Where the repo's tools write build products, caches and scratch output.

Everything lives under `<repo>/target/` (already ignored, shared with cargo). Set
`JACKET_OUTPUT_DIR` to relocate it. Git worktrees reuse expensive downloads and native builds
through `shared_output_dir()`, which resolves to the main checkout's directory.
"""
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main_checkout() -> Path:
    """The main checkout of this repository (ROOT itself outside a linked worktree)."""
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    return Path(common.stdout.strip()).parent if common.returncode == 0 else ROOT


def _override() -> Path | None:
    value = os.environ.get('JACKET_OUTPUT_DIR')
    return Path(value).resolve() if value else None


def output_dir(*parts: str) -> Path:
    """This checkout's output directory (`target/`), optionally joined with `parts`."""
    return (_override() or ROOT / 'target').joinpath(*parts)


def shared_output_dir(*parts: str) -> Path:
    """The main checkout's output directory, shared by every worktree."""
    return (_override() or main_checkout() / 'target').joinpath(*parts)
