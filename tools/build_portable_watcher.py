#!/usr/bin/env python3
"""Build our native watcher callback/lifecycle implementation against public SDK."""
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
if platform.system() != "Darwin":
    raise RuntimeError("native watcher backend currently supports macOS only")
directory = ROOT / "stdlib/internal/watching/native"
subprocess.run(["clang", "-std=c11", "-O2", "-fblocks", "-fvisibility=hidden",
                "-Wall", "-Wextra", "-Werror", "-dynamiclib", str(directory / "watch.c"),
                "-framework", "CoreServices", "-pthread", "-o",
                str(directory / "libportable_watch.dylib")], cwd=ROOT, check=True)
print("Built original native FSEvents lifecycle/queue backend from source")
