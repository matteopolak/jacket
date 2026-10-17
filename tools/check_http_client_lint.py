#!/usr/bin/env python3
"""Lint the Http client in the context of its tests, and keep the lint's negative controls honest.

`jailint --context TEST SOURCE` lints SOURCE as part of the program TEST. The Http client is
generic code, so it must stay clean in the context of its own tests and of the two overload
reductions, and a real error in an unused or selected generic must still be reported.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

from repo_output import output_dir

ROOT = Path(__file__).resolve().parents[1]
TESTS = "stdlib/Extensions/Http/tests"


def main():
    default_target = Path(os.environ.get("CARGO_TARGET_DIR") or ROOT / "target")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=Path, default=default_target,
                        help="cargo target directory holding debug/jaic and debug/jailint")
    parser.add_argument("--output", type=Path, default=output_dir("http-client-lint"))
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    compiler = str(args.target.resolve() / "debug/jaic")
    linter = str(args.target.resolve() / "debug/jailint")
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    results = []

    def run(name, command, code=0, diagnostic=None):
        try:
            process = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                                     text=True, timeout=90)
            passed = process.returncode == code
            if diagnostic is not None:
                passed &= diagnostic in process.stdout + process.stderr
            record = dict(name=name, command=command, passed=passed, code=process.returncode,
                          stdout=process.stdout, stderr=process.stderr)
        except (OSError, subprocess.TimeoutExpired) as error:
            record = dict(name=name, command=command, passed=False, error=str(error))
        results.append(record)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        if not record["passed"]:
            print(json.dumps(record, indent=2))
        return record["passed"]

    run("client-in-its-tests", [linter, "-D", "warnings", "--context",
        f"{TESTS}/client.jai", "stdlib/Extensions/Http/client.jai"])
    for name in ["client-overload-lint", "rejected-overload-lint"]:
        source = f"{TESTS}/{name}.jai"
        run(f"{name}-check", [compiler, "-check", source])
        run(f"{name}-interp", [compiler, "-interp", source])
        for opt in ["O0", "O2"]:
            exe = str(output / f"{name}-{opt}")
            if run(f"{name}-{opt}-build", [compiler, source, f"-{opt}", "-exe", exe]):
                run(f"{name}-{opt}", [exe])
        report = source if name.startswith("rejected") else "stdlib/Extensions/Http/client.jai"
        run(f"{name}-lint", [linter, "-D", "warnings", "--context", source, report])
    # Genuine errors must still fail after any general compiler or linter fix.
    controls = {
        "unused-concrete": 'broken :: () { #assert false "real concrete error"; }\nmain :: () {}\n',
        "selected-generic": 'chosen :: (value: $T) { #assert T == int "real selected error"; _ := value; }\nmain :: () { chosen(1.5); }\n',
    }
    for name, text in controls.items():
        source = output / f"{name}.jai"
        source.write_text(text)
        message = "real concrete error" if name == "unused-concrete" else "real selected error"
        run(f"{name}-lint", [linter, "-D", "warnings", "--context", str(source), str(source)],
            code=1, diagnostic=message)
        if name == "selected-generic":
            run(f"{name}-check", [compiler, "-check", str(source)], code=1, diagnostic=message)
    failed = sum(not result["passed"] for result in results)
    print(f"{len(results) - failed}/{len(results)} pass; {failed} failures")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
