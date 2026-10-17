#!/usr/bin/env python3
"""Narrow display-free native clock failure proof; no shared compiler rebuild."""

import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    base = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    work = Path(tempfile.mkdtemp(prefix="clipboard-clock-", dir=base))
    names = {
        "stdlib/Clipboard/tests/native/clock-failure.c",
        "tools/check_portable_clipboard_clock.py",
    }

    def hashes():
        return {
            name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
            for name in sorted(names)
        }

    before = hashes()
    rows = []

    def run(name, command, diagnostic=None):
        result = subprocess.run(
            list(map(str, command)),
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=30,
            env=dict(os.environ, TMPDIR=str(work)),
        )
        passed = result.returncode == 0 and (
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
        print(name, "PASS" if passed else "FAIL", flush=True)
        return passed

    native = ROOT / "stdlib/internal/clipboard/native"
    test = ROOT / "stdlib/Clipboard/tests/native/clock-failure.c"
    sdk = (
        ["-I/opt/homebrew/include", "-L/opt/homebrew/lib"]
        if platform.system() == "Darwin"
        else []
    )
    for optimization in ["-O0", "-O2"]:
        program = work / ("clock" + optimization)
        if run(
            "clock-build:" + optimization,
            [
                "clang",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pthread",
                optimization,
                "-fsanitize=address,undefined",
                *sdk,
                test,
                native / "data.c",
                "-lX11",
                "-o",
                program,
            ],
        ):
            run(
                "clock-failure-execute:" + optimization,
                [program],
                diagnostic="X11 clipboard monotonic clock failed: Input/output error (errno=5)",
            )
    include = ["-I/opt/homebrew/include"] if platform.system() == "Darwin" else []
    run(
        "production-sdk-syntax",
        [
            "clang",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pthread",
            *include,
            "-fsyntax-only",
            native / "x11.c",
            native / "data.c",
        ],
    )
    run("c-format", ["clang-format", "--dry-run", "--Werror", native / "x11.c", test])
    run("python-lint", ["ruff", "check", Path(__file__)])
    run("python-format", ["ruff", "format", "--check", Path(__file__)])
    after = hashes()
    drift = sorted(name for name in before if before[name] != after[name])
    report = dict(
        inputs=before,
        source_drift=drift,
        commands=rows,
        passed=not drift and all(row["passed"] for row in rows),
        scope="Native clock/queue/cleanup unit proof; no Display or clipboard operation",
    )
    (work / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "summary.json", flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
