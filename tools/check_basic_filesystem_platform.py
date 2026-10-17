"""Check public filesystem availability and a real runtime-provider dependency.

Uses an explicitly supplied source-built clean compiler, never a reference tool.
Every invocation is -check; this driver executes no generated native program.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--targets", nargs="+", default=["macos", "linux", "windows"])
    args = parser.parse_args()
    fixture = Path(__file__).resolve().parents[1] / "stdlib/Basic/tests/driven/filesystem-platform"
    compiler = args.compiler.resolve()
    stdlib = args.stdlib.resolve()
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ, JAIC_STDLIB=str(stdlib), TMPDIR=str(scratch))
    records = []

    def check(label, source, expected=0, extra=(), messages=()):
        argv = [str(compiler), str(source), "-check", *extra]
        result = subprocess.run(argv, env=environment, capture_output=True, text=True)
        records.append(
            dict(
                label=label,
                argv=argv,
                expected_exit=expected,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                expected_diagnostics=messages,
                diagnostic_matches=not messages
                or any(message in result.stderr for message in messages),
            )
        )

    for provider in ["shipped", "dependent"]:
        project = scratch / provider
        project.mkdir()
        if provider == "dependent":
            module = project / "modules/Runtime_Support/module.jai"
            module.parent.mkdir(parents=True)
            text = (fixture / "provider.jai.in").read_text()
            text = text.replace(
                '"@CANONICAL_RUNTIME_ENTRY@"',
                json.dumps(str(stdlib / "Runtime_Support/module.jai")),
            )
            module.write_text(text)
        for name in ["main.jai", "static.jai"]:
            source = project / name
            source.write_bytes((fixture / name).read_bytes())
            check(f"{provider}-{name}", source)

    for target in args.targets:
        check(f"target-{target}", scratch / "shipped/static.jai", extra=["-os", target])
    for name in [
        "get_working_directory",
        "set_working_directory",
        "sleep_milliseconds",
        "exit",
    ]:
        source = scratch / f"wasm-{name}.jai"
        source.write_text(
            f'Basic :: #import "Basic"; #assert is_constant(Basic.{name}); main :: () {{}}\n'
        )
        check(
            f"wasm-{name}",
            source,
            expected=1,
            extra=["-os", "wasm"],
            messages=[f"has no exported member `{name}`"],
        )
    check(
        "true-cycle",
        fixture / "true-cycle/main.jai",
        expected=1,
        messages=["circular dependency", "has no exported member `api`"],
    )
    passed = all(
        row["exit"] == row["expected_exit"] and row["diagnostic_matches"]
        for row in records
    )
    report = dict(
        records=records,
        passed=passed,
        checked_positive_targets=args.targets,
        android_native_proof=False,
        native_programs_executed=0,
    )
    (scratch / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(dict(passed=passed, checks=len(records))))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
