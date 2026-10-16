#!/usr/bin/env python3
"""Canonical Bucket_Array pointer indexing; source/core default, native only after lease."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
T7 = Path("/Volumes/T7/codex-builds/jai-cleanroom-01a137eb/foundations")
TARGET = T7 / "bucket-pointer-target"
JAIC = TARGET / "debug/jaic"
LINT = TARGET / "debug/jailint"
FORMATTER = T7 / "matrix-operator-proof-39s4u7vr/jaifmt"
OWNED_PRODUCTION = [
    "stdlib/Bucket_Array/module.jai",
    "stdlib/internal/collections/bucket-array.jai",
]
POSITIVE = [
    "addresses",
    "helper-control",
    "holes-removal",
    "reexport",
    "fault-control",
    "type-privacy",
    "heap-workflow",
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--native",
        action="store_true",
        help="execute O0/O2 only after an explicit native lease release/grant",
    )
    args = parser.parse_args()
    out = Path(tempfile.mkdtemp(prefix="bucket-pointer-proof-", dir=T7))
    scratch = out / "tmp"
    scratch.mkdir()
    env = dict(
        os.environ,
        JAIC_STDLIB=str(ROOT / "stdlib"),
        CARGO_TARGET_DIR=str(TARGET),
        TMPDIR=str(scratch),
    )
    report = out / "results.json"
    evidence = {
        "source_core_complete": False,
        "native_requested": args.native,
        "native_programs_executed": 0,
        "commands": [],
        "cwd": str(out),
        "environment": {
            n: env[n] for n in ["JAIC_STDLIB", "CARGO_TARGET_DIR", "TMPDIR"]
        },
    }

    def save():
        report.write_text(json.dumps(evidence, indent=2) + "\n")

    def run(command, expected="", required_errors=None):
        p = subprocess.run(
            [str(x) for x in command],
            cwd=out,
            env=env,
            text=True,
            capture_output=True,
            timeout=120,
        )
        valid = (
            (p.returncode != 0 and all(s in p.stderr for s in required_errors))
            if required_errors
            else (p.returncode == 0 and p.stdout == expected and p.stderr == "")
        )
        record = {
            "command": [str(x) for x in command],
            "exit_code": p.returncode,
            "stdout": p.stdout,
            "stderr": p.stderr,
            "passed": valid,
            "required_errors": required_errors,
        }
        evidence["commands"].append(record)
        save()
        if not valid:
            raise RuntimeError(json.dumps(record, indent=2))

    compiler = sorted((ROOT / "crates").rglob("*.rs")) + sorted(
        (ROOT / "crates").rglob("Cargo.toml")
    )
    compiler += [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]

    def hashes(paths):
        return {str(p.relative_to(ROOT)): sha(p) for p in paths}

    evidence["compiler_before"] = hashes(compiler)
    build = [
        "cargo",
        "build",
        "--locked",
        "--offline",
        "-j",
        "2",
        "-p",
        "jaic-cli",
        "-p",
        "jailint",
    ]
    p = subprocess.run(
        build, cwd=ROOT, env=env, text=True, capture_output=True, timeout=120
    )
    evidence["build"] = {
        "command": build,
        "exit_code": p.returncode,
        "stdout": p.stdout,
        "stderr": p.stderr,
    }
    save()
    if p.returncode:
        raise RuntimeError("Source compiler build failed")
    assert evidence["compiler_before"] == hashes(compiler)
    evidence["tools"] = {str(p): sha(p) for p in [JAIC, LINT, FORMATTER]}
    owned = [ROOT / p for p in OWNED_PRODUCTION] + sorted(HERE.rglob("*.jai"))
    run([FORMATTER, "--config", ROOT / "jaifmt.toml", *owned])
    run([FORMATTER, "--config", ROOT / "jaifmt.toml", "--check", *owned])
    watched = [ROOT / p for p in OWNED_PRODUCTION] + [
        ROOT / "stdlib/Basic/module.jai",
        ROOT / "stdlib/internal/basic/apollo.jai",
        ROOT / "stdlib/internal/basic/clock-calendar.jai",
    ]
    evidence["source_before"] = hashes(watched)
    imports = ["-I", HERE / "modules"]
    for name in POSITIVE:
        run([JAIC, HERE / (name + ".jai"), *imports, "-check"])
    existing = ROOT / "tests/portable-stdlib/bucket-array-remove.jai"
    evidence["existing_remove_fixture_sha256"] = sha(existing)
    run([JAIC, existing, "-check"])

    negative = {
        "type-only-negative": ["cannot index a value of type Bucket_Array"],
        "private-negative": ["cannot index a value of type Bucket_Array"],
        "facade-private-negative": ["has no exported member", "Impl"],
        "fault-hole": [
            "assertion failed:",
            "bucket.occupied[slot_index]",
            "operator*[]",
        ],
        "fault-negative": ["assertion failed:", "index >= 0", "operator*[]"],
        "fault-past-bucket": [
            "assertion failed:",
            "bucket_index < all_buckets.count",
            "operator*[]",
        ],
        "fault-empty": [
            "assertion failed:",
            "bucket_index < all_buckets.count",
            "operator*[]",
        ],
        "fault-huge": [
            "assertion failed:",
            "bucket_index < all_buckets.count",
            "operator*[]",
        ],
    }
    for name, errors in negative.items():
        run([JAIC, HERE / (name + ".jai"), *imports, "-check"], required_errors=errors)

    evidence["lint_contexts"] = {}
    for name in POSITIVE:
        path = HERE / (name + ".jai")
        manifest = out / ("lint-" + name + ".json")
        run(
            [
                LINT,
                "-D",
                "warnings",
                *imports,
                "--context",
                path,
                "--manifest",
                manifest,
                path,
            ]
        )
        coverage = json.loads(manifest.read_text())
        assert coverage["contexts"] and all(c["complete"] for c in coverage["contexts"])
        evidence["lint_contexts"][name] = coverage
    manifest = out / "lint-production.json"
    run(
        [
            LINT,
            "-D",
            "warnings",
            *imports,
            "--context",
            HERE / "addresses.jai",
            "--manifest",
            manifest,
            *[ROOT / p for p in OWNED_PRODUCTION],
        ]
    )
    evidence["lint_production"] = json.loads(manifest.read_text())
    assert evidence["lint_production"]["contexts"]
    assert all(c["complete"] for c in evidence["lint_production"]["contexts"])
    evidence["source_core_complete"] = True

    if args.native:
        # Type/privacy-only and intentional assertion-failure programs are never
        # represented as native behavior proofs. These six programs execute real
        # pointer operations, including the heap allocation/growth/reset workflow.
        for index, name in enumerate(n for n in POSITIVE if n != "type-privacy"):
            for level in [0, 2]:
                exe = out / ("native-" + str(index) + "-o" + str(level))
                run(
                    [
                        JAIC,
                        HERE / (name + ".jai"),
                        *imports,
                        "-O" + str(level),
                        "-no_backtrace_on_crash",
                        "-exe",
                        exe,
                    ]
                )
                run([exe], expected="ok\n")
                evidence["native_programs_executed"] += 1
    evidence["source_after"] = hashes(watched)
    evidence["compiler_after"] = hashes(compiler)
    assert evidence["source_before"] == evidence["source_after"]
    assert evidence["compiler_before"] == evidence["compiler_after"]
    evidence["fixture_sha256"] = {
        str(p.relative_to(ROOT)): sha(p) for p in HERE.rglob("*.jai")
    }
    evidence["reviewed_spec_sha256"] = sha(
        ROOT / "docs/specs/stdlib/modules/bucket-array.md"
    )
    save()
    print(report)


if __name__ == "__main__":
    main()
