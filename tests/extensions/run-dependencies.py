#!/usr/bin/env python3
"""Focused production-dependency checks; no reference inputs or skipped failures."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, default=ROOT / "target/debug/jaic")
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/extensions-dependencies")
    args = parser.parse_args()
    compiler = str(args.compiler.resolve())
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    env.pop("EXTENSIONS_ARGS_COUNT", None)
    results = []

    def run(name, command, *, code=0, stdout=None, stderr=None, contains=None, absent=None, extra_env=None, stdin=None):
        try:
            process = subprocess.run(command, cwd=ROOT, env=env | (extra_env or {}),
                                     input=stdin, capture_output=True, text=True, timeout=90)
            passed = process.returncode != 0 if code is None else process.returncode == code
            if stdout is not None:
                passed &= process.stdout == stdout
            if stderr is not None:
                passed &= process.stderr == stderr
            if contains is not None:
                passed &= contains in process.stdout + process.stderr
            if absent is not None:
                passed &= absent not in process.stdout + process.stderr
            record = dict(name=name, command=command, passed=passed,
                          code=process.returncode, stdout=process.stdout, stderr=process.stderr)
        except (OSError, subprocess.TimeoutExpired) as error:
            record = dict(name=name, command=command, passed=False, error=str(error))
        results.append(record)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        if not record["passed"]:
            print(json.dumps(record, indent=2), flush=True)
        return record["passed"]

    fibers = ["io-fibers-context", "io-fibers-many", "io-fibers-channels",
              "io-fibers-sleep-cancel", "io-allocator-ownership", "io-allocator-zero", "io-dependency-workers"]
    args_suites = [f"stdlib/Extensions/Args/tests/{name}.jai"
                  for name in ["parse", "help", "completions"]]
    sources = [f"tests/extensions/{name}.jai" for name in fibers] + args_suites
    for mode in ["interp", "O0", "O2"]:
        for source in sources:
            name = Path(source).stem
            extra = [str(output / f"workers-{mode}.data")] if name == "io-dependency-workers" else []
            if mode == "interp":
                command = [compiler, "-interp", source, "--", *extra]
            else:
                exe = str(output / f"{name}-{mode}")
                if not run(f"{name}-{mode}-build", [compiler, source, f"-{mode}", "-exe", exe]):
                    continue
                command = [exe, *extra]
            run(f"{name}-{mode}", command, stdout="ok\n", extra_env={"JAI_IO_FILE_INLINE": "0"})

        source = "tests/native/fibers/overflow.jai"
        if mode == "interp":
            run("overflow-interp", [compiler, "-interp", source], code=None, contains="stack overflow")
        else:
            exe = str(output / f"overflow-{mode}")
            if run(f"overflow-{mode}-build", [compiler, source, f"-{mode}", "-exe", exe]):
                run(f"overflow-{mode}", [exe], code=None, contains="fiber stack overflow")

    source = "tests/extensions/args-process.jai"
    cases = [([], 0, "count=7 set=false\n", None, {}),
             (["--count", "8"], 0, "count=8 set=true\n", None, {}),
             ([], 0, "count=9 set=true\n", None, {"EXTENSIONS_ARGS_COUNT": "9"}),
             (["--count", "8"], 0, "count=8 set=true\n", None, {"EXTENSIONS_ARGS_COUNT": "9"}),
             (["--version"], 0, None, "1.0", {}),
             (["--help"], 0, None, "Extensions process contract", {}),
             (["--unknown"], 2, "", "unknown option", {}),
             (["--count", "21"], 2, "", "out of range (1..20)", {}),
             (["--count"], 2, "", "value", {})]
    for mode in ["interp", "O0", "O2", "wasm-O0", "wasm-O2"]:
        if mode == "interp":
            command = [compiler, "-interp", source, "--"]
        else:
            exe = str(output / (f"args-{mode}.wasm" if mode.startswith("wasm") else f"args-{mode}"))
            flags = [f"-{mode.split('-')[-1]}"]
            if mode.startswith("wasm"):
                flags += ["-os", "wasm"]
            if not run(f"args-{mode}-build", [compiler, source, *flags, "-exe", exe]):
                continue
            command = ["node", "--no-warnings", str(ROOT / "tools/wasi_run.mjs"), exe] if mode.startswith("wasm") else [exe]
        for index, (argv, code, expected, needle, variables) in enumerate(cases):
            run(f"args-{mode}-{index}", command + argv, code=code, stdout=expected,
                stderr="" if code == 0 else None, contains=needle,
                absent="count=" if argv in [["--help"], ["--version"]] else None,
                extra_env=variables)

    for os_name in ["linux", "windows", "macos"]:
        for cpu in ["x64", "arm64"]:
            for name in ["args-process", "io-dependency-workers", "io-allocator-ownership"]:
                run(f"check-{name}-{os_name}-{cpu}",
                    [compiler, f"tests/extensions/{name}.jai", "-check", "-os", os_name, "-cpu", cpu])

    failures = sum(not result["passed"] for result in results)
    print(f"{len(results) - failures}/{len(results)} checks passed; {failures} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
