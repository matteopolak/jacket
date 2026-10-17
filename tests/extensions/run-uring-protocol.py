#!/usr/bin/env python3
"""Host userspace protocol and Linux source checks; never claim kernel execution."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from repo_output import output_dir


def main():
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=output_dir("extensions-dependencies", "uring-protocol"))
    parser.add_argument("--compiler", type=Path, default=root / "target/debug/jaic")
    parser.add_argument("--linter", type=Path, default=root / "target/debug/jailint")
    parser.add_argument("--formatter", type=Path, default=root / "target/jaifmt")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    compiler = str(args.compiler.resolve(strict=True))
    linter = str(args.linter.resolve(strict=True))
    formatter = str(args.formatter.resolve(strict=True))
    env = dict(os.environ, JAIC_STDLIB=str(root / "stdlib"))
    results = []

    def run(name, command, expected=None):
        try:
            result = subprocess.run(command, cwd=root, env=env, text=True,
                                    capture_output=True, timeout=90)
            passed = result.returncode == 0 and (expected is None or result.stdout == expected)
            record = dict(name=name, command=command, passed=passed, code=result.returncode,
                          stdout=result.stdout, stderr=result.stderr)
        except (OSError, subprocess.TimeoutExpired) as error:
            record = dict(name=name, command=command, passed=False, error=str(error))
        results.append(record)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        if not record["passed"]:
            print(json.dumps(record, indent=2))
        return record["passed"]

    source = "tests/extensions/io-uring-protocol.jai"
    run("host-interp", [compiler, "-interp", source], "ok\n")
    for mode in ["O0", "O2"]:
        exe = str(output / f"protocol-{mode}")
        if run(f"host-{mode}-build", [compiler, source, f"-{mode}", "-exe", exe]):
            run(f"host-{mode}", [exe], "ok\n")

    for cpu in ["x64", "arm64"]:
        for name in ["io-uring-dependency-repro", "io-uring-raw"]:
            run(f"linux-{cpu}-{name}",
                [compiler, "-check", "-os", "linux", "-cpu", cpu, f"tests/extensions/{name}.jai"])
        run(f"linux-{cpu}-strictlint", [linter, "-D", "warnings", "--os", "linux", "--cpu", cpu,
                                      "--context", "tests/extensions/io-uring-raw.jai", "stdlib/Extensions/Io_Uring"])
    run("host-strictlint", [linter, "-D", "warnings", source])
    run("format", [formatter, "--check", "stdlib/Extensions/Io_Uring",
                   "tests/extensions/io-uring-dependency-repro.jai", source])
    failures = sum(not record["passed"] for record in results)
    print(f"{len(results) - failures}/{len(results)} checks passed; {failures} failures; no Linux kernel execution")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
