#!/usr/bin/env python3
"""Build the independently authored generator host operations from this checkout."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT / "stdlib/internal/developer-tools"
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    system = platform.system()
    suffix = {"Darwin": "dylib", "Linux": "so", "Windows": "dll"}[system]
    flags = ["-dynamiclib"] if system == "Darwin" else ["-shared"]
    libraries = []
    if system == "Linux":
        flags += ["-fPIC"]
        libraries += ["-ldl"]
    subprocess.run([str(prefix / "bin/clang"), "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                    *flags, str(directory / "generator-platform-native.c"),
                    "-o", str(directory / f"libgenerator_platform.{suffix}"), *libraries], check=True, cwd=ROOT)
    (directory / "generator-platform-nm.txt").write_text(str(prefix / "bin/llvm-nm"))


if __name__ == "__main__":
    main()
