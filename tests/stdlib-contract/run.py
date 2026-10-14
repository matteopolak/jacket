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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, help="absolute path to source-built jaic under this workspace's target/")
    parser.add_argument("--stdlib-root", required=True, help="absolute path to the public stdlib import root")
    parser.add_argument("--mode", choices=("interp", "native", "check"), help="override each positive case's manifest mode")
    parser.add_argument("--case", action="append", help="run only this manifest case name; repeat to select several")
    args = parser.parse_args()

    jaic = checked_path(args.jaic, "jaic")
    target = (ROOT / "target").resolve()
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
    if args.case:
        unknown = set(args.case) - {case["name"] for case in cases}
        if unknown:
            parser.error("unknown case names: " + ", ".join(sorted(unknown)))
        cases = [case for case in cases if case["name"] in args.case]

    for case in cases:
        source = (SUITE / case["source"]).resolve(strict=True)
        mode = args.mode or case["mode"]
        timeout = case["timeout_seconds"]
        command = [str(jaic), str(source), "-import_dir", str(import_dir)]
        with tempfile.TemporaryDirectory(prefix="jai-stdlib-contract-") as temporary:
            executable = Path(temporary) / ("case.exe" if os.name == "nt" else "case")
            if mode == "interp":
                command += ["-interp", "-no_workspace_output"]
            elif mode == "native":
                command += ["-exe", str(executable)]
            elif mode == "check":
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
                if "error_contains" in case:
                    passed = code == case["exit_code"] and case["error_contains"] in error and output == case["stdout"]
                elif mode == "check":
                    passed = code == 0
                else:
                    passed = code == 0 and output == case["stdout"]
                if not passed:
                    failures.append(f"{case['name']} [{mode}]: exit={code}, stdout={output!r}, stderr={error.strip()!r}")
            except subprocess.TimeoutExpired:
                failures.append(f"{case['name']} [{mode}]: exceeded {timeout}s timeout")
                passed = False
        print(("PASS" if passed else "FAIL") + f" {case['name']} [{mode}]")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"passed {len(cases)} stdlib public-contract fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
