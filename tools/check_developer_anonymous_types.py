#!/usr/bin/env python3
"""Reviewed anonymous typedef normalization and real C ABI consumers."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected=""):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
        raise RuntimeError(f"Anonymous typedef workflow failed: status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")
    return result.stdout


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    tests = ROOT / "tests/developer-tools"
    native = ROOT / "stdlib/internal/developer-tools/clang-bindings"
    oracle = ROOT / "target/developer-tools-anonymous-oracle"
    run([prefix / "bin/clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
         "-I", prefix / "include", tests / "bindings-anonymous-oracle.c", "-L", native,
         "-Wl,-rpath," + str(native), "-lclang", "-o", oracle])
    observations = [line.split("|") for line in run([oracle, tests / "bindings-anonymous.h"], None).splitlines()]
    # SDK inferred tag spelling/anonymous-query values are observations, not
    # compatibility oracles. Source location and canonical identity are tested.
    if [(row[0], row[2], row[5]) for row in observations] != [
            ("Anonymous_Record", "2", "struct"), ("Anonymous_Union", "3", "union"),
            ("Anonymous_Mode", "5", "enum"), ("Record_Alias", "2", "struct"),
            ("Mode_Alias", "5", "enum"), ("Forward_Record", "2", "Forward_Record")]:
        raise RuntimeError("SDK tag location/identity did not match original source")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    jaic = ROOT / "target/debug/jaic"
    emitter = tests / "bindings-anonymous.jai"
    consumer = tests / "bindings-anonymous-roundtrip.jai"
    generated = ROOT / "target/developer-tools-anonymous-generated.jai"
    default_generated = ROOT / "target/developer-tools-anonymous-default.jai"
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, emitter, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-anonymous-emitter-o{mode}"
            run([jaic, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated, default_generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 *flags, tests / "bindings-anonymous-native.c", "-o", ROOT / f"target/libdeveloper_tools_anonymous.{suffix}"])
            run([jaic, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = ROOT / f"target/developer-tools-anonymous-consumer-{mode}-o{optimization}"
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [emitter, consumer, generated, default_generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    for triple, os_tag, cpu in [("x86_64-pc-windows-msvc", "windows", "x64"),
                                ("aarch64-unknown-linux-gnu", "linux", "arm64")]:
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", tests / "bindings-anonymous-native.c", "-o", ROOT / f"target/developer-tools-anonymous-{triple}.o"])
        run([jaic, consumer, "-check", "-os", os_tag, "-cpu", cpu, "-I", ROOT / "target"])
    print("Anonymous record/union/enum naming, omitted naming aliases, retained secondary aliases and forward identity passed interpreter/native Mac O0/O2, C O0/O2, format and strict lint; Windows/Linux target object/source checks only")


if __name__ == "__main__":
    main()
