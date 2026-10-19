"""Where the repo's tools write build products, caches and scratch output, and which platform they ran on.

Everything lives under `<repo>/target/` (already ignored, shared with cargo). Set
`JACKET_OUTPUT_DIR` to relocate it. Git worktrees reuse expensive downloads and native builds
through `shared_output_dir()`, which resolves to the main checkout's directory.
"""
import os
import platform
import subprocess
from pathlib import Path
from toolkit.repo import ROOT



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


_SYSTEMS = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}
_CPUS = {"arm64": "arm64", "aarch64": "arm64", "amd64": "x64", "x86_64": "x64"}


def host_platform() -> str:
    """`<os>-<cpu>` of this machine, e.g. `macos-arm64`, `linux-x64`, `windows-x64`: the names of
    the per-platform directories under `target/`."""
    system = _SYSTEMS.get(platform.system())
    cpu = _CPUS.get(platform.machine().lower())
    if not system or not cpu:
        raise ValueError(f"unsupported host {platform.system()} {platform.machine()}")
    return f"{system}-{cpu}"
