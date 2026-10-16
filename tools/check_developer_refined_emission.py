#!/usr/bin/env python3
"""Reviewed cursor spelling and value/type contracts against shared production."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from build_developer_bindings import llvm_prefix

ROOT = Path(__file__).resolve().parents[1]


def main():
    target = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    scratch = Path(tempfile.mkdtemp(prefix="refined-emission-", dir=target))
    compiler, formatter, linter = (
        target / "debug/jaic",
        target / "jaifmt",
        target / "debug/jailint",
    )
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), TMPDIR=str(scratch))
    clang = llvm_prefix() / "bin/clang++"
    tests = [
        ROOT / "tests/developer-tools" / ("bindings-" + name + ".jai")
        for name in [
            "cursor-source",
            "type-queries",
            "refined-values",
            "generic-record",
            "pointer-recovery",
            "array-normalization",
            "wide-layout",
            "generic-driver",
        ]
    ]
    production = [
        ROOT / "stdlib/internal/developer-tools" / ("binding-" + name + ".jai")
        for name in [
            "cursor-source",
            "type-queries",
            "type-emission",
            "value-emission",
            "expression-normalization",
            "declaration-emission",
            "canonical-driver",
        ]
    ]
    production.append(ROOT / "stdlib/internal/developer-tools/cpp-fragments.jai")
    sources = list((ROOT / "stdlib/Bindings_Generator").rglob("*.jai"))
    sources += list((ROOT / "stdlib/internal/developer-tools").rglob("*.jai"))
    negative = ROOT / "tests/developer-tools/bindings-hex128-negative.jai"
    sources += tests + [negative, Path(__file__)]
    sources += list((ROOT / "tests/developer-tools").glob("bindings-wide-layout.*"))
    sources += [
        ROOT / "tests/developer-tools/bindings-generic-record.hpp",
        ROOT / "tests/developer-tools/bindings-generic-native.cpp",
    ]

    def snapshot():
        return {
            str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(set(sources))
        }

    before = snapshot()
    receipt = {
        "passed": False,
        "source_sha256": before,
        "compiler_sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
        "commands": [],
        "native_platform": "Mac host only",
        "retired_oracle": "Cursor concatenation is not compilable C++ reconstruction",
    }

    def run(command, expected="", negative=False):
        result = subprocess.run(
            [str(v) for v in command],
            cwd=scratch,
            env=env,
            capture_output=True,
            text=True,
            timeout=90,
        )
        passed = (
            result.returncode == 0 and result.stdout == expected and not result.stderr
        )
        if negative:
            passed = (
                result.returncode != 0
                and "unsupported integer emission width"
                in result.stdout + result.stderr
            )
        index = len(receipt["commands"])
        (scratch / (str(index) + ".stdout.txt")).write_text(result.stdout)
        (scratch / (str(index) + ".stderr.txt")).write_text(result.stderr)
        receipt["commands"].append(
            {
                "command": [str(v) for v in command],
                "exit": result.returncode,
                "passed": passed,
                "first_diagnostic": result.stderr.splitlines()[:1],
            }
        )
        if not passed:
            raise RuntimeError("Refined emission check failed; see " + str(scratch))

    try:
        generated_sources = []
        for source in tests:
            generated = scratch / (source.stem + "-generated.jai")
            arguments = (
                [generated]
                if source.stem
                in [
                    "bindings-refined-values",
                    "bindings-generic-record",
                    "bindings-wide-layout",
                    "bindings-generic-driver",
                ]
                else []
            )
            if arguments:
                generated_sources.append(generated)
            if source.stem == "bindings-refined-values":
                arguments.append(scratch / "generated-raw.cpp")
            run([compiler, source, "-interp", "--", *arguments], "ok\n")
            for optimization in [0, 2]:
                binary = scratch / (source.stem + "-o" + str(optimization))
                run([compiler, source, "-O" + str(optimization), "-exe", binary])
                run([binary, *arguments], "ok\n")
        for generated in generated_sources:
            if generated.stem == "bindings-generic-driver-generated":
                run(
                    [
                        clang,
                        "-std=c++17",
                        "-Wall",
                        "-Wextra",
                        "-Werror",
                        "-O2",
                        "-dynamiclib",
                        ROOT / "tests/developer-tools/bindings-generic-native.cpp",
                        "-o",
                        scratch / "libgeneric_fixture.dylib",
                    ]
                )
            if generated.stem == "bindings-wide-layout-generated":
                run(
                    [
                        clang.parent / "clang",
                        "-std=c11",
                        "-Wall",
                        "-Wextra",
                        "-Werror",
                        "-O2",
                        "-dynamiclib",
                        ROOT / "tests/developer-tools/bindings-wide-layout.c",
                        "-o",
                        scratch / "libwide_fixture.dylib",
                    ]
                )
            run([formatter, "--config", ROOT / "jaifmt.toml", generated])
            run([compiler, generated, "-interp"], "ok\n")
            for optimization in [0, 2]:
                binary = scratch / (generated.stem + "-o" + str(optimization))
                run([compiler, generated, "-O" + str(optimization), "-exe", binary])
                run([binary], "ok\n")
        for optimization in [0, 2]:
            binary = scratch / ("raw-consumer-o" + str(optimization))
            run(
                [
                    clang,
                    "-std=c++17",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-O" + str(optimization),
                    scratch / "generated-raw.cpp",
                    "-o",
                    binary,
                ]
            )
            run([binary])
        for mode in ["signed", "unsigned"]:
            run([compiler, negative, "-interp", "--", mode], negative=True)
        for optimization in [0, 2]:
            binary = scratch / ("hex-negative-o" + str(optimization))
            run([compiler, negative, "-O" + str(optimization), "-exe", binary])
            for mode in ["signed", "unsigned"]:
                run([binary, mode], negative=True)
        run(
            [
                formatter,
                "--config",
                ROOT / "jaifmt.toml",
                "--check",
                *tests,
                *production,
                *generated_sources,
                negative,
            ]
        )
        run([linter, "-D", "warnings", *tests, *generated_sources, negative])
        for source in production:
            run(
                [
                    linter,
                    "-D",
                    "warnings",
                    ROOT / "stdlib/Bindings_Generator/module.jai",
                    source,
                ]
            )
        receipt["passed"] = True
    finally:
        receipt["source_unchanged"] = snapshot() == before
        receipt["passed"] &= receipt["source_unchanged"]
        (scratch / "verification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if not receipt["passed"]:
        raise RuntimeError("Source changed during verification: " + str(scratch))
    print("Refined emission PASS " + str(scratch / "verification.json"))


if __name__ == "__main__":
    main()
