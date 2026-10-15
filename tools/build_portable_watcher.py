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
                "-arch", "arm64", "-arch", "x86_64",
                "-Wall", "-Wextra", "-Werror", "-dynamiclib", str(directory / "watch.c"),
                "-Wl,-install_name,@rpath/libportable_watch.dylib",
                "-framework", "CoreServices", "-pthread", "-o",
                str(directory / "libportable_watch.dylib")], cwd=ROOT, check=True)
print("Built independent native FSEvents lifecycle/queue backend from source")
