#!/usr/bin/env python3
"""Targeted original C++ customization and C-model regression effects on the host."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != expected:
        diagnostic = (result.stderr or result.stdout).splitlines()[:2]
        raise RuntimeError(f"Targeted C++ workflow failed: {command}; status={result.returncode}; diagnostic={diagnostic}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    emitter = ROOT / "tests/developer-tools/bindings-cpp-custom.jai"
    consumer = ROOT / "tests/developer-tools/bindings-cpp-custom-roundtrip.jai"
    generated = ROOT / "target/developer-tools-cpp-custom-generated.jai"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([JAIC, emitter, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-cpp-custom-emitter-o{mode}"
            run([JAIC, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated])
        run([ROOT / "target/debug/jailint", "-D", "warnings", generated])
        flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
        suffix = "dylib" if platform.system() == "Darwin" else "so"
        for optimization in [0, 2]:
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "target/developer-tools-cpp-custom.cpp", ROOT / "tests/developer-tools/bindings-cpp-custom-native.cpp",
                 "-o", ROOT / f"target/libdeveloper_tools_cpp_custom.{suffix}"])
            run([JAIC, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-cpp-custom-consumer-{mode}-o{optimization}"
            run([JAIC, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    # Same model/parser changed: retain real C callback, macro and type effects.
    for name in ["bindings-custom", "bindings-model-properties", "bindings-macro-custom", "bindings-macro-enums", "bindings-macro-merge"]:
        source = ROOT / f"tests/developer-tools/{name}.jai"
        if not source.exists():
            raise RuntimeError(f"Required existing model regression fixture is absent: {source}")
        run([JAIC, source, "-interp", "-I", ROOT / "target"], "ok\n")
        for optimization in [0, 2]:
            executable = ROOT / f"target/developer-tools-cpp-regression-{name}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    print("Customized C++ overload/POD/reference/argument callbacks and C-model regressions passed interpreter/native Mac O0/O2")


if __name__ == "__main__":
    main()
