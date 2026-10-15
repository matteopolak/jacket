#!/usr/bin/env python3
"""Focused reviewed Basic/Math gates against the authorized source-built compiler."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=root / "artifacts/core-public-verification")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    compiler = os.environ.get("JAIC", str(root / "target/debug/jaic"))
    linter = os.environ.get("JAILINT", str(root / "target/debug/jailint"))
    env = dict(os.environ, JAIC_STDLIB=str(root / "stdlib"))
    results = []

    def run(name, command):
        try:
            process = subprocess.run(command, cwd=root, env=env, capture_output=True,
                                     text=True, timeout=120)
            record = dict(name=name, command=command, code=process.returncode,
                          passed=process.returncode == 0, stdout=process.stdout, stderr=process.stderr)
        except (OSError, subprocess.TimeoutExpired) as error:
            record = dict(name=name, command=command, passed=False, error=str(error))
        results.append(record)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        if not record["passed"]:
            print(json.dumps(record, indent=2))
        return record["passed"]

    core = ["core-public-gaps", "core-context-style", "core-integer-time", "core-hash-free"]
    sources = [f"tests/portable-stdlib/{name}.jai" for name in core + ["basic-configuration", "formatter-records"]]
    sources.append("stdlib/Extensions/Json/tests/numeric_ownership.jai")
    for source in sources:
        name = Path(source).stem
        run(f"{name}-interp", [compiler, "-interp", source])
        for opt in ["O0", "O2"]:
            exe = str(output / f"{name}-{opt}")
            if run(f"{name}-{opt}-build", [compiler, source, f"-{opt}", "-exe", exe]):
                run(f"{name}-{opt}", [exe])
    for name in core:
        source = f"tests/portable-stdlib/{name}.jai"
        run(f"{name}-strictlint", [linter, "-D", "warnings", "--context", source, source])
    failures = sum(not result["passed"] for result in results)
    print(f"{len(results) - failures}/{len(results)} pass; {failures} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
