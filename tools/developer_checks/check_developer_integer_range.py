#!/usr/bin/env python3
"""Public signed-range checks against an original native Clang integer oracle."""
import platform
from dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from build_developer_bindings import llvm_prefix


run = labeled_runner('Signed range')


def main():
    prefix = llvm_prefix()
    jaic = cargo_binary("jaic")
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-integer-range.jai"
    native = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-integer-range-native.c"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for optimization in [0, 2]:
        run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
             native, "-o", output_dir(f"libdeveloper_tools_integer_range.{suffix}")])
        run([jaic, source, "-interp"], "ok\n")
        executable = output_dir(f"developer-tools-integer-range-o{optimization}")
        run([jaic, source, f"-O{optimization}", "-exe", executable])
        run([executable], "ok\n")
    run([prefix / "bin/clang", "-std=c11", "--target=x86_64-pc-windows-msvc", "-Wall", "-Wextra", "-Werror",
         "-c", native, "-o", output_dir("developer-tools-integer-range-windows.obj")])
    run([jaic, source, "-check", "-os", "windows", "-cpu", "x64"])
    paths = [source, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", *paths])
    print("Signed8/16/32/64/128 range/default/alias/enum checks passed interpreter/native Mac O0/O2 and configured C O0/O2; Windows source/object checks, formatting and strict lint passed")


if __name__ == "__main__":
    main()
