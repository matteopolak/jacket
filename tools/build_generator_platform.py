#!/usr/bin/env python3
"""Build the independently authored generator host operations from this checkout."""

import argparse
from pathlib import Path
import platform
import subprocess
import build_developer_bindings

ROOT = Path(__file__).resolve().parents[1]


def build(prefix=None, output_dir=None):
    directory = ROOT / "stdlib/internal/developer-tools"
    output = Path(output_dir or directory).resolve()
    output.mkdir(parents=True, exist_ok=True)
    prefix = build_developer_bindings.llvm_prefix(prefix)
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
    return output / f"libgenerator_platform.{suffix}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--llvm-prefix", type=Path)
    args = parser.parse_args()
    build(args.llvm_prefix, args.output_dir)


if __name__ == "__main__":
    main()
