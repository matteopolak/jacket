#!/usr/bin/env python3
"""Public generator type printers through actual generated C++/Jai consumers."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, output=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != output:
        raise RuntimeError(f"Type emission failure: {command}; status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    source = ROOT / "tests/developer-tools/bindings-type-emission.jai"
    errors = ROOT / "tests/developer-tools/bindings-type-emission-errors.jai"
    consumer = ROOT / "tests/developer-tools/bindings-type-emission-roundtrip.jai"
    generated = ROOT / "target/developer-tools-type-emission-generated.jai"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for mode in ["interp", "0", "2"]:
        for fixture in [source, errors]:
            if mode == "interp":
                run([JAIC, fixture, "-interp"], "ok\n")
            else:
                executable = ROOT / f"target/developer-tools-{fixture.stem}-o{mode}"
                run([JAIC, fixture, f"-O{mode}", "-exe", executable])
                run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "target/developer-tools-type-emission.cpp", "-o", ROOT / f"target/libdeveloper_tools_type_emission.{suffix}"])
            run([JAIC, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-type-consumer-{mode}-o{optimization}"
            run([JAIC, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
        # Existing public fragment printers share the C++ type renderer.
        fragments = ROOT / "tests/developer-tools/bindings-cpp-fragments.jai"
        if mode == "interp":
            run([JAIC, fragments, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-type-fragments-o{mode}"
            run([JAIC, fragments, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        for optimization in [0, 2]:
            executable = ROOT / f"target/developer-tools-type-fragments-{mode}-cxx{optimization}"
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 ROOT / "target/developer-tools-cpp-fragments.cpp", "-o", executable])
            run([executable])
    paths = [source, errors, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai",
             ROOT / "stdlib/internal/developer-tools/cpp-fragments.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai",
             ROOT / "stdlib/internal/developer-tools/binding-cpp-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    print("Seven public generator type/query helpers passed interpreter/native Mac O0/O2 and configured C++ O0/O2 consumers, diagnostics, formatting and strict lint")


if __name__ == "__main__":
    main()
