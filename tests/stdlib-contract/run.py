#!/usr/bin/env python3
"""Run public stdlib contract fixtures against an explicitly selected local jaic."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
SUITE = Path(__file__).resolve().parent


def checked_path(raw: str, label: str) -> Path:
    path = Path(raw).expanduser()
    if not path.is_absolute():
        raise SystemExit(f"--{label} must be an absolute path")
    return path.resolve(strict=True)


def case_passes(case, mode, code, output, error):
    if mode == "check-fail":
        diagnostic = case.get("diagnostic_contains")
        if not isinstance(diagnostic, str) or not diagnostic.strip():
            raise ValueError("check-fail requires a precise diagnostic_contains")
        return code != 0 and diagnostic in output + error
    if "error_contains" in case:
        return code == case["exit_code"] and case["error_contains"] in error and output == case["stdout"]
    if mode == "check":
        return code == 0
    return code == 0 and output == case["stdout"]


def selected_cases(cases, names):
    by_name = {case["name"]: case for case in cases}
    if len(by_name) != len(cases):
        raise ValueError("duplicate manifest case names")
    chosen = set(names) if names else set(by_name)
    pending = list(chosen)
    while pending:
        name = pending.pop()
        if name not in by_name:
            raise ValueError("unknown case name: " + name)
        baseline = by_name[name].get("positive_baseline")
        if baseline:
            if (baseline not in by_name or by_name[baseline]["mode"] == "check-fail"
                    or "error_contains" in by_name[baseline]
                    or "positive_baseline" in by_name[baseline]):
                raise ValueError("positive_baseline must name a positive case: " + baseline)
            if baseline not in chosen:
                chosen.add(baseline)
                pending.append(baseline)
    ordered = []
    emitted = set()
    for case in cases:
        if case["name"] not in chosen:
            continue
        baseline = case.get("positive_baseline")
        if baseline and baseline not in emitted:
            ordered.append(by_name[baseline])
            emitted.add(baseline)
        if case["name"] not in emitted:
            ordered.append(case)
            emitted.add(case["name"])
    return ordered


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, help="absolute path to source-built jaic under this workspace's target/")
    parser.add_argument("--stdlib-root", required=True, help="absolute path to the public stdlib import root")
    parser.add_argument("--mode", choices=("interp", "native", "check", "check-fail"), help="override positive cases; declared negative cases retain their mode")
    parser.add_argument("--opt", choices=("O0", "O2"), default="O0", help="explicit native optimization level")
    parser.add_argument("--case", action="append", help="run only this manifest case name; repeat to select several")
    args = parser.parse_args()

    jaic = checked_path(args.jaic, "jaic")
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    if target not in jaic.parents or jaic.name not in {"jaic", "jaic.exe"}:
        raise SystemExit(f"refusing to execute a compiler outside this workspace's target directory: {jaic}")
    if not os.access(jaic, os.X_OK):
        raise SystemExit(f"compiler is not executable: {jaic}")
    stdlib = checked_path(args.stdlib_root, "stdlib-root")

    env = os.environ.copy()
    env["JAIC_STDLIB"] = str(stdlib)
    import_dir = SUITE
    failures = []
    cases = json.loads((SUITE / "manifest.json").read_text())
    try:
        cases = selected_cases(cases, args.case)
        for case in cases:
            if case["mode"] == "check-fail" or args.mode == "check-fail":
                case_passes(case, "check-fail", 0, "", "")
    except ValueError as error:
        parser.error(str(error))
    outcomes = {}

    for case in cases:
        source = (SUITE / case["source"]).resolve(strict=True)
        negative = case["mode"] == "check-fail" or "error_contains" in case
        mode = case["mode"] if negative else args.mode or case["mode"]
        timeout = case["timeout_seconds"]
        command = [str(jaic), str(source), "-import_dir", str(import_dir)]
        with tempfile.TemporaryDirectory(prefix="jai-stdlib-contract-") as temporary:
            executable = Path(temporary) / ("case.exe" if os.name == "nt" else "case")
            if mode == "interp":
                command += ["-interp", "-no_workspace_output"]
            elif mode == "native":
                command += ["-" + args.opt, "-exe", str(executable)]
            elif mode in ("check", "check-fail"):
                command += ["-check"]
            try:
                compiled = subprocess.run(command, cwd=temporary, env=env, text=True,
                                          capture_output=True, timeout=timeout)
                output = compiled.stdout
                error = compiled.stderr
                code = compiled.returncode
                if code == 0 and mode == "native":
                    ran = subprocess.run([str(executable)], cwd=temporary, env=env,
                                         text=True, capture_output=True, timeout=timeout)
                    output, error, code = ran.stdout, error + ran.stderr, ran.returncode
                passed = case_passes(case, mode, code, output, error)
                baseline = case.get("positive_baseline")
                if baseline and not outcomes.get(baseline, False):
                    passed = False
                    error += "\npositive baseline did not pass: " + baseline
                if not passed:
                    failures.append(f"{case['name']} [{mode}]: exit={code}, stdout={output!r}, stderr={error.strip()!r}")
            except subprocess.TimeoutExpired:
                failures.append(f"{case['name']} [{mode}]: exceeded {timeout}s timeout")
                passed = False
        print(("PASS" if passed else "FAIL") + f" {case['name']} [{mode}]")
        outcomes[case["name"]] = passed

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"passed {len(cases)} stdlib public-contract fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
