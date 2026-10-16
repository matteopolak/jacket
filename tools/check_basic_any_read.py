#!/usr/bin/env python3
"""Run the official-derived Any formatting boundary controls in a leased slot."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from check_basic_null_format import ROOT, hashes


def expected_case_result(result, fault, stdout, diagnostic):
    if result.stdout != stdout:
        return False
    if fault:
        return result.returncode != 0 and diagnostic in result.stderr
    return result.returncode == 0 and not result.stderr


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--case", help="Run only the named independent control")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    temporary = args.output / "tmp"
    temporary.mkdir()
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), TMPDIR=str(temporary))
    before = hashes()
    rows = []

    def command(name, argv, expected=None):
        result = subprocess.run(
            list(map(str, argv)),
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=60,
        )
        passed = result.returncode == 0
        if expected is not None:
            passed = expected_case_result(result, *expected)
        rows.append(
            dict(
                name=name,
                argv=list(map(str, argv)),
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        )
        print(name, "PASS" if passed else "FAIL", flush=True)
        (args.output / "commands.json").write_text(json.dumps(rows, indent=2) + "\n")
        return passed

    cases = [
        ("format-null-any-contract", False, "continued\n", None),
        (
            "format-any-missing-value",
            True,
            "",
            "cannot format an Any without payload storage",
        ),
        (
            "format-any-missing-type",
            True,
            "",
            "cannot format an Any without type metadata",
        ),
        (
            "format-any-missing-value-no-assert",
            True,
            "",
            "cannot format an Any without payload storage",
        ),
        (
            "format-null-lvalue-any-probe",
            True,
            "7\ntrue\n",
            "cannot format an Any without payload storage",
        ),
    ]
    if args.case and args.case not in {case[0] for case in cases}:
        parser.error("unknown case")
    for name, fault, stdout, diagnostic in cases:
        if args.case and args.case != name:
            continue
        source = ROOT / "tests/portable-stdlib" / (name + ".jai")
        for mode in ["interp", "O0", "O2"]:
            if mode == "interp":
                command(
                    name + ":interp",
                    [args.compiler, source, "-interp"],
                    (fault, stdout, diagnostic),
                )
            else:
                exe = args.output / (name + "-" + mode)
                if command(
                    name + ":build:" + mode,
                    [args.compiler, source, "-" + mode, "-exe", exe],
                ):
                    command(
                        name + ":native:" + mode, [exe], (fault, stdout, diagnostic)
                    )
    after = hashes()
    drift = sorted(
        k for k in before.keys() | after.keys() if before.get(k) != after.get(k)
    )
    report = dict(
        inputs=before,
        source_drift=drift,
        compiler_sha256=hashlib.sha256(args.compiler.read_bytes()).hexdigest(),
        commands=rows,
        passed=not drift and all(row["passed"] for row in rows),
        oracle="Root sanitized official native public probes; partial-null Any faults; no eager boxing read",
    )
    (args.output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
