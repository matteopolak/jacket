#!/usr/bin/env python3
"""Focused, source-matched Math matrix operator verification; all output on T7."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
TARGET = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
T7 = TARGET
JAIC = TARGET / "debug/jaic"
LINT = TARGET / "debug/jailint"
# Byte-identical NEW independent controls from c22aea92 are local fixtures.
PEER = HERE


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hashes(paths):
    return {str(p.relative_to(ROOT)): sha(p) for p in paths}


def main():
    out = Path(tempfile.mkdtemp(prefix="matrix-operator-proof-", dir=T7))
    report = out / "results.json"
    evidence = {"complete": False, "commands": []}
    env = dict(
        os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), CARGO_TARGET_DIR=str(TARGET)
    )

    def save():
        report.write_text(json.dumps(evidence, indent=2) + "\n")

    def run(command, expected="", rejection=None, cwd=out):
        result = subprocess.run(
            [str(x) for x in command],
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
            timeout=120,
        )
        passed = (
            (result.returncode != 0 and rejection in result.stderr)
            if rejection
            else (
                result.returncode == 0
                and result.stdout == expected
                and result.stderr == ""
            )
        )
        record = {
            "command": [str(x) for x in command],
            "exit_code": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "passed": passed,
            "required_rejection": rejection,
        }
        evidence["commands"].append(record)
        save()
        if not passed:
            raise RuntimeError(json.dumps(record, indent=2))

    compiler_sources = sorted((ROOT / "crates").rglob("*.rs")) + sorted(
        (ROOT / "crates").rglob("Cargo.toml")
    )
    compiler_sources += [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    evidence["compiler_source_before"] = hashes(compiler_sources)
    command = [
        "cargo",
        "build",
        "--locked",
        "--offline",
        "-p",
        "jaic-cli",
        "-p",
        "jailint",
    ]
    result = subprocess.run(
        command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=120
    )
    evidence["source_build"] = {
        "command": command,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }
    save()
    if result.returncode:
        raise RuntimeError("Source compiler build failed")
    assert evidence["compiler_source_before"] == hashes(compiler_sources)
    evidence["tools"] = {str(p): sha(p) for p in [JAIC, LINT]}

    formatter = out / "jaifmt"
    run(
        [
            JAIC,
            ROOT / "jaifmt/main.jai",
            "-O2",
            "-no_backtrace_on_crash",
            "-exe",
            formatter,
        ]
    )
    owned = [
        ROOT / "stdlib/internal/numerics/math_matrix.jai",
        *sorted(HERE.glob("*.jai")),
    ]
    run([formatter, "--config", ROOT / "jaifmt.toml", *owned])
    run([formatter, "--config", ROOT / "jaifmt.toml", "--check", *owned])
    watched = [
        ROOT / "stdlib/Math/module.jai",
        ROOT / "stdlib/internal/numerics/math_matrix.jai",
        ROOT / "stdlib/internal/numerics/math_vectors.jai",
        ROOT / "stdlib/internal/numerics/math_types.jai",
        ROOT / "stdlib/internal/numerics/math/module.jai",
    ]
    evidence["source_before"] = hashes(watched)

    cases = [
        (HERE / name, "ok\n")
        for name in ["matrix2-vector2.jai", "family.jai", "existing-overloads.jai"]
    ]
    cases += [
        (PEER / name, "")
        for name in [
            "matrix-vector-operator-oracle.jai",
            "matrix-vector-multiply-control.jai",
        ]
    ]
    evidence["fixture_sha256"] = {str(p): sha(p) for p, _ in cases}
    for index, (path, expected) in enumerate(cases):
        run([JAIC, path, "-interp"], expected=expected)
        for level in [0, 2]:
            exe = out / ("case-" + str(index) + "-o" + str(level))
            run([JAIC, path, "-O" + str(level), "-no_backtrace_on_crash", "-exe", exe])
            run([exe], expected=expected)
    for name in ["reverse-vector-negative.jai", "mixed-add-negative.jai"]:
        run([JAIC, HERE / name, "-check"], rejection="no overload of")
    run(
        [
            LINT,
            "-D",
            "warnings",
            ROOT / "stdlib/Math/module.jai",
            *[p for p, _ in cases if p.parent == HERE],
        ]
    )
    evidence["source_after"] = hashes(watched)
    evidence["compiler_source_after"] = hashes(compiler_sources)
    assert evidence["source_before"] == evidence["source_after"]
    assert evidence["compiler_source_before"] == evidence["compiler_source_after"]
    evidence["complete"] = True
    save()
    print(report)


if __name__ == "__main__":
    main()
