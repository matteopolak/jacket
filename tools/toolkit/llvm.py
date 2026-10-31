"""Where the LLVM install is, and which version it is (docs/tools/llvm-setup.md).

    version_env(version)    LLVM_VERSION, LLVM_MAJOR and LLVM_SYS_PREFIX_VAR for a full version string
    major()                 the LLVM major version
    major_minor()           "<major>.<minor>", the release llvm-sys is built for
    sys_prefix_var()        the variable llvm-sys reads (LLVM_SYS_<major><minor>_PREFIX)
    configured_prefix()     the install the environment names (that variable, else LLVM_PREFIX), or None
    prefixes()              every install worth looking in, best first
    tool(name)              the path of an LLVM program (`clang`, `llvm-symbolizer`), or None
    llvm_config()           the path of `llvm-config`, or None

The version comes from the environment CI exports (`LLVM_MAJOR`, `LLVM_SYS_PREFIX_VAR`) when set, else from
`.github/pins.json`, its one home. Standard library only.
"""

import functools
import json
import os
import shutil
import subprocess
from pathlib import Path

from toolkit.host import exe_name

__all__ = [
    "configured_prefix", "llvm_config", "major", "major_minor", "prefixes", "sys_prefix_var", "tool", "version_env",
]

_PINS = Path(__file__).resolve().parents[2] / ".github" / "pins.json"


def version_env(version: str) -> dict[str, str]:
    """The forms of a full LLVM version the setup actions and scripts export. The variable name follows the llvm-sys
    crate feature (`llvm23-1` reads LLVM_SYS_231_PREFIX)."""
    major, minor = version.split(".")[:2]
    return {
        "LLVM_VERSION": version,
        "LLVM_MAJOR": major,
        "LLVM_SYS_PREFIX_VAR": f"LLVM_SYS_{major}{minor}_PREFIX",
    }


@functools.cache
def _pinned() -> dict[str, str]:
    return version_env(json.loads(_PINS.read_text())["llvm"])


def major() -> str:
    return os.environ.get("LLVM_MAJOR") or _pinned()["LLVM_MAJOR"]


def major_minor() -> str:
    return ".".join(_pinned()["LLVM_VERSION"].split(".")[:2])


def sys_prefix_var() -> str:
    return os.environ.get("LLVM_SYS_PREFIX_VAR") or _pinned()["LLVM_SYS_PREFIX_VAR"]


def configured_prefix() -> Path | None:
    """The install named by the environment: the llvm-sys variable, else `LLVM_PREFIX`."""
    for name in (sys_prefix_var(), "LLVM_PREFIX"):
        if os.environ.get(name):
            return Path(os.environ[name])
    return None


def llvm_config(prefix: Path | None = None) -> Path | None:
    """`llvm-config` of `prefix`, else the versioned one on PATH, else the plain one."""
    if prefix is not None:
        found = prefix / "bin" / exe_name("llvm-config")
        return found if found.is_file() else None
    found = shutil.which(f"llvm-config-{major()}") or shutil.which("llvm-config")
    return Path(found) if found else None


def _llvm_config_prefix() -> Path | None:
    config = llvm_config()
    if not config:
        return None
    try:
        return Path(subprocess.check_output([str(config), "--prefix"], text=True, timeout=30).strip())
    except (OSError, subprocess.SubprocessError):
        return None


def prefixes() -> list[Path]:
    """Installs worth looking in, best first: the configured one, `llvm-config --prefix`, then the platform's usual
    places (Homebrew's `llvm@<major>`, Debian's `/usr/lib/llvm-<major>`). Only those that exist."""
    candidates = [configured_prefix(), _llvm_config_prefix()]
    candidates += [Path(root) / f"llvm@{major()}" for root in ("/opt/homebrew/opt", "/usr/local/opt")]
    candidates.append(Path(f"/usr/lib/llvm-{major()}"))
    found: list[Path] = []
    for candidate in candidates:
        if candidate and candidate.is_dir() and candidate not in found:
            found.append(candidate)
    return found


def tool(name: str) -> str | None:
    """An LLVM program: from the installs above, else `<name>-<major>` or `<name>` on PATH."""
    for prefix in prefixes():
        candidate = prefix / "bin" / exe_name(name)
        if candidate.is_file():
            return str(candidate)
    return shutil.which(f"{name}-{major()}") or shutil.which(name)
