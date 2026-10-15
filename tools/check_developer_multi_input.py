#!/usr/bin/env python3
"""Public multiple-header bindings and C++ bridges with actual native calls."""
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
        raise RuntimeError(f"Multiple-input workflow failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    source = ROOT / "tests/developer-tools/bindings-multi.jai"
    consumer = ROOT / "tests/developer-tools/bindings-multi-roundtrip.jai"
    native = ROOT / "tests/developer-tools/bindings-multi-native.c"
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    generated = [ROOT / f"target/developer-tools-multi-{name}.jai"
                 for name in ["generated", "custom", "cpp-generated"]]
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-multi-emitter-o{mode}"
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 native, "-o", ROOT / f"target/libdeveloper_tools_multi.{suffix}"])
            run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 native, ROOT / "target/developer-tools-multi-cpp-bridge.cpp",
                 "-o", ROOT / f"target/libdeveloper_tools_multi_cpp.{suffix}"])
            run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-multi-consumer-{mode}-o{optimization}"
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [source, consumer, *generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/bindings-driver.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", "--os", "windows", "--cpu", "x64",
         ROOT / "stdlib/Bindings_Generator/module.jai"])
    print("Multiple-header C bindings/customization/C++ bridges passed interpreter/native Mac O0/O2, configured native O0/O2 consumers, diagnostics/preservation, formatting and strict lint")


if __name__ == "__main__":
    main()
