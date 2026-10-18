#!/usr/bin/env python3
"""Build our native watcher callback/lifecycle implementation against public SDK.

Not a step of the normal build: crates/jaic-cli/build.rs embeds the same backend in
jaic (docs/native/crash-runtime-build.md). This helper writes to --output-dir
(default target/native-helpers/macos-watch), never to the source tree.
"""
import argparse
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
if platform.system() != "Darwin":
    raise RuntimeError("native watcher backend currently supports macOS only")
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output-dir", type=Path)
args = parser.parse_args()
output = args.output_dir or Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "native-helpers/macos-watch"
output.mkdir(parents=True, exist_ok=True)
directory = ROOT / "stdlib/internal/watching/native"
subprocess.run(["clang", "-std=c11", "-O2", "-fblocks", "-fvisibility=hidden",
                "-arch", "arm64", "-arch", "x86_64",
                "-Wall", "-Wextra", "-Werror", "-dynamiclib", str(directory / "watch.c"),
                "-Wl,-install_name,@rpath/libportable_watch.dylib",
                "-framework", "CoreServices", "-pthread", "-o",
                str(output / "libportable_watch.dylib")], cwd=ROOT, check=True)
print("Built independent native FSEvents lifecycle/queue backend from source")
