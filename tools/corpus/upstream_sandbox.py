"""Confines upstream corpus programs: jaic compiling them (compile-time code runs), and the programs it builds.

The sources are third-party and unreviewed, so every invocation runs inside `bench.sandbox.Sandbox`:
no network, writes only to the case's scratch and working directories, reads limited to the system,
the jaic build, the stdlib, the native libraries and the upstream sources. macOS uses sandbox-exec
and Linux bwrap. Windows has no equivalent: it runs unconfined, on the disposable CI runner only.
Outside CI a platform without a sandbox refuses upstream cases unless `--no-sandbox` is passed.
"""
import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import os
import sys
import tempfile
from pathlib import Path

from bench.sandbox import Sandbox, environment


class Unsandboxed(Exception):
    """Upstream cases may not run here without a sandbox."""


def in_ci(env=None) -> bool:
    env = os.environ if env is None else env
    return bool(env.get("CI") or env.get("GITHUB_ACTIONS"))


def policy(no_sandbox: bool, env=None, available=Sandbox.available) -> bool:
    """Whether upstream commands run inside a sandbox. Raises Unsandboxed when they could not be
    confined and that was not asked for explicitly (`--no-sandbox`) or excused (a CI runner)."""
    if no_sandbox:
        return False
    reason = available()
    if not reason:
        return True
    if in_ci(env):
        return False
    raise Unsandboxed(f"upstream cases need a sandbox ({reason}); pass --no-sandbox to run them unconfined")


def native_library_paths(env=None) -> list[Path]:
    env = os.environ if env is None else env
    return [Path(p) for p in env.get("JAIC_NATIVE_LIBS", "").split(os.pathsep) if p]


def jaic_reads(jaic: str, stdlib: Path, upstream: Path, env=None) -> list[Path]:
    """What jaic and its programs may read: the build tree around the binary (native libraries,
    runtime helpers), the stdlib, the native libraries and the upstream sources."""
    binary = Path(jaic).resolve()
    # The interpreter: setup commands are `python3 -c` patches, and its framework or toolcache
    # install lies outside the system paths.
    return [binary.parent.parent, stdlib, upstream, *native_library_paths(env), Path(sys.base_prefix)]


def make_sandbox(reads, writes, loopback=False) -> Sandbox:
    return Sandbox(reads=[Path(p) for p in reads], writes=[Path(p) for p in writes], loopback=loopback)


def confine(sandbox, scratch: Path, command: list[str], env: dict | None = None):
    """The command (and environment) to run it under: wrapped, with temporary files and jaic's
    cache redirected into `scratch`; unchanged when `sandbox` is None."""
    if sandbox is None:
        return command, env
    redirected = environment(scratch)
    inner = {**(env if env is not None else os.environ)}
    inner.update(TMPDIR=redirected["TMPDIR"], JAIC_CACHE_DIR=redirected["JAIC_CACHE_DIR"])
    return sandbox.wrap(command), inner


def scratch_dir(label: str) -> Path:
    return Path(tempfile.mkdtemp(prefix=f"upstream-sandbox-{label}-"))
