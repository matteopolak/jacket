#!/usr/bin/env python3
"""Cross-source Windows time-zone layout checks; actual MSVC ABI is opt-in."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "stdlib/Bindings_Generator/tests/driven/sdk/windows-time-zone-layout.jai"
C_FIXTURE = ROOT / "stdlib/Bindings_Generator/tests/driven/sdk/windows_time_zone_sdk.c"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs():
    paths = {
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
        FIXTURE,
        C_FIXTURE,
        Path(__file__),
    }
    for folder, extensions in [
        ("crates", {".rs", ".toml", ".h", ".c"}),
        ("stdlib", {".jai"}),
    ]:
        paths.update(
            p
            for p in (ROOT / folder).rglob("*")
            if p.is_file() and p.suffix in extensions and not p.name.startswith("._")
        )
    paths.add(ROOT / "docs/specs/stdlib/modules/windows.md")
    return {str(p.relative_to(ROOT)): sha(p) for p in sorted(paths)}


def metrics(text):
    result = {}
    for line in text.splitlines():
        key, value = line.split()
        if key in result:
            raise ValueError("duplicate ABI metric: " + key)
        result[key] = int(value)
    if len(result) != 39:
        raise ValueError("incomplete ABI output: expected 39 metrics")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["compiler", "linter", "formatter"]:
        parser.add_argument("--" + name, type=Path)
    parser.add_argument(
        "--sdk-runtime",
        action="store_true",
        help="authorized normal Windows CI with MSVC/SDK: compare real C and Jai ABI",
    )
    args = parser.parse_args()
    supplied = [args.compiler, args.linter, args.formatter]
    if any(supplied) and not all(supplied):
        parser.error("provide compiler, linter and formatter together")
    if args.sdk_runtime and platform.system() != "Windows":
        parser.error("actual Windows SDK/runtime checks require a Windows host")
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    work = Path(tempfile.mkdtemp(prefix="windows-time-zone-", dir=target))
    temporary = work / "tmp"
    temporary.mkdir()
    env = dict(os.environ, TMPDIR=str(temporary), JAIC_STDLIB=str(ROOT / "stdlib"))
    before = inputs()
    rows = []
    suffix = ".exe" if platform.system() == "Windows" else ""
    compiler = (
        args.compiler.resolve(strict=True)
        if args.compiler
        else target / ("debug/jaic" + suffix)
    )
    lint = (
        args.linter.resolve(strict=True)
        if args.linter
        else target / ("debug/jailint" + suffix)
    )
    formatter = (
        args.formatter.resolve(strict=True)
        if args.formatter
        else target / ("jaifmt" + suffix)
    )

    def run(name, command, cwd=ROOT):
        command = list(map(str, command))
        try:
            result = subprocess.run(
                command, cwd=cwd, env=env, capture_output=True, text=True, timeout=240
            )
            row = dict(
                name=name,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=result.returncode == 0,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            row = dict(name=name, command=command, error=str(error), passed=False)
        rows.append(row)
        (work / "commands.json").write_text(json.dumps(rows, indent=2) + "\n")
        print(name, "PASS" if row["passed"] else "FAIL", flush=True)
        return row

    built = (
        all(supplied)
        or run(
            "source-build",
            [
                "cargo",
                "build",
                "--locked",
                "--offline",
                "-p",
                "jaic-cli",
                "-p",
                "jailint",
            ],
        )["passed"]
    )
    if built:
        run(
            "jai-format",
            [formatter, "--check", ROOT / "stdlib/Windows/declarations.jai", FIXTURE],
        )
        for cpu in ["x64", "arm64"]:
            run(
                "windows-source:" + cpu,
                [compiler, FIXTURE, "-check", "-os", "windows", "-cpu", cpu],
            )
            run(
                "windows-strict-lint:" + cpu,
                [
                    lint,
                    "-D",
                    "warnings",
                    "-os",
                    "windows",
                    "-cpu",
                    cpu,
                    "--context",
                    FIXTURE,
                    "--manifest",
                    work / ("lint-" + cpu + ".json"),
                    ROOT / "stdlib/Windows/declarations.jai",
                    FIXTURE,
                ],
            )
        if args.sdk_runtime:
            native = work / "sdk-layout.exe"
            c = run(
                "msvc-sdk-build",
                [
                    "cl",
                    "/nologo",
                    "/std:c11",
                    "/W4",
                    "/WX",
                    C_FIXTURE,
                    "/Fe:" + str(native),
                    "/Fo:" + str(work / "sdk-layout.obj"),
                ],
                cwd=work,
            )
            oracle = (
                run("msvc-sdk-execute", [native], cwd=work) if c["passed"] else None
            )
            expected = None
            if oracle and oracle["passed"]:
                try:
                    expected = metrics(oracle["stdout"])
                except ValueError as error:
                    rows.append(
                        dict(
                            name="msvc-sdk-metric-parse", passed=False, error=str(error)
                        )
                    )
            for mode in ["interp", "O0", "O2"]:
                if mode == "interp":
                    result = run("windows-jai:interp", [compiler, FIXTURE, "-interp"])
                else:
                    program = work / ("jai-layout-" + mode + ".exe")
                    built = run(
                        "windows-jai-build:" + mode,
                        [compiler, FIXTURE, "-" + mode, "-exe", program],
                    )
                    result = (
                        run("windows-jai-execute:" + mode, [program])
                        if built["passed"]
                        else None
                    )
                matches = False
                mismatch = None
                if expected is not None and result and result["passed"]:
                    try:
                        actual = metrics(result["stdout"])
                        matches = actual == expected
                        mismatch = {
                            key: dict(sdk=expected.get(key), jai=actual.get(key))
                            for key in expected.keys() | actual.keys()
                            if expected.get(key) != actual.get(key)
                        }
                    except ValueError as error:
                        mismatch = dict(error=str(error))
                rows.append(
                    dict(
                        name="actual-sdk-abi-match:" + mode,
                        passed=matches,
                        mismatches=mismatch,
                    )
                )
                print(
                    "actual-sdk-abi-match:" + mode,
                    "PASS" if matches else "FAIL",
                    flush=True,
                )
    after = inputs()
    drift = sorted(
        p for p in before.keys() | after.keys() if before.get(p) != after.get(p)
    )
    receipt = dict(
        commands=rows,
        source_inputs=before,
        source_drift=drift,
        tools={str(p): sha(p) for p in [compiler, lint, formatter] if p.is_file()},
        tool_origin="explicit source-built tools"
        if all(supplied)
        else "fresh CLI/lint build",
        actual_windows_sdk_runtime=args.sdk_runtime,
        host=platform.system(),
        passed=not drift and all(r["passed"] for r in rows),
    )
    (work / "summary.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(work / "summary.json", flush=True)
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
