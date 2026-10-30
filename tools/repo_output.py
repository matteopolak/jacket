"""Where the repo's tools write build products, caches and scratch output, and which platform they ran on.

Everything lives under `<repo>/target/` (already ignored, shared with cargo). Set
`JACKET_OUTPUT_DIR` to relocate it. Git worktrees reuse expensive downloads and native builds
through `shared_output_dir()`, which resolves to the main checkout's directory.
"""
import os
from pathlib import Path
from toolkit.host import host_cpu, host_os
from toolkit.repo import ROOT



def main_checkout() -> Path:
    """The main checkout of this repository: ROOT itself outside a linked worktree, or when the checkout has
    no version-control folder (an offline VM). It is read from the marker file a linked worktree has in
    place of that folder; no program is run, so it works where git is not installed."""
    marker = ROOT / '.git'
    try:
        if marker.is_file():
            text = marker.read_text(encoding='utf-8').strip()
            if text.startswith('gitdir:'):
                gitdir = (ROOT / text[len('gitdir:'):].strip()).resolve()
                link = gitdir / 'commondir'
                common = (gitdir / link.read_text(encoding='utf-8').strip()).resolve() if link.is_file() \
                    else gitdir.parent.parent
                if common.is_dir():
                    return common.parent
    except (OSError, ValueError):
        pass
    return ROOT


def _override() -> Path | None:
    value = os.environ.get('JACKET_OUTPUT_DIR')
    return Path(value).resolve() if value else None


def output_dir(*parts: str) -> Path:
    """This checkout's output directory (`target/`), optionally joined with `parts`."""
    return (_override() or ROOT / 'target').joinpath(*parts)


def shared_output_dir(*parts: str) -> Path:
    """The main checkout's output directory, shared by every worktree."""
    return (_override() or main_checkout() / 'target').joinpath(*parts)



def host_platform() -> str:
    """`<os>-<cpu>` of this machine, e.g. `macos-arm64`, `linux-x64`, `windows-x64`: the names of
    the per-platform directories under `target/`."""
    system, cpu = host_os(), host_cpu()
    if system not in ("macos", "linux", "windows") or cpu not in ("arm64", "x64"):
        raise ValueError(f"unsupported host {system} {cpu}")
    return f"{system}-{cpu}"
