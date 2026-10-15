#!/usr/bin/env python3
"""Public cursor token reconstruction into compiled independently authored C++."""
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or result.stdout != expected:
        raise RuntimeError(f"Cursor reconstruction failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    source = ROOT / "tests/developer-tools/bindings-cursor-source.jai"
    queries = ROOT / "tests/developer-tools/bindings-type-queries.jai"
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    for mode in ["interp", "0", "2"]:
        for fixture in [source, queries]:
            if mode == "interp":
                run([jaic, fixture, "-interp"], "ok\n")
            else:
                executable = ROOT / f"target/developer-tools-{fixture.stem}-o{mode}"
                run([jaic, fixture, f"-O{mode}", "-exe", executable])
                run([executable], "ok\n")
        for optimization in [0, 2]:
            consumer = ROOT / f"target/developer-tools-cursor-consumer-{mode}-o{optimization}"
            run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 ROOT / "target/developer-tools-cursor-reconstructed.cpp", "-o", consumer])
            run([consumer])
    paths = [source, queries, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-cursor-source.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-queries.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", *paths])
    print("Five public cursor/type helpers passed interpreter/native Mac O0/O2, configured C++ O0/O2 consumers, cycle diagnostics, formatting and strict lint")


if __name__ == "__main__":
    main()
