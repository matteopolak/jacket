#!/usr/bin/env python3
"""Guarded source checks for null-format contracts/probes; no runtime execution."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = [
    "tests/portable-stdlib/format-null-any-contract.jai",
    "tests/portable-stdlib/format-null-type-probe.jai",
    "tests/portable-stdlib/format-null-descriptor-probe.jai",
    "tests/portable-stdlib/format-default-type-identity.jai",
    "tests/portable-stdlib/format-null-lvalue-any-probe.jai",
    "tests/portable-stdlib/format-any-missing-value.jai",
    "tests/portable-stdlib/format-any-missing-type.jai",
    "tests/portable-stdlib/format-any-missing-value-no-assert.jai",
]
PRODUCTION = [
    "stdlib/internal/basic/format.jai",
    "stdlib/internal/basic/format-type-names.jai",
]


def hashes():
    paths = {
        ROOT / name
        for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]
        + FIXTURES
        + PRODUCTION
    }
    paths.add(Path(__file__).resolve())
    for directory in ["stdlib", "jaifmt"]:
        paths.update(
            p for p in (ROOT / directory).rglob("*.jai") if not p.name.startswith("._")
        )
    paths.update(
        p
        for p in (ROOT / "crates").rglob("*")
        if p.is_file() and (p.suffix in {".rs", ".c", ".h"} or p.name == "Cargo.toml")
    )
    paths.add(ROOT / "docs/specs/stdlib/modules/basic.md")
    return {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(paths)
    }


def main():
    base = Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    work = Path(tempfile.mkdtemp(prefix="null-format-source-", dir=base))
    target = work / "tools"
    temporary = work / "tmp"
    temporary.mkdir()
    env = dict(
        os.environ,
        CARGO_TARGET_DIR=str(target),
        TMPDIR=str(temporary),
        JAIC_STDLIB=str(ROOT / "stdlib"),
        PYTHONDONTWRITEBYTECODE="1",
        RUFF_CACHE_DIR=str(work / "ruff-cache"),
    )
    before = hashes()
    rows = []

    def run(name, command):
        result = subprocess.run(
            list(map(str, command)),
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=240,
        )
        rows.append(
            dict(
                name=name,
                command=list(map(str, command)),
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=result.returncode == 0,
            )
        )
        (work / "commands.json").write_text(json.dumps(rows, indent=2) + "\n")
        print(name, "PASS" if result.returncode == 0 else "FAIL", flush=True)
        return result.returncode == 0

    if run(
        "fresh-tools",
        ["cargo", "build", "--locked", "--offline", "-p", "jaic-cli", "-p", "jailint"],
    ):
        compiler, lint = target / "debug/jaic", target / "debug/jailint"
        formatter = work / "jaifmt"
        if run(
            "formatter-build",
            [compiler, ROOT / "jaifmt/main.jai", "-O2", "-exe", formatter],
        ):
            run(
                "owned-test-format",
                [formatter, "--check", *[ROOT / p for p in FIXTURES + PRODUCTION]],
            )
        for system in ["windows", "linux", "macos"]:
            for cpu in ["x64", "arm64"]:
                for fixture in FIXTURES:
                    run(
                        "source:" + system + ":" + cpu + ":" + Path(fixture).name,
                        [
                            compiler,
                            ROOT / fixture,
                            "-check",
                            "-no_dce",
                            "-os",
                            system,
                            "-cpu",
                            cpu,
                        ],
                    )
                run(
                    "strict-lint:" + system + ":" + cpu,
                    [
                        lint,
                        "-D",
                        "warnings",
                        "-os",
                        system,
                        "-cpu",
                        cpu,
                        "--context",
                        ROOT / FIXTURES[1],
                        "--manifest",
                        work / ("lint-" + system + "-" + cpu + ".json"),
                        *[ROOT / p for p in PRODUCTION],
                    ],
                )
    run(
        "python-lint",
        ["ruff", "check", Path(__file__), ROOT / "tools/check_basic_any_read.py"],
    )
    run(
        "python-format",
        [
            "ruff",
            "format",
            "--check",
            Path(__file__),
            ROOT / "tools/check_basic_any_read.py",
        ],
    )
    after = hashes()
    drift = sorted(
        name
        for name in before.keys() | after.keys()
        if before.get(name) != after.get(name)
    )
    report = dict(
        inputs=before,
        source_drift=drift,
        commands=rows,
        passed=not drift and all(row["passed"] for row in rows),
        formatter_utility_only=True,
        formatting_runtime_tests_executed=0,
        null_any_policy="Only all-zero Any is empty; corrupt partial-null values fault when formatted; boxing is readless",
        null_Type_policy="Official default scalar/record Type is void; explicit null descriptor invalid; compiler default-image reducer RED",
        production_modified=True,
        legacy_oracles_modified=False,
    )
    (work / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "summary.json", flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
