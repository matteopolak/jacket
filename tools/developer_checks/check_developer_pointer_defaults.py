#!/usr/bin/env python3
"""Real pointer-default calls through the reviewed normalizer/signature printer."""
import platform
from dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from build_developer_bindings import llvm_prefix


run = labeled_runner('Pointer defaults')


def main():
    prefix = llvm_prefix()
    jaic = cargo_binary("jaic")
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-pointer-defaults.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-pointer-defaults-roundtrip.jai"
    generated = output_dir("developer-tools-pointer-defaults-generated.jai")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-pointer-defaults-emitter-o{mode}")
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-pointer-defaults-native.c",
                 "-o", output_dir(f"libdeveloper_tools_pointer_defaults.{suffix}")])
            run([jaic, consumer, "-interp", "-I", output_dir()], "ok\n")
            executable = output_dir(f"developer-tools-pointer-defaults-consumer-{mode}-o{optimization}")
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [source, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-expression-normalization.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    print("Public zero/null normalization and real pointer-default calls passed interpreter/native Mac O0/O2, configured C O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
