#!/usr/bin/env python3
"""Guarded Clipboard source/SDK/core proof; Linux display execution is opt-in."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile

from clipboard_linux_diagnostics import failure_diagnostic, report_failure

ROOT = Path(__file__).resolve().parents[1]
JAI = [
    "stdlib/Clipboard/module.jai",
    "stdlib/internal/clipboard/linux.jai",
    "stdlib/internal/window/connection.jai",
    "stdlib/Clipboard/tests/linux-api.jai",
    "stdlib/Clipboard/tests/linux-roundtrip.jai",
    "stdlib/Clipboard/tests/linux-no-empty.jai",
    "stdlib/Clipboard/tests/linux-private.jai",
    "stdlib/Clipboard/tests/platform-api.jai",
    "stdlib/Clipboard/tests/linux-window-source.jai",
]
C = [
    "stdlib/internal/clipboard/native/" + name for name in ["x11.c", "data.c", "data.h"]
]
TEST_C = [
    "stdlib/Clipboard/tests/native/data.c",
    "stdlib/Clipboard/tests/native/x11.c",
    "stdlib/Clipboard/tests/native/clock-failure.c",
]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs():
    paths = {ROOT / "Cargo.toml", ROOT / "Cargo.lock"}
    paths.update(
        p
        for p in (ROOT / "crates").rglob("*")
        if p.is_file() and (p.suffix in {".rs", ".c", ".h"} or p.name == "Cargo.toml")
    )
    for directory in ["stdlib", "jaifmt"]:
        paths.update(
            p for p in (ROOT / directory).rglob("*.jai") if not p.name.startswith("._")
        )
    paths.update(
        ROOT / p
        for p in JAI
        + C
        + TEST_C
        + [
            "tools/check_portable_clipboard.py",
            "tools/build_portable_clipboard.py",
            "tools/check_portable_clipboard_clock.py",
            "tools/clipboard_linux_diagnostics.py",
            "tools/test_clipboard_linux_diagnostics.py",
            "docs/specs/stdlib/modules/clipboard.md",
        ]
    )
    return {str(p.relative_to(ROOT)): digest(p) for p in sorted(paths)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--linux-runtime",
        action="store_true",
        help="authorized normal Linux CI display: modifies its clipboard",
    )
    args = parser.parse_args()
    if args.linux_runtime and platform.system() != "Linux":
        parser.error("Linux native execution requires an actual Linux host/display")
    base = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    work = Path(tempfile.mkdtemp(prefix="clipboard-proof-", dir=base))
    target = work / "tools"
    temporary = work / "tmp"
    temporary.mkdir()
    env = dict(
        os.environ,
        CARGO_TARGET_DIR=str(target),
        TMPDIR=str(temporary),
        JAIC_STDLIB=str(ROOT / "stdlib"),
    )
    source = inputs()
    rows = []

    def run(name, command, status=0, diagnostic=None):
        result = subprocess.run(
            list(map(str, command)),
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=240,
        )
        passed = result.returncode == status and (
            diagnostic is None or diagnostic in result.stderr
        )
        rows.append(
            dict(
                name=name,
                command=list(map(str, command)),
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        )
        (work / "commands.json").write_text(json.dumps(rows, indent=2) + "\n")
        print(name, "PASS" if passed else "FAIL", flush=True)
        if not passed:
            failures = [
                dict(
                    name=row["name"],
                    exit=row["exit"],
                    diagnostic=failure_diagnostic(row),
                )
                for row in rows
                if not row["passed"]
            ]
            (work / "failure-diagnostics.json").write_text(
                json.dumps(failures, indent=2) + "\n"
            )
            report_failure(rows[-1], work / "commands.json")
        return passed

    built = run(
        "fresh-tools",
        ["cargo", "build", "--locked", "--offline", "-p", "jaic-cli", "-p", "jailint"],
    )
    compiler = target / "debug/jaic"
    lint = target / "debug/jailint"
    formatter = work / "jaifmt"
    if built:
        formatted = run(
            "formatter-build",
            [compiler, ROOT / "jaifmt/main.jai", "-O2", "-exe", formatter],
        )
        if formatted:
            run("jai-format", [formatter, "--check", *[ROOT / p for p in JAI]])
        for cpu in ["x64", "arm64"]:
            for fixture in [
                "linux-api.jai",
                "linux-roundtrip.jai",
                "linux-window-source.jai",
            ]:
                run(
                    "linux-source:" + cpu + ":" + fixture,
                    [
                        compiler,
                        ROOT / "stdlib/Clipboard/tests" / fixture,
                        "-check",
                        "-no_dce",
                        "-os",
                        "linux",
                        "-cpu",
                        cpu,
                    ],
                )
            for fixture, name in [
                ("linux-no-empty.jai", "os_clipboard_empty"),
                ("linux-private.jai", "native_read"),
            ]:
                run(
                    "linux-negative:" + cpu + ":" + fixture,
                    [
                        compiler,
                        ROOT / "stdlib/Clipboard/tests" / fixture,
                        "-check",
                        "-os",
                        "linux",
                        "-cpu",
                        cpu,
                    ],
                    status=1,
                    diagnostic=name,
                )
            for system in ["linux", "macos", "windows"]:
                fixture = ROOT / "stdlib/Clipboard/tests/platform-api.jai"
                run(
                    "platform-source:" + system + ":" + cpu,
                    [
                        compiler,
                        fixture,
                        "-check",
                        "-no_dce",
                        "-os",
                        system,
                        "-cpu",
                        cpu,
                    ],
                )
                backend = {"linux": "linux", "macos": "cocoa", "windows": "win32"}[
                    system
                ]
                run(
                    "strict-lint:" + system + ":" + cpu,
                    [
                        lint,
                        "-D",
                        "warnings",
                        "-os",
                        system,
                        "-cpu",
                        cpu,
                        "--context",
                        fixture,
                        "--manifest",
                        work / ("lint-" + system + "-" + cpu + ".json"),
                        ROOT / JAI[0],
                        ROOT / ("stdlib/internal/clipboard/" + backend + ".jai"),
                    ],
                )
            run(
                "connection-strict-lint:" + cpu,
                [
                    lint,
                    "-D",
                    "warnings",
                    "-os",
                    "linux",
                    "-cpu",
                    cpu,
                    "--context",
                    ROOT / "stdlib/Clipboard/tests/linux-window-source.jai",
                    "--manifest",
                    work / ("lint-connection-" + cpu + ".json"),
                    ROOT / JAI[2],
                ],
            )
    include = ["-I/opt/homebrew/include"] if platform.system() == "Darwin" else []
    run(
        "native-x11-sdk-source",
        [
            "clang",
            "-std=c11",
            "-pthread",
            "-Wall",
            "-Wextra",
            "-Werror",
            *include,
            "-fsyntax-only",
            *[ROOT / p for p in C if p.endswith(".c")],
            ROOT / TEST_C[1],
        ],
    )
    for optimization in ["-O0", "-O2"]:
        executable = work / ("data" + optimization)
        run(
            "data-build:" + optimization,
            [
                "clang",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                optimization,
                "-fsanitize=address,undefined",
                ROOT / TEST_C[0],
                ROOT / C[1],
                "-o",
                executable,
            ],
        )
        if executable.exists():
            run("data-execute:" + optimization, [executable])
    run(
        "python-lint",
        [
            "ruff",
            "check",
            ROOT / "tools/check_portable_clipboard.py",
            ROOT / "tools/build_portable_clipboard.py",
            ROOT / "tools/clipboard_linux_diagnostics.py",
            ROOT / "tools/test_clipboard_linux_diagnostics.py",
        ],
    )
    run(
        "python-format",
        [
            "ruff",
            "format",
            "--check",
            ROOT / "tools/check_portable_clipboard.py",
            ROOT / "tools/build_portable_clipboard.py",
            ROOT / "tools/clipboard_linux_diagnostics.py",
            ROOT / "tools/test_clipboard_linux_diagnostics.py",
        ],
    )
    run(
        "c-format",
        ["clang-format", "--dry-run", "--Werror", *[ROOT / p for p in C + TEST_C]],
    )
    run(
        "clock-failure-core",
        [sys.executable, ROOT / "tools/check_portable_clipboard_clock.py"],
    )
    if args.linux_runtime and built:
        native = work / "native"
        if run(
            "linux-helper-build",
            [
                sys.executable,
                ROOT / "tools/build_portable_clipboard.py",
                "--output",
                native,
            ],
        ):
            env["JAIC_NATIVE_LIBS"] = str(native)
            env["LD_LIBRARY_PATH"] = str(native) + (
                ":" + env["LD_LIBRARY_PATH"] if env.get("LD_LIBRARY_PATH") else ""
            )
            run(
                "linux-interpreter",
                [
                    compiler,
                    ROOT / "stdlib/Clipboard/tests/linux-roundtrip.jai",
                    "-interp",
                ],
            )
            for optimization in ["-O0", "-O2"]:
                program = work / ("roundtrip" + optimization)
                if run(
                    "linux-jai-build:" + optimization,
                    [
                        compiler,
                        ROOT / "stdlib/Clipboard/tests/linux-roundtrip.jai",
                        optimization,
                        "-exe",
                        program,
                    ],
                ):
                    run("linux-jai-execute:" + optimization, [program])
                program = work / ("x11-clients" + optimization)
                if run(
                    "linux-clients-build:" + optimization,
                    [
                        "cc",
                        "-std=c11",
                        "-Wall",
                        "-Wextra",
                        "-Werror",
                        optimization,
                        ROOT / TEST_C[1],
                        native / "libportable_clipboard.a",
                        "-lX11",
                        "-pthread",
                        "-o",
                        program,
                    ],
                ):
                    run("linux-clients-execute:" + optimization, [program])
    final = inputs()
    drift = sorted(
        p for p in source.keys() | final.keys() if source.get(p) != final.get(p)
    )
    receipt = dict(
        source_inputs=source,
        commands=rows,
        source_drift=drift,
        compiler_sha256=digest(compiler) if compiler.exists() else None,
        linux_runtime_executed=args.linux_runtime,
        passed=not drift and all(row["passed"] for row in rows),
    )
    (work / "summary.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(work / "summary.json", flush=True)
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
