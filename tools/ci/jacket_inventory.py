#!/usr/bin/env python3
"""Allowlist public coverage evidence from jacket JSON without retaining test output.

    jacket test --format json | python3 tools/ci/jacket_inventory.py outcomes
    python3 tools/ci/jacket_inventory.py metadata --partition hash:1/2

Only relative case identities, kinds, events, skip categories and public run/tool versions
leave this filter. Reporter paths, notes, stdout/stderr and arbitrary diagnostic text do not.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import re
import subprocess
import sys

KINDS = {"test", "golden", "compile_fail", "doc", "bench", "subtest"}
EVENTS = {"started", "passed", "failed", "skipped", "timed_out", "xfail", "xpass", "flaky"}
COUNTS = {"tests", "passed", "failed", "skipped", "timed_out", "not_run", "xfail", "xpass", "flaky"}


def identity(record: dict) -> dict:
    name, kind = record["name"], record["kind"]
    if not isinstance(name, str) or name.startswith(("/", "\\")) or re.match(r"^[A-Za-z]:(?!:)", name) or ".." in name.split("/"):
        raise ValueError("non-relative case identity")
    if kind not in KINDS:
        raise ValueError("unknown case kind")
    return {"name": name, "kind": kind}


def skip_category(reason: str | None) -> str | None:
    if not reason:
        return None
    if reason == "ignored (`jai,ignore`)":
        return "declared-ignore"
    if reason in {"interp only", "native only"}:
        return "mode-filter"
    if reason.startswith(("only on ", "skipped on ")):
        return "target-filter"
    if reason.startswith("built for "):
        return "cross-target-runner"
    if reason.startswith("missing test prerequisite:"):
        return "prerequisite"
    if reason.startswith("needs "):
        return "capability"
    return "declared-or-runtime-skip"


def sanitize(record: dict, mode: str) -> dict | None:
    """Select a fixed schema, never copying arbitrary reporter/environment fields."""
    if mode == "list":
        if record.get("type") != "list":
            raise ValueError("expected a discovery record")
        return {"type": "list", **identity(record)}
    if record.get("type") == "test":
        event = record["event"]
        # The reporter also emits progress notices without a kind. They are not
        # completed outcomes and cannot satisfy the verifier's terminal ledger.
        if event == "slow":
            return None
        if event not in EVENTS:
            raise ValueError("unknown test outcome")
        result = {"type": "test", "event": event, **identity(record), "reason_category": skip_category(record.get("reason"))}
        if "subtests" in record:
            children = []
            for child in record["subtests"]:
                name = identity({"name": child["name"], "kind": "subtest"})["name"]
                if child["status"] not in {"pass", "fail", "skip"}:
                    raise ValueError("unknown child outcome")
                children.append({"name": name, "status": child["status"]})
            result["subtests"] = children
        return result
    if record.get("type") == "run" and record.get("event") in {"started", "finished"}:
        counts = {key: value for key, value in record.items() if key in COUNTS}
        if any(not isinstance(value, int) or value < 0 for value in counts.values()):
            raise ValueError("invalid run count")
        return {"type": "run", "event": record["event"], **counts}
    raise ValueError("unknown inventory record")


def diagnostic_record(raw: dict) -> dict:
    """Normal hosted failure details stay in the masked job log, outside the artifact."""
    fields = ("type", "event", "name", "kind", "file", "line", "reason", "stdout", "stderr", "problems")
    result = {key: raw[key] for key in fields if key in raw}
    if "failures" in raw:
        result["failures"] = [{key: failure[key] for key in ("test", "file", "line", "message") if key in failure}
                              for failure in raw["failures"]]
    return result


def metadata(partition: str) -> dict:
    """Public source/run identity only; never enumerate environment variables or hostname."""
    if not re.fullmatch(r"all|(?:hash|count):[1-9][0-9]*/[1-9][0-9]*", partition):
        raise ValueError("invalid partition")
    revision = os.environ.get("GITHUB_SHA") or subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("invalid source revision")
    rustc = subprocess.check_output(["rustc", "--version"], text=True).strip()
    if not re.fullmatch(r"rustc [A-Za-z0-9.+-]+ \([0-9a-f]+ [0-9-]+\)", rustc):
        raise ValueError("unexpected public Rust version format")
    return {"revision": revision, "partition": partition, "os": platform.system(), "arch": platform.machine(), "rustc": rustc}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["list", "outcomes", "metadata"])
    parser.add_argument("--partition", default="all")
    parser.add_argument("--diagnostics", action="store_true", help="Retain normal failed-case details in hosted CI logs only")
    args = parser.parse_args()
    if args.diagnostics and (args.mode != "outcomes" or os.environ.get("GITHUB_ACTIONS") != "true"):
        parser.error("--diagnostics is reserved for outcome capture in hosted CI")
    if args.mode == "metadata":
        print(json.dumps(metadata(args.partition), sort_keys=True))
        return
    error = None
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            raw = json.loads(line)
            if not isinstance(raw, dict):
                raise ValueError("expected a JSON object")
            value = sanitize(raw, args.mode)
        except (ValueError, TypeError, KeyError) as problem:
            # Keep reading before failing: closing the pipe could stop the real
            # test command, changing its result and discarding later outcomes.
            if error is None:
                error = problem
            continue
        if value is None:
            continue
        if args.diagnostics and raw.get("type") == "test" and raw.get("event") in {"failed", "timed_out", "xpass", "flaky"}:
            print("JACKET_TEST_DIAGNOSTIC " + json.dumps(diagnostic_record(raw), sort_keys=True), file=sys.stderr, flush=True)
        print(json.dumps(value, sort_keys=True), flush=True)
    if error is not None:
        raise error


if __name__ == "__main__":
    main()
