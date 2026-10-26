#!/usr/bin/env python3
"""Public multiple-header bindings and C++ bridges with actual native calls."""
import platform
from developer_checks.dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from developer_checks.build_developer_bindings import llvm_prefix


run = labeled_runner('Multiple-input workflow')


def main():
    prefix = llvm_prefix()
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-multi.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-multi-roundtrip.jai"
    native = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-multi-native.c"
    jaic = cargo_binary("jaic")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    generated = [output_dir(f"developer-tools-multi-{name}.jai")
                 for name in ["generated", "custom", "cpp-generated"]]
    # The generator loads every configured library, so both must exist before the
    # first generation; the C++ one is rebuilt with the generated bridge below.
    run([prefix / "bin/clang", "-std=c11", "-O0", *flags, native,
         "-o", output_dir(f"libdeveloper_tools_multi.{suffix}")])
    run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-O0", *flags, native,
         "-o", output_dir(f"libdeveloper_tools_multi_cpp.{suffix}")])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-multi-emitter-o{mode}")
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 native, "-o", output_dir(f"libdeveloper_tools_multi.{suffix}")])
            run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 native, output_dir("developer-tools-multi-cpp-bridge.cpp"),
                 "-o", output_dir(f"libdeveloper_tools_multi_cpp.{suffix}")])
            run([jaic, consumer, "-interp", "-I", output_dir()], "ok\n")
            executable = output_dir(f"developer-tools-multi-consumer-{mode}-o{optimization}")
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [source, consumer, *generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/bindings-driver.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-os", "windows", "-cpu", "x64",
         ROOT / "stdlib/Bindings_Generator/module.jai"])
    print("Multiple-header C bindings/customization/C++ bridges passed interpreter/native Mac O0/O2, configured native O0/O2 consumers, diagnostics/preservation, formatting and strict lint")


if __name__ == "__main__":
    main()
