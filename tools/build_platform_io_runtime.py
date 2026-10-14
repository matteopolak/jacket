#!/usr/bin/env python3
"""Build the independent Windows fault backend against public native SDK headers."""
import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--cpu", choices=["x64", "arm64"], default="x64")
    parser.add_argument("--compiler", help="native C compiler; defaults to clang or the target MinGW compiler")
    args = parser.parse_args()
    if args.cpu != "x64":
        parser.error("the reviewed public Windows crash-backtrace contract supports x64 only")
    stdlib = args.stdlib.resolve(strict=True)
    if not stdlib.is_relative_to(ROOT):
        parser.error("build outputs must stay in this checkout; use a local staged stdlib")
    directory = stdlib / "internal/runtime/crash-native"
    directory.mkdir(parents=True, exist_ok=True)
    source = ROOT / "stdlib/internal/runtime/crash-native/runtime-windows.c"
    native = os.name == "nt"
    if native:
        compiler = args.compiler or "clang"
        triple = "aarch64" if args.cpu == "arm64" else "x86_64"
        target = [f"--target={triple}-pc-windows-msvc"]
        import_library = directory / "platform_io_windows_crash.lib"
        linker = [f"-Wl,/IMPLIB:{import_library}"]
    else:
        prefix = "aarch64" if args.cpu == "arm64" else "x86_64"
        compiler = args.compiler or shutil.which(prefix + "-w64-mingw32-clang") or prefix + "-w64-mingw32-gcc"
        target = []
        import_library = directory / "libplatform_io_windows_crash.dll.a"
        linker = [f"-Wl,--out-implib,{import_library}"]
    output = directory / "platform_io_windows_crash.dll"
    subprocess.run([compiler, *target, "-std=c11", "-shared", "-O2", "-Wall", "-Wextra", "-Werror",
                    str(source), "-o", str(output), *linker, "-ldbghelp"], cwd=ROOT, check=True)
    print(f"built Windows {args.cpu} exception diagnostics from source on {platform.system()}: {output}")


if __name__ == "__main__":
    main()
