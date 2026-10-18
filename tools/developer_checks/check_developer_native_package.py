#!/usr/bin/env python3
"""Actual relocated Mac native package loading, independent of SDK load paths."""
import os
import json
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    environment = {key: value for key, value in os.environ.items() if not key.startswith("DYLD_")}
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=environment,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr:
        raise RuntimeError(f"Native package regression: status={result.returncode}; "
                           f"diagnostic={result.stderr.splitlines()[:2]}")
    return result.stdout


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("This test requires actual Mac dyld execution")
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    original = ROOT / "target/developer-tools-package-before"
    relocated = ROOT / "target/developer-tools-package-relocated"
    for directory in [original, relocated]:
        if directory.exists():
            shutil.rmtree(directory)
    private = original / "stdlib/internal/developer-tools"
    clang_dir = private / "clang-bindings"
    check_dir = private / "check-native"
    clang_dir.mkdir(parents=True)
    check_dir.mkdir()
    source = ROOT / "stdlib/internal/developer-tools"
    shutil.copy2(source / "clang-bindings/developer-tools-clang", clang_dir)
    manifest = json.loads((source / "clang-bindings/runtime-sdk.json").read_text())
    for name in manifest:
        shutil.copy2(source / "clang-bindings" / name, clang_dir / name)
    shutil.copy2(source / "check-native/libdeveloper_tools_check.dylib", check_dir)
    header = original / "original.h"
    header.write_text("typedef struct Original { int value; } Original;\nint original(int value);\n")
    for optimization in [0, 2]:
        run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             "-I", prefix / "include", ROOT / "stdlib/Debug_Info/tests/driven/native-package-probe.c",
             "-o", original / f"probe-o{optimization}"])
    original.rename(relocated)
    private = relocated / "stdlib/internal/developer-tools"
    for optimization in [0, 2]:
        run([relocated / f"probe-o{optimization}", relocated,
             private / "clang-bindings/libclang.dylib", private / "check-native/libdeveloper_tools_check.dylib",
             relocated / "original.h"])
    # A new process must fail when the bundled dependency is unavailable; it
    # must not recover through the configured/Homebrew SDK's copy.
    llvm = next(name for name in manifest if name.startswith("libLLVM."))
    required = private / "clang-bindings" / llvm
    hidden = required.with_suffix(".temporarily-unavailable")
    required.rename(hidden)
    try:
        result = subprocess.run([str(relocated / "probe-o0"), str(relocated),
                                 str(private / "clang-bindings/libclang.dylib"),
                                 str(private / "check-native/libdeveloper_tools_check.dylib"),
                                 str(relocated / "original.h")], capture_output=True, text=True, timeout=90,
                                env={key: value for key, value in os.environ.items() if not key.startswith("DYLD_")})
        if result.returncode != 2 or llvm not in result.stderr:
            raise RuntimeError("Missing bundled LLVM must fail without recovering through a developer SDK")
    finally:
        hidden.rename(required)
    generated = relocated / "generated.jai"
    run([private / "clang-bindings/developer-tools-clang", relocated / "original.h", generated, "native"])
    if "Original :: struct" not in generated.read_text() or "original ::" not in generated.read_text():
        raise RuntimeError("Relocated native generator did not emit actual original declarations")
    print("Relocated Mac SDK closure loads from selected package; actual libclang parsing, helper emission and Check demangling passed native O0/O2 without DYLD overrides")


if __name__ == "__main__":
    main()
