#!/usr/bin/env python3
"""Build this worker's new private native POSIX crash backend from source."""
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT / "stdlib/internal/runtime/crash-native"
    system = platform.system()
    if system == "Darwin":
        shared = ["-dynamiclib"]
        output = directory / "libportable_crash.dylib"
        libraries = ["-pthread"]
    elif system == "Linux":
        shared = ["-shared", "-fPIC"]
        output = directory / "libportable_crash.so"
        libraries = ["-pthread", "-ldl"]
    else:
        raise SystemExit("portable native crash backend supports Darwin/Linux hosts only")
    subprocess.run(["clang", "-std=c11", "-O2", "-fno-omit-frame-pointer", "-fvisibility=hidden",
                    "-Wall", "-Wextra", "-Werror", *shared, str(directory / "runtime.c"),
                    "-o", str(output), *libraries], cwd=ROOT, check=True)
    print("Built private crash runtime from source:", output.relative_to(ROOT))


if __name__ == "__main__":
    main()
