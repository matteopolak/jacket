#!/usr/bin/env python3
"""Source-build the owned Check demangler adapter against configured official LLVM."""
import os
from pathlib import Path
import platform
import subprocess
import developer_native_runtime

ROOT = Path(__file__).resolve().parents[1]


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    directory = ROOT / "stdlib/internal/developer-tools/check-native"
    library_dir = Path(subprocess.check_output([prefix / "bin/llvm-config", "--libdir"], text=True).strip())
    library_names = subprocess.check_output([prefix / "bin/llvm-config", "--libnames", "demangle"], text=True).split()
    libraries = [str(library_dir / name) for name in library_names]
    if platform.system() == "Darwin":
        flags, suffix = ["-dynamiclib", "-Wl,-headerpad_max_install_names"], "dylib"
    elif platform.system() == "Linux":
        flags, suffix = ["-shared", "-fPIC", f"-Wl,-rpath,{prefix / 'lib'}"], "so"
    elif platform.system() == "Windows":
        flags, suffix = ["-shared"], "dll"
    else:
        raise RuntimeError("Check demangler native build needs a supported LLVM SDK host")
    basename = "developer_tools_check" if suffix == "dll" else "libdeveloper_tools_check"
    subprocess.run([str(prefix / "bin/clang++"), "-std=c++17", "-O2", "-Wall", "-Wextra", "-Werror",
                    "-isystem", str(prefix / "include"), *flags, str(directory / "signatures.cpp"),
                    *libraries, "-o", str(directory / f"{basename}.{suffix}")], cwd=ROOT, check=True)
    if platform.system() == "Darwin":
        runtime = ROOT / "stdlib/internal/developer-tools/clang-bindings"
        image = directory / f"{basename}.{suffix}"
        names = [Path(name).name for name in developer_native_runtime.dependencies(image)
                 if not developer_native_runtime.system_library(name)]
        developer_native_runtime.stage_macos_sdk(prefix, runtime, names)
        developer_native_runtime.relocate_macos_helper(image, runtime, library=True)


if __name__ == "__main__":
    main()
