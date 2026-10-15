#!/usr/bin/env python3
"""Real C variadic generation, public type printing and independent ABI effects."""
import os
from pathlib import Path
import platform
import subprocess
import build_developer_bindings

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != expected:
        raise RuntimeError(f"C variadic workflow failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    build_developer_bindings.build(prefix)
    tests = ROOT / "tests/developer-tools"
    jaic = ROOT / "target/debug/jaic"
    emitter = tests / "bindings-variadic.jai"
    consumer = tests / "bindings-variadic-roundtrip.jai"
    producer = tests / "producer-requests/c-variadic-abi.jai"
    generated = [ROOT / f"target/developer-tools-variadic-{name}.jai" for name in ["default", "generated", "signature"]]
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, emitter, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-variadic-emitter-o{mode}"
            run([jaic, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 *flags, tests / "bindings-variadic-native.c", "-o", ROOT / f"target/libdeveloper_tools_variadic.{suffix}"])
            for source, tag in [(producer, "producer"), (consumer, "generated")]:
                run([jaic, source, "-interp", "-I", ROOT / "target"], "ok\n")
                executable = ROOT / f"target/developer-tools-variadic-{tag}-{mode}-o{optimization}"
                run([jaic, source, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
                run([executable], "ok\n")
    paths = [emitter, consumer, producer, *generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    for triple, os_tag, cpu in [("x86_64-pc-windows-msvc", "windows", "x64"),
                                ("aarch64-unknown-linux-gnu", "linux", "arm64")]:
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", tests / "bindings-variadic-native.c", "-o", ROOT / f"target/developer-tools-variadic-{triple}.o"])
        run([jaic, consumer, "-check", "-os", os_tag, "-cpu", cpu, "-I", ROOT / "target"])
    print("C variadic default/custom generation and public type printer passed interpreter/native Mac O0/O2 with C O0/O2 integer/float promotions, pointer args, empty tails, returned function pointers, collision-free names, format and strict lint; Windows/Linux object/source checks only")


if __name__ == "__main__":
    main()
