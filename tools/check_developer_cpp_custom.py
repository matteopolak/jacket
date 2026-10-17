#!/usr/bin/env python3
"""Targeted original C++ customization and C-model regression effects on the host."""

import os
from pathlib import Path
import platform
import subprocess
import build_developer_bindings
import build_generator_platform
from driven_sources import driven

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, expected=""):
    result = subprocess.run(
        [str(item) for item in command],
        cwd=ROOT,
        env=ENV,
        text=True,
        capture_output=True,
        timeout=90,
    )
    if result.returncode or result.stderr or result.stdout != expected:
        diagnostic = (result.stderr or result.stdout).splitlines()[:2]
        raise RuntimeError(
            f"Targeted C++ workflow failed: {command}; status={result.returncode}; diagnostic={diagnostic}"
        )


def prepare_callback_input(prefix, flags, suffix):
    """A real C pointer parameter for the ordinary Jai wrapper callback control."""
    target = ROOT / "target"
    (target / "developer-tools-cpp-callback.h").write_text(
        "int original_callback_value(const int *value);\n"
    )
    source = target / "developer-tools-cpp-callback.c"
    source.write_text(
        '#include "developer-tools-cpp-callback.h"\n'
        "int original_callback_value(const int *value) { return *value * 10 + 3; }\n"
    )
    run(
        [
            prefix / "bin/clang",
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            *flags,
            source,
            "-o",
            target / f"libdeveloper_tools_cpp_callback.{suffix}",
        ]
    )


def reject_wrong_adapter_schema(consumer):
    """Keep the original aggregate ABI mismatch as a source-only negative."""
    original = consumer.read_text()
    import_line = 'G :: #import "developer-tools-cpp-custom-bridge";'
    if original.count(import_line) != 1:
        raise RuntimeError("Required bridge consumer import is missing or ambiguous")
    negative = ROOT / "target/developer-tools-cpp-wrong-abi.jai"
    negative.write_text(
        original.replace(
            import_line, 'G :: #import "developer-tools-cpp-custom-generated";'
        )
    )
    result = subprocess.run(
        [str(JAIC), str(negative), "-check", "-I", str(ROOT / "target")],
        cwd=ROOT,
        env=ENV,
        text=True,
        capture_output=True,
        timeout=90,
    )
    if (
        result.returncode == 0
        or "make_custom_record" not in result.stderr
        or "parameter `value` given twice" not in result.stderr
    ):
        raise RuntimeError(
            f"Wrong adapter schema did not produce its required ABI rejection: {result.stderr}"
        )


def check_cpp_custom():
    prefix = build_developer_bindings.llvm_prefix()
    # The public generator executes this real host provider on every native OS.
    # Its preparation must not depend on the Darwin-only object oracle suite.
    build_generator_platform.build(prefix)
    emitter = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom-roundtrip.jai"
    generated_schemas = [
        ROOT / "target" / f"developer-tools-cpp-{name}.jai"
        for name in ["custom-generated", "custom-bridge", "callback-generated"]
    ]
    flags = build_developer_bindings.shared_flags()
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    prepare_callback_input(prefix, flags, suffix)
    # The generator must inspect the real C++ exports before an adapter exists.
    # Build the original input first; the bridge build below replaces it only
    # after generation has completed in its separate process.
    run(
        [
            prefix / "bin/clang++",
            "-std=c++17",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            *flags,
            ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom-native.cpp",
            "-o",
            ROOT / f"target/libdeveloper_tools_cpp_custom.{suffix}",
        ]
    )
    for name, source in [
        ("bindings", "input"),
        ("properties", "model-properties"),
        ("macros", "macro-values"),
        ("merge", "macro-merge"),
    ]:
        run(
            [
                prefix / "bin/clang",
                "-std=c11",
                "-O2",
                "-Wall",
                "-Wextra",
                "-Werror",
                *flags,
                driven(f"bindings-{source}.c"),
                "-o",
                ROOT / f"target/libdeveloper_tools_{name}.{suffix}",
            ]
        )
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([JAIC, emitter, "-interp"], "ok\n")
        else:
            executable = ROOT / f"target/developer-tools-cpp-custom-emitter-o{mode}"
            run([JAIC, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        for generated in generated_schemas:
            run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
            run(
                [
                    ROOT / "target/jaifmt",
                    "--config",
                    ROOT / "jaifmt.toml",
                    "--check",
                    generated,
                ]
            )
            run([ROOT / "target/debug/jailint", "-D", "warnings", generated])
        reject_wrong_adapter_schema(consumer)
        for optimization in [0, 2]:
            run(
                [
                    prefix / "bin/clang++",
                    "-std=c++17",
                    f"-O{optimization}",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    *flags,
                    ROOT / "target/developer-tools-cpp-custom.cpp",
                    ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom-native.cpp",
                    "-o",
                    ROOT / f"target/libdeveloper_tools_cpp_custom.{suffix}",
                ]
            )
            run([JAIC, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
            executable = (
                ROOT
                / f"target/developer-tools-cpp-custom-consumer-{mode}-o{optimization}"
            )
            run(
                [
                    JAIC,
                    consumer,
                    f"-O{optimization}",
                    "-exe",
                    executable,
                    "-I",
                    ROOT / "target",
                ]
            )
            run([executable], "ok\n")
    print(
        "Customized C++ overload/POD/reference and real argument-wrapper callbacks passed interpreter/native host O0/O2",
        flush=True,
    )


def check_c_model_regressions():
    # Same model/parser changed: retain real C callback, macro and type effects.
    for name in [
        "bindings-custom",
        "bindings-model-properties",
        "bindings-macro-custom",
        "bindings-macro-enums",
        "bindings-macro-merge",
    ]:
        source = driven(f"{name}.jai")
        if not source.exists():
            raise RuntimeError(
                f"Required existing model regression fixture is absent: {source}"
            )
        run([JAIC, source, "-interp", "-I", ROOT / "target"], "ok\n")
        for optimization in [0, 2]:
            executable = (
                ROOT / f"target/developer-tools-cpp-regression-{name}-o{optimization}"
            )
            run(
                [
                    JAIC,
                    source,
                    f"-O{optimization}",
                    "-exe",
                    executable,
                    "-I",
                    ROOT / "target",
                ]
            )
            run([executable], "ok\n")
    print(
        "Required C-model regressions passed interpreter/native host O0/O2",
        flush=True,
    )


def main():
    check_cpp_custom()
    check_c_model_regressions()


if __name__ == "__main__":
    main()
