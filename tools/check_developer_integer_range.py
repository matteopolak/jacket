#!/usr/bin/env python3
"""Public signed-range checks against an original native Clang integer oracle."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != expected:
        raise RuntimeError(f"Signed range failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    source = ROOT / "tests/developer-tools/bindings-integer-range.jai"
    native = ROOT / "tests/developer-tools/bindings-integer-range-native.c"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for optimization in [0, 2]:
        run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
             native, "-o", ROOT / f"target/libdeveloper_tools_integer_range.{suffix}"])
        run([jaic, source, "-interp"], "ok\n")
        executable = ROOT / f"target/developer-tools-integer-range-o{optimization}"
        run([jaic, source, f"-O{optimization}", "-exe", executable])
        run([executable], "ok\n")
    run([prefix / "bin/clang", "-std=c11", "--target=x86_64-pc-windows-msvc", "-Wall", "-Wextra", "-Werror",
         "-c", native, "-o", ROOT / "target/developer-tools-integer-range-windows.obj"])
    run([jaic, source, "-check", "-os", "windows", "-cpu", "x64"])
    paths = [source, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", *paths])
    print("Signed8/16/32/64/128 range/default/alias/enum checks passed interpreter/native Mac O0/O2 and configured C O0/O2; Windows source/object checks, formatting and strict lint passed")


if __name__ == "__main__":
    main()
