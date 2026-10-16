#!/usr/bin/env python3
"""Paired public operator-filter consumer checks; no internal-name fallback."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
FACADES = ["Basic", "Math", "RadixSort", "Bit_Array", "Soa", "Xar"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--lint", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, default=ROOT / "stdlib")
    parser.add_argument(
        "--native",
        action="store_true",
        help="use only after integrator serialized native gate releases",
    )
    parser.add_argument("--output-root", type=Path, required=True)
    args = parser.parse_args()
    args.output_root.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix="public-filter-proof-", dir=args.output_root))
    scratch = out / "tmp"
    scratch.mkdir()
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib), TMPDIR=str(scratch))
    report = out / "results.json"
    evidence = {
        "complete": False,
        "commands": [],
        "required_failures": [],
        "native_requested": args.native,
        "cwd": str(out),
        "environment": {n: env[n] for n in ["JAIC_STDLIB", "TMPDIR"]},
        "tools": {str(p): sha(p) for p in [args.compiler, args.lint]},
    }

    def save():
        report.write_text(json.dumps(evidence, indent=2) + "\n")

    def run(command, expected="", rejection=None):
        p = subprocess.run(
            [str(x) for x in command],
            cwd=out,
            env=env,
            text=True,
            capture_output=True,
            timeout=120,
        )
        valid = (
            (p.returncode != 0 and rejection in p.stderr)
            if rejection
            else (p.returncode == 0 and p.stdout == expected and p.stderr == "")
        )
        item = {
            "command": [str(x) for x in command],
            "exit_code": p.returncode,
            "stdout": p.stdout,
            "stderr": p.stderr,
            "passed": valid,
            "required_rejection": rejection,
        }
        evidence["commands"].append(item)
        if not valid:
            evidence["required_failures"].append(item)
        save()
        return valid

    watched = [args.stdlib / n / "module.jai" for n in FACADES]
    watched += [
        args.stdlib / p
        for p in [
            "internal/basic/apollo.jai",
            "internal/basic/integer128.jai",
            "internal/numerics/math_matrix.jai",
            "internal/numerics/math_vectors.jai",
            "internal/collections/radixsort.jai",
            "internal/collections/bit-array.jai",
            "internal/collections/soa.jai",
            "internal/collections/xar.jai",
        ]
    ]

    def source_hashes():
        return {str(p.relative_to(args.stdlib)): sha(p) for p in watched}

    evidence["source_before"] = source_hashes()
    names = ["basic", "math", "radixsort", "bit-array", "soa", "xar", "symbol-positive"]
    for index, name in enumerate(names):
        path = HERE / (name + ".jai")
        if not run([args.compiler, path, "-check"]):
            continue
        run([args.compiler, path, "-interp"], expected="ok\n")
        if args.native:
            for level in [0, 2]:
                exe = out / ("case-" + str(index) + "-o" + str(level))
                if run(
                    [
                        args.compiler,
                        path,
                        "-O" + str(level),
                        "-no_backtrace_on_crash",
                        "-exe",
                        exe,
                    ]
                ):
                    run([exe], expected="ok\n")
    run(
        [args.compiler, HERE / "internal-name-negative.jai", "-check"],
        rejection="operator Sub is not defined for Apollo_Time",
    )
    run(
        [
            args.lint,
            "-D",
            "warnings",
            *[args.stdlib / n / "module.jai" for n in FACADES],
            *[HERE / (name + ".jai") for name in names],
        ]
    )
    evidence["source_after"] = source_hashes()
    if evidence["source_before"] != evidence["source_after"]:
        raise RuntimeError("Source changed during paired verification")
    evidence["fixture_sha256"] = {p.name: sha(p) for p in HERE.glob("*.jai")}
    evidence["complete"] = not evidence["required_failures"]
    save()
    print(report)
    raise SystemExit(0 if evidence["complete"] else 1)


if __name__ == "__main__":
    main()
