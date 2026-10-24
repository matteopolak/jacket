#!/usr/bin/env python3
"""Generated functions that return procedures: parenthesized result, then real calls against C."""
import os
from pathlib import Path
import platform
from dev_common import JAIFMT, ROOT, labeled_runner
from build_developer_bindings import llvm_prefix


run = labeled_runner('Procedure results')


def main():
    prefix = llvm_prefix()
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    driven = ROOT / "stdlib/Bindings_Generator/tests/driven"
    source = driven / "bindings-procedure-results.jai"
    consumer = driven / "bindings-procedure-results-roundtrip.jai"
    generated = ROOT / "target/developer-tools-procedure-results-generated.jai"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    library = ROOT / f"target/libdeveloper_tools_procedure_results.{suffix}"

    def build_library(optimization):
        run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
             driven / "bindings-procedure-results-native.c", "-o", library])

    # The generator reads the library's exported symbols, so it is built before generating.
    build_library(0)
    run([jaic, source, "-interp"], "ok\n")
    run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
    for optimization in [0, 2]:
        build_library(optimization)
        run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
        executable = ROOT / f"target/developer-tools-procedure-results-consumer-o{optimization}"
        run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
        run([executable], "ok\n")
    # The emitter is formatted here but linted with the rest of the stdlib: alone it does not compile.
    paths = [source, consumer, generated, ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths[:3]])
    print("Functions returning procedures print the result in parentheses and call through real C in the interpreter and native O0/O2")


if __name__ == "__main__":
    main()
