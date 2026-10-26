#!/usr/bin/env python3
"""SDK-evaluated byte-string macros through both actual generator pipelines."""
import platform
import build_developer_bindings
from dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from build_developer_bindings import llvm_prefix


run = labeled_runner('String macros')


def main():
    prefix = llvm_prefix()
    build_developer_bindings.build(prefix)
    oracle = output_dir("developer-tools-string-evaluation")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-string-evaluation.c", "-L", prefix / "lib",
         "-Wl,-rpath," + str(prefix / "lib"), "-lclang", "-o", oracle])
    run([oracle])
    jaic = cargo_binary("jaic")
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-string-macros.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-string-macros-roundtrip.jai"
    generated = [output_dir(f"developer-tools-string-macros-{name}.jai") for name in ["generated", "custom"]]
    generated.append(output_dir("developer-tools-original-string-generated.jai"))
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    # The generator loads the configured library, so it must exist before the first generation.
    run([prefix / "bin/clang", "-std=c11", "-O0", *flags, ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-string-macros-native.c",
         "-o", output_dir(f"libdeveloper_tools_string_macros.{suffix}")])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-string-macros-emitter-o{mode}")
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-string-macros-native.c",
                 "-o", output_dir(f"libdeveloper_tools_string_macros.{suffix}")])
            run([jaic, consumer, "-interp", "-I", output_dir()], "ok\n")
            executable = output_dir(f"developer-tools-string-macros-consumer-{mode}-o{optimization}")
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [source, consumer, *generated, ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-macros.jai",
             ROOT / "stdlib/Bindings_Generator/module.jai", ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    print("Real byte-string/escaped/concatenated/alias/UTF8/empty macros passed both generator paths interpreter/native Mac O0/O2, configured C O0/O2, rejection preservation, formatting and strict lint")


if __name__ == "__main__":
    main()
