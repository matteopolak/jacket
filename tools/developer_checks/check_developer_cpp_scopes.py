#!/usr/bin/env python3
"""Canonical C++ scopes, explicit callback flattening and real native adapters."""
import platform
from dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from build_developer_bindings import llvm_prefix


run = labeled_runner('C++ scope workflow')


def main():
    prefix = llvm_prefix()
    oracle = output_dir("developer-tools-cpp-function-types")
    run([prefix / "bin/clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-function-types.c", "-L", prefix / "lib",
         "-Wl,-rpath," + str(prefix / "lib"), "-lclang", "-o", oracle])
    run([oracle])
    jaic = cargo_binary("jaic")
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-roundtrip.jai"
    generated = output_dir("developer-tools-cpp-scopes-generated.jai")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    # The generator loads the configured library, so it must exist before the first
    # generation; it is rebuilt with the generated bridge below.
    run([prefix / "bin/clang++", "-std=c++17", "-O0", *flags, ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-native.cpp",
         "-o", output_dir(f"libdeveloper_tools_cpp_scopes.{suffix}")])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-cpp-scopes-emitter-o{mode}")
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-scopes-native.cpp",
                 output_dir("developer-tools-cpp-scopes-bridge.cpp"),
                 "-o", output_dir(f"libdeveloper_tools_cpp_scopes.{suffix}")])
            run([jaic, consumer, "-interp", "-I", output_dir()], "ok\n")
            executable = output_dir(f"developer-tools-cpp-scopes-consumer-{mode}-o{optimization}")
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [source, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai",
             ROOT / "stdlib/internal/developer-tools/binding-cpp-model.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    print("Real nested/reopened C++ scopes, same-name record identity, explicit flattened adapter names, POD/scalar calls and diagnosed collision/scope/cycle preservation passed interpreter/native Mac O0/O2, configured C++ O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
