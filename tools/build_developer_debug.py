#!/usr/bin/env python3
"""Build only this worker's independent native Debug helper in this worktree."""
from pathlib import Path
import os
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT / "stdlib/internal/developer-tools/debug-native"
    system = platform.system()
    prefix = os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX")
    compiler = str(Path(prefix) / "bin/clang") if prefix else "clang"
    if system == "Darwin":
        flags, suffix = ["-dynamiclib"], "dylib"
    elif system == "Linux":
        flags, suffix = ["-shared", "-fPIC"], "so"
    elif system == "Windows":
        output = directory / "developer_tools_debug.dll"
        subprocess.run([compiler, "-std=c11", "-O2", "-fno-omit-frame-pointer", "-Wall", "-Wextra", "-Werror",
                        "-shared", str(directory / "runtime-windows.c"), "-ldbghelp", "-lkernel32", "-luser32", "-ladvapi32", "-o", str(output)],
                       cwd=ROOT, check=True)
        return
    else:
        raise RuntimeError("Native Debug helper currently supports macOS/Linux/Windows hosts")
    subprocess.run([compiler, "-std=c11", "-O2", "-fno-omit-frame-pointer", "-Wall", "-Wextra", "-Werror",
                    *flags, str(directory / "runtime.c"), "-o", str(directory / f"libdeveloper_tools_debug.{suffix}")],
                   cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
