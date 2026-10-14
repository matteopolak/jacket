#!/usr/bin/env python3
"""Run public-language contract fixtures with an explicitly selected local jaic build."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).resolve().parent


def compiler_path(raw: str) -> Path:
    path = Path(raw).expanduser()
    if not path.is_absolute():
        raise SystemExit("--jaic must be an absolute path to a locally source-built compiler")
    resolved = path.resolve(strict=True)
    target = (ROOT / "target").resolve()
    if target not in resolved.parents:
        raise SystemExit(f"refusing to run a compiler outside this workspace's target directory: {resolved}")
    if resolved.name not in {"jaic", "jaic.exe"}:
        raise SystemExit("--jaic must name the jaic executable")
    if not os.access(resolved, os.X_OK):
        raise SystemExit(f"compiler is not executable: {resolved}")
    return resolved


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, help="absolute path to a source-built jaic under this workspace's target/")
    parser.add_argument("--stdlib-root", type=Path, default=ROOT / "stdlib", help="standard-library root for the source-built compiler")
    parser.add_argument("--timeout", type=float, default=30, help="seconds per fixture")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    jaic = compiler_path(args.jaic)
    cases = json.loads((FIXTURES / "manifest.json").read_text())
    import_dir = FIXTURES / "modules"
    environment = os.environ.copy()
    environment["JAIC_STDLIB"] = str(args.stdlib_root.resolve(strict=True))

    failures = []
    for case in cases:
        source = FIXTURES / case["main"]
        mode = case["mode"]
        command = [str(jaic), str(source)]
        command += ["-interp", "-no_workspace_output"] if mode == "interp" else ["-check"]
        command += ["-import_dir", str(import_dir)]
        try:
            result = subprocess.run(command, cwd=source.parent, env=environment, text=True,
                                    capture_output=True, timeout=args.timeout)
        except subprocess.TimeoutExpired:
            failures.append(f"{case['name']}: exceeded {args.timeout}s timeout")
            print(f"FAIL {case['name']} (timeout)")
            continue

        if mode == "check_error":
            ok = result.returncode != 0 and case["expected_stderr_contains"] in result.stderr
            if not ok:
                failures.append(
                    f"{case['name']}: expected compile error ({case['expected_error_reason']}); "
                    f"exit={result.returncode}, stderr={result.stderr.strip()!r}"
                )
        else:
            ok = result.returncode == 0 and result.stdout == case["expected_stdout"]
            if not ok:
                failures.append(
                    f"{case['name']}: expected exit 0 and stdout {case['expected_stdout']!r}; "
                    f"exit={result.returncode}, stdout={result.stdout!r}, stderr={result.stderr.strip()!r}"
                )
        print(("PASS" if ok else "FAIL") + f" {case['name']}")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"passed {len(cases)} public-language fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
