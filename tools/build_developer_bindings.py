#!/usr/bin/env python3
"""Build the independently authored binding tool beside its private stdlib sources.

Requires an official LLVM SDK containing clang-c headers and libclang. On Mac,
the runtime closure is staged beside the helper with relocatable load paths.
Generated native images are build artifacts, not vendored compatibility inputs.
Shared build integration is coordinator-owned.
"""
import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
import developer_native_runtime

ROOT = Path(__file__).resolve().parents[1]


def llvm_prefix(prefix=None):
    configured = prefix or os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX") or os.environ.get("LLVM_SYS_231_PREFIX")
    if not configured:
        config = shutil.which("llvm-config-23") or shutil.which("llvm-config")
        if not config:
            raise RuntimeError("set DEVELOPER_TOOLS_LLVM_PREFIX or LLVM_SYS_231_PREFIX to the official LLVM SDK")
        configured = subprocess.check_output([config, "--prefix"], text=True).strip()
    sdk = Path(configured).resolve(strict=True)
    if not (sdk / "include/clang-c/Index.h").is_file():
        raise RuntimeError("official LLVM SDK lacks clang-c/Index.h: " + str(sdk))
    return sdk


def shared_flags():
    if platform.system() == "Darwin":
        return ["-dynamiclib"]
    if platform.system() == "Linux":
        return ["-shared", "-fPIC"]
    raise RuntimeError("developer-tool native verification requires a macOS/Linux SDK host")


def shared_library(name, directory=None):
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    shared_flags()  # Diagnose unsupported hosts before choosing a filename.
    return (directory or ROOT / "target") / ("lib" + name + "." + suffix)


def build(prefix=None):
    if platform.system() not in {"Darwin", "Linux"}:
        raise RuntimeError("Binding tool build currently supports macOS/Linux hosts")
    prefix = llvm_prefix(prefix)
    directory = ROOT / "stdlib/internal/developer-tools/clang-bindings"
    output = directory / "developer-tools-clang"
    loader_flags = ["-Wl,-headerpad_max_install_names"] if platform.system() == "Darwin" else ["-Wl,-rpath," + str(prefix / "lib")]
    subprocess.run([str(prefix / "bin/clang"), "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                    "-I", str(prefix / "include"), str(directory / "main.c"), "-L", str(prefix / "lib"),
                    *loader_flags, "-lclang", "-o", str(output)],
                   cwd=ROOT, check=True)
    library_name = "libclang.dylib" if platform.system() == "Darwin" else "libclang.so"
    library = prefix / "lib" / library_name
    if not library.is_file():
        raise RuntimeError(f"Official SDK is missing {library_name}")
    if platform.system() == "Darwin":
        names = [library_name, *developer_native_runtime.runtime_names(output)]
        developer_native_runtime.stage_macos_sdk(prefix, directory, list(dict.fromkeys(names)))
        developer_native_runtime.relocate_macos_helper(output, directory)
    else:
        link = directory / library_name
        if link.is_symlink():
            link.unlink()
        elif link.exists():
            raise RuntimeError(f"Refusing to replace non-symlink native dependency: {link}")
        link.symlink_to(library)
    (directory / "compiler-path.txt").write_text(str(prefix / "bin/clang"))
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--llvm-prefix", type=Path)
    args = parser.parse_args()
    print("Built private binding tool from source:", build(args.llvm_prefix).relative_to(ROOT))


if __name__ == "__main__":
    main()
