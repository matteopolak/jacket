#!/usr/bin/env python3
"""File_Watcher source proof, and opt-in real Linux/Windows CI execution."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OWN = [
    "stdlib/File_Watcher/module.jai",
    "stdlib/internal/watching/portable-model.jai",
    "stdlib/internal/watching/portable.jai",
    "stdlib/internal/watching/inotify.jai",
    "stdlib/internal/watching/windows.jai",
    "stdlib/internal/watching/event-records/module.jai",
    "stdlib/File_Watcher/tests/event-records.jai",
    *[
        "tests/portable-stdlib/file-watcher/" + name + ".jai"
        for name in ["platform-contracts", "platform-effects", "platform-edges"]
    ],
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot():
    paths = {ROOT / "Cargo.toml", ROOT / "Cargo.lock"}
    for folder, suffixes in [
        ("crates", {".rs", ".toml", ".c", ".h"}),
        ("stdlib", {".jai", ".c", ".h"}),
        ("jaifmt", {".jai"}),
    ]:
        paths.update(
            p
            for p in (ROOT / folder).rglob("*")
            if p.is_file() and p.suffix in suffixes and not p.name.startswith("._")
        )
    paths.update(ROOT / p for p in OWN)
    paths.add(Path(__file__).resolve())
    paths.add(ROOT / "docs/specs/stdlib/modules/file-watcher.md")
    return {str(p.relative_to(ROOT)): sha(p) for p in sorted(paths)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["compiler", "linter", "formatter"]:
        parser.add_argument("--" + name, type=Path)
    parser.add_argument(
        "--native-records",
        action="store_true",
        help="coordinator-authorized host-only bounded decoder O0/O2 checks",
    )
    parser.add_argument(
        "--native-runtime",
        action="store_true",
        help="coordinator-authorized actual Linux/Windows CI: run filesystem tests",
    )
    args = parser.parse_args()
    supplied = [args.compiler, args.linter, args.formatter]
    if any(supplied) and not all(supplied):
        parser.error("provide compiler, linter and formatter together")
    if args.native_runtime and platform.system() not in {"Linux", "Windows"}:
        parser.error(
            "platform runtime acceptance requires an actual Linux/Windows host"
        )
    target = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    work = Path(tempfile.mkdtemp(prefix="file-watcher-proof-", dir=target))
    temporary = work / "tmp"
    temporary.mkdir()
    env = dict(os.environ, TMPDIR=str(temporary), JAIC_STDLIB=str(ROOT / "stdlib"))
    before = snapshot()
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

    def run(name, command, expected_stdout=None):
        command = list(map(str, command))
        try:
            result = subprocess.run(
                command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180
            )
            passed = result.returncode == 0 and (
                expected_stdout is None or result.stdout == expected_stdout
            )
            row = dict(
                name=name,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        except (subprocess.TimeoutExpired, OSError) as error:
            row = dict(name=name, command=command, error=str(error), passed=False)
        rows.append(row)
        (work / "commands.json").write_text(json.dumps(rows, indent=2) + "\n")
        print(name, "PASS" if row["passed"] else "FAIL", flush=True)
        return row["passed"]

    built = True
    if not all(supplied):
        built = run(
            "source-build-cli-lint",
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
        )
    # The formatter must already be source-built in the assigned target. Do not
    # borrow peer binaries or silently build native tools without coordination.
    run("source-formatter-check", [formatter, "--check", *[ROOT / p for p in OWN]])
    if built:
        for system in ["linux", "windows", "macos"]:
            for cpu in ["x64", "arm64"]:
                fixtures = ["platform-contracts"]
                if system != "macos":
                    fixtures += ["platform-effects", "platform-edges"]
                for fixture in fixtures:
                    source = (
                        ROOT / "tests/portable-stdlib/file-watcher" / (fixture + ".jai")
                    )
                    run(
                        f"source:{system}:{cpu}:{fixture}",
                        [
                            compiler,
                            source,
                            "-check",
                            "-no_dce",
                            "-os",
                            system,
                            "-cpu",
                            cpu,
                        ],
                    )
                if system == "macos":
                    continue  # Existing Mac implementation remains unchanged.
                selected = [ROOT / OWN[i] for i in [0, 1, 2, 5]] + [
                    ROOT
                    / f"stdlib/internal/watching/{'inotify' if system == 'linux' else 'windows'}.jai"
                ]
                for fixture in fixtures:
                    context = (
                        ROOT / "tests/portable-stdlib/file-watcher" / (fixture + ".jai")
                    )
                    run(
                        f"strict-lint:{system}:{cpu}:{fixture}",
                        [
                            lint,
                            "-D",
                            "warnings",
                            "-os",
                            system,
                            "-cpu",
                            cpu,
                            "--context",
                            context,
                            "--manifest",
                            work / f"lint-{system}-{cpu}-{fixture}.json",
                            *selected,
                            context,
                        ],
                    )
        unit = ROOT / "stdlib/File_Watcher/tests/event-records.jai"
        run("bounded-records-interpreter", [compiler, unit, "-interp"], "ok\n")
        run(
            "bounded-records-strict-lint",
            [lint, "-D", "warnings", "--context", unit, ROOT / OWN[5], unit],
        )
        if args.native_records:
            for optimization in ["-O0", "-O2"]:
                executable = work / ("bounded-records" + optimization + suffix)
                if run(
                    "bounded-records-build:" + optimization,
                    [compiler, unit, optimization, "-exe", executable],
                ):
                    run("bounded-records-native:" + optimization, [executable], "ok\n")
        if args.native_runtime:
            for fixture in ["platform-effects", "platform-edges"]:
                source = (
                    ROOT / "tests/portable-stdlib/file-watcher" / (fixture + ".jai")
                )
                directory = work / (fixture + "-interp-data")
                directory.mkdir()
                run(
                    f"runtime:{fixture}:interp",
                    [compiler, source, "-interp", "--", directory],
                    "ok\n",
                )
                for optimization in ["-O0", "-O2"]:
                    program = work / (fixture + optimization + suffix)
                    if run(
                        f"build:{fixture}:{optimization}",
                        [compiler, source, optimization, "-exe", program],
                    ):
                        directory = work / (fixture + optimization + "-data")
                        directory.mkdir()
                        run(
                            f"runtime:{fixture}:{optimization}",
                            [program, directory],
                            "ok\n",
                        )
    after = snapshot()
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
        native_platform_runtime_requested=args.native_runtime,
        native_records_requested=args.native_records,
        host=platform.system(),
        passed=not drift and all(r["passed"] for r in rows),
    )
    (work / "summary.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(work / "summary.json", flush=True)
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
