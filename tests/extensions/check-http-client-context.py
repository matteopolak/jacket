#!/usr/bin/env python3
"""Required green client audit plus independent reductions and negative controls."""
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
    parser.add_argument("--target", type=Path, default=output_dir("http-client-target"))
    parser.add_argument("--output", type=Path, default=output_dir("http-client-context"))
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    compiler = str(args.target.resolve() / "debug/jaic")
    linter = str(args.target.resolve() / "debug/jailint")
    env = dict(os.environ, JAIC_STDLIB=str(root / "stdlib"))
    results = []

    def run(name, command, code=0, diagnostic=None):
        try:
            process = subprocess.run(command, cwd=root, env=env, capture_output=True,
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

    run("required-production-context", [linter, "-D", "warnings", "--context",
        "stdlib/Extensions/Http/tests/client.jai", "stdlib/Extensions/Http/client.jai"])
    for name in ["http-client-overload-lint-repro", "rejected-overload-lint-repro"]:
        source = f"tests/extensions/{name}.jai"
        run(f"{name}-check", [compiler, "-check", source])
        run(f"{name}-interp", [compiler, "-interp", source])
        for opt in ["O0", "O2"]:
            exe = str(output / f"{name}-{opt}")
            if run(f"{name}-{opt}-build", [compiler, source, f"-{opt}", "-exe", exe]):
                run(f"{name}-{opt}", [exe])
        report = source if name.startswith("rejected") else "stdlib/Extensions/Http/client.jai"
        run(f"{name}-lint", [linter, "-D", "warnings", "--context", source, report])

    # Genuine negative controls must still fail after any general compiler fix.
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
    print(f"{len(results) - failed}/{len(results)} pass; {failed} failures; production lint must pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
