#!/usr/bin/env python3
"""Validate crash reports with a synthetic Rust panic, isolated from compiler fuzz inputs.

    PYTHONPATH=tools python3 tools/fuzz/validate_crash_summary.py --out target/panic-summary

Requires rustc. The saved report, replay output and known input form the acceptance artifact.
The optional phases support an isolated intentionally failing Actions scheduling probe.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

import crash_summary as summary

DATA = b"summary probe\n\t\x00\xff```"
MESSAGE = "synthetic crash-summary acceptance panic"
SOURCE = r'''fn main() {
    let path = std::env::args().nth(1).expect("input");
    let bytes = std::fs::read(path).expect("read input");
    assert_eq!(bytes, b"summary probe\n\t\0\xff```");
    panic!("synthetic crash-summary acceptance panic");
}
'''


def prepare(out: Path) -> tuple[Path, Path]:
    """Build the isolated panic harness and identify its exact input bytes."""
    out.mkdir(parents=True, exist_ok=True)
    source = out / "panic.rs"
    source.write_text(SOURCE)
    program = out / ("panic.exe" if os.name == "nt" else "panic")
    subprocess.run(["rustc", "--edition=2024", "-g", str(source), "-o", str(program)], check=True, timeout=60)
    finding = out / ("crash-" + hashlib.sha256(DATA).hexdigest())
    finding.write_bytes(DATA)
    return program, finding


def report(out: Path) -> summary.Replay:
    """Replay and verify a report; a missing panic or incomplete report fails acceptance."""
    program = out / ("panic.exe" if os.name == "nt" else "panic")
    finding = out / ("crash-" + hashlib.sha256(DATA).hexdigest())
    replay = summary.run_replay([str(program), str(finding)], 10)
    (out / "replay.json").write_text(json.dumps({"status": replay.status, "output": replay.output}, indent=2))
    assert replay.status is not None and replay.status != 0, replay
    message, backtrace = summary.extract_panic(replay.output)
    assert MESSAGE in message, replay.output
    assert backtrace.startswith("stack backtrace:"), replay.output
    assert len(backtrace.splitlines()) <= summary.BACKTRACE_LINES
    report_path = out / "summary.md"
    previous = "Synthetic panic acceptance (not a compiler defect).\n\n"
    report_path.write_text(previous)
    old = os.environ.get("GITHUB_STEP_SUMMARY")
    os.environ["GITHUB_STEP_SUMMARY"] = str(report_path)
    try:
        assert summary.main(["check", "--artifacts", str(out)], builder=lambda: True, replayer=lambda _: replay) == 0
    finally:
        if old is None:
            os.environ.pop("GITHUB_STEP_SUMMARY", None)
        else:
            os.environ["GITHUB_STEP_SUMMARY"] = old
    rendered = report_path.read_text()
    for expected in [previous, finding.name, hashlib.sha256(DATA).hexdigest(), MESSAGE, "stack backtrace:", summary.hex_dump(DATA), summary.escape_text(DATA)]:
        assert expected in rendered, expected
    assert finding.read_bytes() == DATA
    return replay


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--phase", choices=["all", "fuzz", "summary"], default="all")
    args = parser.parse_args()
    out = args.out.resolve()
    if args.phase != "summary":
        prepare(out)
    if args.phase == "fuzz":
        done = subprocess.run([str(out / ("panic.exe" if os.name == "nt" else "panic")), str(out / ("crash-" + hashlib.sha256(DATA).hexdigest()))], env={**os.environ, **summary.REPLAY_ENV}, timeout=10)
        assert done.returncode != 0, "the deliberate panic did not occur"
        return done.returncode
    report(out)
    summary.append_summary((out / "summary.md").read_text())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
