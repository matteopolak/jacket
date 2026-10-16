#!/usr/bin/env python3
"""Build the independently authored generator host operations from this checkout."""

import argparse
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT / "stdlib/internal/developer-tools"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=directory)
    output = parser.parse_args().output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    prefix = Path(
        os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm")
    )
    system = platform.system()
    suffix = {"Darwin": "dylib", "Linux": "so", "Windows": "dll"}[system]
    flags = ["-dynamiclib"] if system == "Darwin" else ["-shared"]
    if system == "Darwin":
        flags += ["-Wl,-install_name,@rpath/libgenerator_platform.dylib"]
    libraries = []
    if system == "Linux":
        flags += ["-fPIC"]
        libraries += ["-ldl"]
    subprocess.run(
        [
            str(prefix / "bin/clang"),
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            *flags,
            str(directory / "generator-platform-native.c"),
            "-o",
            str(output / f"libgenerator_platform.{suffix}"),
            *libraries,
        ],
        check=True,
        cwd=ROOT,
    )
    (output / "generator-platform-nm.txt").write_text(str(prefix / "bin/llvm-nm"))


if __name__ == "__main__":
    main()
