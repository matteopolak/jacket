#!/usr/bin/env python3
"""Canonical C++ scopes, explicit callback flattening and real native adapters."""
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
        raise RuntimeError(f"C++ scope workflow failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    oracle = ROOT / "target/developer-tools-cpp-function-types"
    run([prefix / "bin/clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-function-types.c", "-L", prefix / "lib",
         "-Wl,-rpath," + str(prefix / "lib"), "-lclang", "-o", oracle])
    run([oracle])
    jaic = ROOT / "target/debug/jaic"
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-roundtrip.jai"
    generated = ROOT / "target/developer-tools-cpp-scopes-generated.jai"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    # The generator loads the configured library, so it must exist before the first
    # generation; it is rebuilt with the generated bridge below.
    run([prefix / "bin/clang++", "-std=c++17", "-O0", *flags, ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-native.cpp",
         "-o", ROOT / f"target/libdeveloper_tools_cpp_scopes.{suffix}"])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-cpp-scopes-emitter-o{mode}"
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-native.cpp",
                 ROOT / "target/developer-tools-cpp-scopes-bridge.cpp",
                 "-o", ROOT / f"target/libdeveloper_tools_cpp_scopes.{suffix}"])
            run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-cpp-scopes-consumer-{mode}-o{optimization}"
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [source, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai",
             ROOT / "stdlib/internal/developer-tools/binding-cpp-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    print("Real nested/reopened C++ scopes, same-name record identity, explicit flattened adapter names, POD/scalar calls and diagnosed collision/scope/cycle preservation passed interpreter/native Mac O0/O2, configured C++ O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
