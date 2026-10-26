#!/usr/bin/env python3
"""Real C variadic generation, public type printing and independent ABI effects."""
import platform
from developer_checks import build_developer_bindings
from developer_checks.driven_sources import driven
from developer_checks.dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from developer_checks.build_developer_bindings import llvm_prefix


run = labeled_runner('C variadic workflow')


def main():
    prefix = llvm_prefix()
    build_developer_bindings.build(prefix)
    jaic = cargo_binary("jaic")
    emitter = driven("bindings-variadic.jai")
    consumer = driven("bindings-variadic-roundtrip.jai")
    generated = [output_dir(f"developer-tools-variadic-{name}.jai") for name in ["default", "generated", "signature"]]
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    # The generator loads the configured library, so it must exist before the first generation.
    run([prefix / "bin/clang", "-std=c11", "-O0", *flags, driven("bindings-variadic-native.c"),
         "-o", output_dir(f"libdeveloper_tools_variadic.{suffix}")])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, emitter, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-variadic-emitter-o{mode}")
            run([jaic, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 *flags, driven("bindings-variadic-native.c"), "-o", output_dir(f"libdeveloper_tools_variadic.{suffix}")])
            for source, tag in [(consumer, "generated")]:
                run([jaic, source, "-interp", "-I", output_dir()], "ok\n")
                executable = output_dir(f"developer-tools-variadic-{tag}-{mode}-o{optimization}")
                run([jaic, source, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
                run([executable], "ok\n")
    paths = [emitter, consumer, *generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    for triple, os_tag, cpu in [("x86_64-pc-windows-msvc", "windows", "x64"),
                                ("aarch64-unknown-linux-gnu", "linux", "arm64")]:
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", driven("bindings-variadic-native.c"), "-o", output_dir(f"developer-tools-variadic-{triple}.o")])
        run([jaic, consumer, "-check", "-os", os_tag, "-cpu", cpu, "-I", output_dir()])
    print("C variadic default/custom generation and public type printer passed interpreter/native Mac O0/O2 with C O0/O2 integer/float promotions, pointer args, empty tails, returned function pointers, collision-free names, format and strict lint; Windows/Linux object/source checks only")


if __name__ == "__main__":
    main()
