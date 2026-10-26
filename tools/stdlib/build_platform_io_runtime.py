#!/usr/bin/env python3
"""Build the independent Windows fault backend against public native SDK headers."""

import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
from toolkit.repo import ROOT
from repo_output import output_dir


def build_static(compiler, target, source, directory, native):
    """`platform_io_windows_crash.lib`: the same source as an archive, with DbgHelp as a default library."""
    objects = output_dir("windows-crash-runtime")
    objects.mkdir(parents=True, exist_ok=True)
    obj = objects / "platform_io_windows_crash.obj"
    # `-DJAI_STATIC_CRASH_RUNTIME` makes the object ask the linker for dbghelp; the DLL links it itself.
    define = ["-DJAI_STATIC_CRASH_RUNTIME"] if native else []
    subprocess.run(
        [
            compiler,
            *target,
            "-std=c11",
            "-c",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            *define,
            str(source),
            "-o",
            str(obj),
        ],
        cwd=ROOT,
        check=True,
    )
    if native:
        archive = directory / "platform_io_windows_crash.lib"
        archive.unlink(missing_ok=True)
        tool = shutil.which("llvm-lib") or "lib"
        subprocess.run([tool, f"/OUT:{archive}", str(obj)], cwd=ROOT, check=True)
    else:
        archive = directory / "libplatform_io_windows_crash.a"
        archive.unlink(missing_ok=True)
        subprocess.run(
            [shutil.which("llvm-ar") or "ar", "rcs", str(archive), str(obj)],
            cwd=ROOT,
            check=True,
        )
    print(f"built the static crash runtime: {archive}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--cpu", choices=["x64", "arm64"], default="x64")
    parser.add_argument(
        "--compiler",
        help="native C compiler; defaults to clang or the target MinGW compiler",
    )
    parser.add_argument(
        "--static-library",
        action="store_true",
        help="also build platform_io_windows_crash.lib as a static archive that programs link in, so "
        "they need no DLL at run time (release archives); the DLL stays for the interpreter",
    )
    args = parser.parse_args()
    stdlib = args.stdlib.resolve(strict=True)
    if not stdlib.is_relative_to(ROOT):
        parser.error(
            "build outputs must stay in this checkout; use a local staged stdlib"
        )
    directory = stdlib / "internal/runtime/crash-native"
    directory.mkdir(parents=True, exist_ok=True)
    source = ROOT / "stdlib/internal/runtime/crash-native" / "runtime-windows.c"
    native = os.name == "nt"
    if native:
        compiler = args.compiler or "clang"
        triple = "aarch64" if args.cpu == "arm64" else "x86_64"
        target = [f"--target={triple}-pc-windows-msvc"]
        import_library = directory / "platform_io_windows_crash.lib"
        if args.static_library:
            # The static archive takes the import library's name, so the import library goes elsewhere.
            scratch = output_dir("windows-crash-runtime")
            scratch.mkdir(parents=True, exist_ok=True)
            import_library = scratch / "platform_io_windows_crash.lib"
        linker = [f"-Wl,/IMPLIB:{import_library}"]
    else:
        prefix = "aarch64" if args.cpu == "arm64" else "x86_64"
        compiler = (
            args.compiler
            or shutil.which(prefix + "-w64-mingw32-clang")
            or prefix + "-w64-mingw32-gcc"
        )
        target = []
        import_library = directory / "libplatform_io_windows_crash.dll.a"
        linker = [f"-Wl,--out-implib,{import_library}"]
    output = directory / "platform_io_windows_crash.dll"
    subprocess.run(
        [
            compiler,
            *target,
            "-std=c11",
            "-shared",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            str(source),
            "-o",
            str(output),
            *linker,
            "-ldbghelp",
        ],
        cwd=ROOT,
        check=True,
    )
    if args.static_library:
        build_static(compiler, target, source, directory, native)
    print(
        f"built Windows {args.cpu} exception diagnostics from source on {platform.system()}: {output}"
    )


if __name__ == "__main__":
    main()
