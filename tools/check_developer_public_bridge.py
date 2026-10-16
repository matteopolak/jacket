#!/usr/bin/env python3
"""Current canonical public bridge/type/macro effects with private source identities."""

import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

from build_developer_bindings import llvm_prefix

ROOT = Path(__file__).resolve().parents[1]


def main():
    host = platform.system()
    if host not in ["Darwin", "Linux"]:
        raise RuntimeError("Native bridge runner requires a configured macOS/Linux SDK")
    target = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    scratch = Path(tempfile.mkdtemp(prefix="canonical-public-bridge-", dir=target))
    compiler, formatter, linter = (
        target / "debug/jaic",
        target / "jaifmt",
        target / "debug/jailint",
    )
    prefix = llvm_prefix()
    clang = prefix / "bin/clang++"
    resource = subprocess.check_output(
        [str(clang), "-print-resource-dir"], text=True
    ).strip()
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    tests = ROOT / "tests/developer-tools"
    fixtures = [
        tests / ("bindings-" + name + ".jai")
        for name in [
            "macro-value-printing",
            "public-type-shapes",
            "name-clashes",
            "type-emission-errors",
            "public-cpp-bridge",
            "public-cpp-bridge-roundtrip",
            "type-emission",
            "type-emission-roundtrip",
            "cpp-fragments",
            "template-arguments",
        ]
    ]
    sources = list((ROOT / "stdlib/Bindings_Generator").rglob("*.jai"))
    sources += list((ROOT / "stdlib/internal/developer-tools").rglob("*.jai"))
    sources += fixtures + [Path(__file__)]
    sources += list(tests.glob("bindings-public-cpp-bridge*"))
    sources += list(tests.glob("bindings-macro-value-printing*"))

    def snapshot():
        return {
            str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(set(sources))
            if p.is_file()
        }

    before = snapshot()
    generated_sources = [
        scratch / ("developer-tools-" + stem + "-generated.jai")
        for stem in ["public-bridge", "type-emission"]
    ]
    receipt = {
        "passed": False,
        "sdk": str(prefix),
        "resource_dir": resource,
        "source_sha256": before,
        "commands": [],
        "native_platform": host + " " + platform.machine(),
        "other_native_platforms": "not executed",
    }

    def run(command, expected=""):
        result = subprocess.run(
            [str(v) for v in command],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=90,
        )
        receipt["commands"].append(
            {
                "command": [str(v) for v in command],
                "exit": result.returncode,
                "stdout_match": result.stdout == expected,
                "stderr_empty": not result.stderr,
            }
        )
        if result.returncode or result.stdout != expected or result.stderr:
            raise RuntimeError(
                "Public bridge verification failed: "
                + (result.stderr or result.stdout)[:400]
            )

    def execute_fixture(name, mode, arguments=()):
        source = tests / ("bindings-" + name + ".jai")
        if mode == "interp":
            run([compiler, source, "-interp", "--", *arguments], "ok\n")
        else:
            executable = scratch / (name + "-o" + mode)
            run([compiler, source, "-O" + mode, "-exe", executable])
            run([executable, *arguments], "ok\n")

    try:
        for mode in ["interp", "0", "2"]:
            for name in [
                "macro-value-printing",
                "public-type-shapes",
                "name-clashes",
                "type-emission-errors",
                "template-arguments",
            ]:
                execute_fixture(name, mode)
            execute_fixture(
                "public-cpp-bridge", mode, [scratch, "-resource-dir", resource]
            )
            execute_fixture("type-emission", mode, [scratch])
            execute_fixture("cpp-fragments", mode, [scratch])
            for generated in generated_sources:
                run([formatter, "--config", ROOT / "jaifmt.toml", generated])
            for optimization in [0, 2]:
                flags = [
                    "-std=c++17",
                    "-O" + str(optimization),
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                ]
                executable = scratch / (
                    "cpp-consumer-" + mode + "-o" + str(optimization)
                )
                run(
                    [
                        clang,
                        *flags,
                        "-I",
                        scratch,
                        tests / "bindings-public-cpp-bridge-consumer.cpp",
                        "-o",
                        executable,
                    ]
                )
                run([executable])
                fragment_executable = scratch / (
                    "cpp-fragments-" + mode + "-o" + str(optimization)
                )
                run(
                    [
                        clang,
                        *flags,
                        scratch / "developer-tools-cpp-fragments.cpp",
                        "-o",
                        fragment_executable,
                    ]
                )
                run([fragment_executable])
                for stem, library, consumer in [
                    ("public-bridge", "public_bridge", "public-cpp-bridge-roundtrip"),
                    (
                        "type-emission",
                        "developer_tools_type_emission",
                        "type-emission-roundtrip",
                    ),
                ]:
                    library_flags = (
                        ["-dynamiclib"] if host == "Darwin" else ["-shared", "-fPIC"]
                    )
                    suffix = ".dylib" if host == "Darwin" else ".so"
                    run(
                        [
                            clang,
                            *flags,
                            *library_flags,
                            scratch / ("developer-tools-" + stem + ".cpp"),
                            "-o",
                            scratch / ("lib" + library + suffix),
                        ]
                    )
                    source = tests / ("bindings-" + consumer + ".jai")
                    run([compiler, source, "-interp", "-I", scratch], "ok\n")
                    executable = scratch / (
                        consumer + "-" + mode + "-o" + str(optimization)
                    )
                    run(
                        [
                            compiler,
                            source,
                            "-O" + str(optimization),
                            "-I",
                            scratch,
                            "-exe",
                            executable,
                        ]
                    )
                    run([executable], "ok\n")
        run(
            [
                formatter,
                "--config",
                ROOT / "jaifmt.toml",
                "--check",
                *fixtures,
                *generated_sources,
            ]
        )
        run(
            [
                linter,
                "-D",
                "warnings",
                "-I",
                scratch,
                *fixtures,
                *generated_sources,
                ROOT / "stdlib/Bindings_Generator/module.jai",
            ]
        )
        receipt["passed"] = True
    finally:
        receipt["source_unchanged"] = before == snapshot()
        if not receipt["source_unchanged"]:
            receipt["passed"] = False
        (scratch / "verification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if not receipt["source_unchanged"]:
        raise RuntimeError(
            "Shared sources changed during verification; receipt invalidated"
        )
    print(
        "Canonical public bridge/type/macro checks passed; receipt: "
        + str(scratch / "verification.json")
    )


if __name__ == "__main__":
    main()
