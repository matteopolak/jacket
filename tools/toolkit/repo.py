"""Where the repository and its build products are.

    ROOT                                 the checkout
    cargo_target_dir(resolve=False)      `$CARGO_TARGET_DIR`, else cargo's configured target directory, else `<repo>/target`
    cargo_binary(name, profile, triple)  `<target dir>[/<triple>]/<profile>/<name>` (`.exe` on Windows), profile "debug" or "release"
    jaifmt_path()                        the repository formatter
    stdlib_env(**extra)                  the environment with `JAIC_STDLIB` set

Scripts never read `CARGO_TARGET_DIR` or build `ROOT / "target"` themselves (tools/ci/check_source_style.py enforces it);
a test that wants another directory sets `CARGO_TARGET_DIR`. Host facts (`host_os`, `exe_name`, ...) are in `toolkit.host`.
"""

import functools
import json
import os
import subprocess
from pathlib import Path


from toolkit.host import exe_name

ROOT = Path(__file__).resolve().parents[2]

__all__ = ["ROOT", "cargo_binary", "cargo_target_dir", "jaifmt_path", "stdlib_env"]


@functools.cache
def _metadata_target_dir() -> Path:
    """Cargo's own answer (`.cargo/config.toml` and `build.target-dir` are honoured); `<repo>/target` when cargo cannot say."""
    try:
        out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
                             cwd=ROOT, capture_output=True, text=True, check=True, timeout=30)
        return Path(json.loads(out.stdout)["target_directory"])
    except (OSError, subprocess.SubprocessError, ValueError, KeyError):
        return ROOT / "target"


def cargo_target_dir(*, resolve: bool = False) -> Path:
    """The directory cargo builds into: `$CARGO_TARGET_DIR`, else cargo's configured target directory, else `<repo>/target`."""
    env = os.environ.get("CARGO_TARGET_DIR")
    path = Path(env) if env else _metadata_target_dir()
    return path.resolve() if resolve else path


def cargo_binary(name: str, profile: str = "debug", triple: str | None = None) -> Path:
    """A cargo-built executable: `<target dir>[/<triple>]/<profile>/<name>`, with `.exe` when it runs on Windows."""
    base = cargo_target_dir() / triple if triple else cargo_target_dir()
    return base / profile / exe_name(name, ("windows" if "windows" in triple else "linux") if triple else None)


def jaifmt_path() -> Path:
    """The one place the repository's own jaifmt is built and read: `<cargo target directory>/jaifmt`."""
    return cargo_target_dir() / exe_name("jaifmt")


def stdlib_env(**extra: str) -> dict[str, str]:
    """The current environment with `JAIC_STDLIB` pointing at the checkout's `stdlib/` (plus `extra`)."""
    return dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), **extra)
