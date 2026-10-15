#!/usr/bin/env python3
"""SDK-evaluated byte-string macros through both actual generator pipelines."""
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
        raise RuntimeError(f"String macros failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    build_developer_bindings.build(prefix)
    oracle = ROOT / "target/developer-tools-string-evaluation"
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         ROOT / "tests/developer-tools/bindings-string-evaluation.c", "-L", prefix / "lib",
         "-Wl,-rpath," + str(prefix / "lib"), "-lclang", "-o", oracle])
    run([oracle])
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    source = ROOT / "tests/developer-tools/bindings-string-macros.jai"
    consumer = ROOT / "tests/developer-tools/bindings-string-macros-roundtrip.jai"
    generated = [ROOT / f"target/developer-tools-string-macros-{name}.jai" for name in ["generated", "custom"]]
    generated.append(ROOT / "target/developer-tools-original-string-generated.jai")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-string-macros-emitter-o{mode}"
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "tests/developer-tools/bindings-string-macros-native.c",
                 "-o", ROOT / f"target/libdeveloper_tools_string_macros.{suffix}"])
            run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-string-macros-consumer-{mode}-o{optimization}"
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [source, consumer, *generated, ROOT / "tests/developer-tools/bindings-macros.jai",
             ROOT / "stdlib/Bindings_Generator/module.jai", ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    print("Real byte-string/escaped/concatenated/alias/UTF8/empty macros passed both generator paths interpreter/native Mac O0/O2, configured C O0/O2, rejection preservation, formatting and strict lint")


if __name__ == "__main__":
    main()
