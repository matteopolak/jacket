#!/usr/bin/env python3
"""Run a predeclared number of independent Io::backends diagnostic attempts.

    python3 tools/ci/investigate_io.py --jacket target/release/jacket --out target/ci-io-investigation

This is a diagnostic experiment, not a test retry policy. Every attempt and failure is
retained; the driver itself forces each available backend and reports refused backends.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import time
from pathlib import Path

from toolkit.repo import ROOT


def main():
    import resource

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jacket", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--attempts", type=int, default=5, choices=range(1, 31), metavar="1..30")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    jacket = args.jacket.resolve()
    jaic = jacket.with_name("jaic")
    if not jacket.is_file() or not jaic.is_file():
        parser.error("the requested branch's jaic and jacket must exist beside each other")
    tool_env = {**os.environ, "JAIC_STDLIB": str(ROOT / "stdlib")}
    def public_path(path: Path) -> str:
        try:
            return path.relative_to(ROOT).as_posix()
        except ValueError:
            return "selected-tools/" + path.name

    environment = {
        "binaries": {public_path(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in [jaic, jacket]},
        "stdlib": "stdlib",
        "system": platform.system(), "machine": platform.machine(), "kernel": platform.release(),
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "limits": {name: resource.getrlimit(getattr(resource, name)) for name in ["RLIMIT_NOFILE", "RLIMIT_NPROC", "RLIMIT_AS"]},
        "attempts_per_mode": args.attempts,
    }
    for name in ["/proc/sys/kernel/io_uring_disabled", "/proc/sys/fs/file-max", "/proc/sys/fs/file-nr"]:
        path = Path(name)
        if path.exists():
            environment[name] = path.read_text()
    (out / "environment.json").write_text(json.dumps(environment, indent=2))
    base = [str(jacket), "test", "--path", str(ROOT / "stdlib/Extensions/Io"), "--no-doc", "--format", "json"]
    modes = {"native": [], "interp": ["--interp"], "o2": ["--", "-O2"]}
    outcomes = [{"mode": mode, "attempt": attempt, "state": "planned", "returncode": None}
                for mode in modes for attempt in range(1, args.attempts + 1)]

    def save_outcomes():
        # Atomic replacement leaves a readable denominator if the hosted job stops us.
        temporary = out / "outcomes.tmp"
        temporary.write_text(json.dumps(outcomes, indent=2))
        temporary.replace(out / "outcomes.json")

    save_outcomes()
    catalogue = out / "catalogue.jsonl"
    with catalogue.open("wb") as stdout, (out / "catalogue.stderr").open("wb") as stderr:
        listing = subprocess.run([*base, "--list"], cwd=ROOT, env=tool_env, stdout=stdout, stderr=stderr)
    listing.check_returncode()
    names = [record["name"] for line in catalogue.read_text().splitlines() if (record := json.loads(line)).get("name", "").split("::")[-1] == "backends"]
    assert len(names) == 1, names
    for record in outcomes:
        mode, attempt = record["mode"], record["attempt"]
        command = [*base, "--exact", names[0], *modes[mode]]
        prefix = out / f"{mode}-{attempt}"
        public_command = [public_path(jacket), *command[1:]]
        public_command = [arg.replace(str(ROOT) + os.sep, "") for arg in public_command]
        prefix.with_suffix(".command.json").write_text(json.dumps(public_command))
        record["state"] = "running"
        save_outcomes()
        started = time.monotonic()
        # Jacket owns test-process groups and their ordinary case timeouts. Killing
        # only Jacket here could orphan those groups and contaminate later attempts.
        # A hung supervisor stops this experiment; the hosted job bounds its lifecycle.
        with prefix.with_suffix(".stdout").open("wb") as stdout, prefix.with_suffix(".stderr").open("wb") as stderr:
            done = subprocess.run(command, cwd=ROOT, env=tool_env, stdout=stdout, stderr=stderr)
        record.update(state="completed", returncode=done.returncode, seconds=time.monotonic() - started)
        save_outcomes()
        print(json.dumps(record), flush=True)
    return int(any(record["state"] != "completed" or record["returncode"] != 0 for record in outcomes))



if __name__ == "__main__":
    raise SystemExit(main())
