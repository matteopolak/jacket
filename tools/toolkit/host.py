"""Facts about the machine a script runs on.

    host_os()          "macos" | "linux" | "windows"
    host_cpu()         "arm64" | "x64"
    exe_name(name)     `name` with `.exe` appended on Windows
    brew_prefix()      Homebrew's prefix on macOS (`$HOMEBREW_PREFIX`, else `brew --prefix`), or None
"""

import functools
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

__all__ = ["brew_prefix", "exe_name", "host_cpu", "host_os"]

_SYSTEMS = {"darwin": "macos", "linux": "linux", "win32": "windows"}
_CPUS = {"arm64": "arm64", "aarch64": "arm64", "amd64": "x64", "x86_64": "x64", "x64": "x64"}


def host_os() -> str:
    """This machine's OS in the names the lint profiles and platform directories use."""
    return _SYSTEMS.get(sys.platform, sys.platform)


def host_cpu() -> str:
    """This machine's CPU in the names the lint profiles and platform directories use."""
    machine = platform.machine().lower()
    return _CPUS.get(machine, machine)


def exe_name(name: str, os_name: str | None = None) -> str:
    """`name` as an executable's file name on `os_name` (default: this machine)."""
    return name + ".exe" if (os_name or host_os()) == "windows" else name


@functools.cache
def brew_prefix() -> Path | None:
    """Homebrew's prefix on macOS (headers in `include/`, libraries in `lib/`); None elsewhere or without Homebrew."""
    if host_os() != "macos":
        return None
    if os.environ.get("HOMEBREW_PREFIX"):
        return Path(os.environ["HOMEBREW_PREFIX"])
    if shutil.which("brew"):
        try:
            out = subprocess.run(["brew", "--prefix"], capture_output=True, text=True, check=True, timeout=30)
            return Path(out.stdout.strip())
        except (OSError, subprocess.SubprocessError):
            pass
    return None
