"""Require discovery and actual execution of the standalone codec regressions.

PYTHONPATH=tools python3 tools/portable/image_conformance/check_registration.py --jacket target/debug/jacket --output target/image-registration
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess

from toolkit.repo import ROOT

REQUIRED = frozenset(("jpeg_dnl", "jpeg_structure", "jpeg_padding", "qoi_framing"))
MODES = ("interp", "native-asan", "native-o2", "wasi-o2")


def validate_events(events, *, catalogue):
    """Require each expected golden ID; a zero exit with missing or skipped cases fails."""
    if catalogue:
        cases = [row for row in events if row.get("type") == "list"]
        if (
            len(cases) != len(REQUIRED)
            or {row.get("name") for row in cases} != REQUIRED
        ):
            raise ValueError(
                "Jacket catalogue must contain every required codec regression exactly once"
            )
        if any(row.get("kind") != "golden" for row in cases):
            raise ValueError("standalone codec regressions must be golden programs")
        return
    started = [
        row
        for row in events
        if row.get("type") == "test" and row.get("event") == "started"
    ]
    completed = [
        row
        for row in events
        if row.get("type") == "test" and row.get("event") not in ("started", "slow")
    ]
    for rows in (started, completed):
        if len(rows) != len(REQUIRED) or {row.get("name") for row in rows} != REQUIRED:
            raise ValueError(
                "Jacket must actually start and finish every required codec regression"
            )
        if any(row.get("kind") != "golden" for row in rows):
            raise ValueError(
                "required codec cases were not executed as golden programs"
            )
    if any(row.get("event") != "passed" for row in completed):
        raise ValueError(
            "required codec regressions may not skip, fail, or pass as flaky/xfail"
        )
    summary = [
        row
        for row in events
        if row.get("type") == "run" and row.get("event") == "finished"
    ]
    if len(summary) != 1 or summary[0].get("passed") != len(REQUIRED):
        raise ValueError("Jacket did not finish the complete required run")
    if any(
        summary[0].get(field, 0)
        for field in (
            "failed",
            "timed_out",
            "flaky",
            "xfail",
            "xpass",
            "skipped",
            "not_run",
        )
    ):
        raise ValueError("Jacket summary contains a non-passing required outcome")


def invoke(command, output, name, *, catalogue):
    """Retain the actual runner transcript before validating it; e.g. invoke(cmd, out, 'interp', catalogue=False)."""
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    (output / (name + ".stdout")).write_text(result.stdout)
    (output / (name + ".stderr")).write_text(result.stderr)
    if result.returncode:
        raise subprocess.CalledProcessError(
            result.returncode, command, output=result.stdout, stderr=result.stderr
        )
    events = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    validate_events(events, catalogue=catalogue)
    return {"exit": result.returncode, "required_ids": sorted(REQUIRED), "passed": True}


def verify_scratch(compiler, output):
    """Prove the WASI mount is owned scratch, not its source cwd, and normal cleanup completes."""
    binary = output / "scratch.wasm"
    built = subprocess.run(
        [
            str(compiler),
            "-os",
            "wasm",
            "-O2",
            str(Path(__file__).with_name("registration_scratch.jai")),
            "-exe",
            str(binary),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    (output / "scratch-build.stdout").write_text(built.stdout)
    (output / "scratch-build.stderr").write_text(built.stderr)
    if built.returncode:
        raise RuntimeError("registration scratch probe did not compile")
    source = output / "probe-cwd"
    temporary = output / "probe-temp"
    source.mkdir()
    temporary.mkdir()
    ran = subprocess.run(
        [
            "node",
            "--no-warnings",
            str(Path(__file__).with_name("wasi_registration.mjs")),
            str(binary),
        ],
        cwd=source,
        env=dict(os.environ, TMPDIR=str(temporary)),
        capture_output=True,
        text=True,
    )
    (output / "scratch-run.stdout").write_text(ran.stdout)
    (output / "scratch-run.stderr").write_text(ran.stderr)
    if ran.returncode or ran.stdout.strip() != "registration scratch write passed":
        raise RuntimeError("WASI registration probe did not execute its scratch write")
    if list(source.iterdir()) or list(temporary.iterdir()):
        raise RuntimeError(
            "WASI registration host exposed its caller cwd or retained temporary scratch"
        )
    rejected = subprocess.run(
        [
            "node",
            "--no-warnings",
            str(Path(__file__).with_name("wasi_registration.mjs")),
            str(output / "missing.wasm"),
        ],
        cwd=source,
        env=dict(os.environ, TMPDIR=str(temporary)),
        capture_output=True,
        text=True,
    )
    (output / "scratch-error.stderr").write_text(rejected.stderr)
    if (
        rejected.returncode != 134
        or list(source.iterdir())
        or list(temporary.iterdir())
    ):
        raise RuntimeError(
            "WASI registration host did not clean up after module startup failure"
        )
    return {
        "passed": True,
        "caller_directory_untouched": True,
        "temporary_scratch_removed": True,
        "startup_failure_cleanup_checked": True,
    }


def main():
    """Run the real Jacket catalogue and all four modes; see the module command example."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jacket", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected-source")
    args = parser.parse_args()
    source_head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()
    source_status = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True
    )
    if args.expected_source and (source_head != args.expected_source or source_status):
        raise SystemExit(
            "required Jacket registration needs a clean checkout at the expected source"
        )
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    command = [
        str(args.jacket.resolve(strict=True)),
        "test",
        "--path",
        "stdlib/Extensions/Image",
        "--no-doc",
        "--format",
        "json",
        "-j",
        "2",
        "--exact",
        *sorted(REQUIRED),
    ]
    jacket = args.jacket.resolve(strict=True)
    compiler = jacket.with_name("jaic.exe" if jacket.suffix == ".exe" else "jaic")
    receipt = {
        "passed": False,
        "modes": {},
        "source_head": source_head,
        "source_status": source_status,
        "input_sha256": {
            str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in (
                Path(__file__).resolve(),
                Path(__file__).with_name("wasi_registration.mjs"),
                Path(__file__).with_name("registration_scratch.jai"),
                *(
                    ROOT / "stdlib/Extensions/Image/tests" / (name + extension)
                    for name in sorted(REQUIRED)
                    for extension in (".jai", ".exit")
                ),
            )
        },
        "jacket_sha256": hashlib.sha256(jacket.read_bytes()).hexdigest(),
        "compiler_sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
    }
    try:
        receipt["scratch_isolation"] = verify_scratch(compiler, output)
        receipt["catalogue"] = invoke(
            [*command, "--list"], output, "catalogue", catalogue=True
        )
        for mode in MODES:
            flags = {
                "interp": ["--interp"],
                "native-asan": ["--", "-O0", "-sanitize", "address,undefined"],
                "native-o2": ["--", "-O2"],
                "wasi-o2": [
                    "--os",
                    "wasm",
                    "--runner",
                    shlex.join(
                        [
                            "node",
                            "--no-warnings",
                            str(Path(__file__).with_name("wasi_registration.mjs")),
                        ]
                    ),
                    "--",
                    "-O2",
                ],
            }[mode]
            receipt["modes"][mode] = invoke(
                [*command, *flags], output, mode, catalogue=False
            )
            print(mode + ": all four required Jacket codec programs passed", flush=True)
        receipt["passed"] = True
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
