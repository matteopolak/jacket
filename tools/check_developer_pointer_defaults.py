#!/usr/bin/env python3
"""Real pointer-default calls through the reviewed normalizer/signature printer."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != expected:
        raise RuntimeError(f"Pointer defaults failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    source = ROOT / "tests/developer-tools/bindings-pointer-defaults.jai"
    consumer = ROOT / "tests/developer-tools/bindings-pointer-defaults-roundtrip.jai"
    generated = ROOT / "target/developer-tools-pointer-defaults-generated.jai"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, source, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-pointer-defaults-emitter-o{mode}"
            run([jaic, source, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", *flags,
                 ROOT / "tests/developer-tools/bindings-pointer-defaults-native.c",
                 "-o", ROOT / f"target/libdeveloper_tools_pointer_defaults.{suffix}"])
            run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-pointer-defaults-consumer-{mode}-o{optimization}"
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [source, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-expression-normalization.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    print("Public zero/null normalization and real pointer-default calls passed interpreter/native Mac O0/O2, configured C O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
