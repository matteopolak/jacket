#!/usr/bin/env python3
"""Collect and verify actual compiler target-work identities from an opt-in CI run.

    cargo nextest run ... | python3 tools/ci/compiler_target_inventory.py collect
    python3 tools/ci/compiler_target_inventory.py verify downloaded-artifacts --tier full --revision EXPECTED_SHA

The Rust harness owns module discovery. Only its reserved, fixed-schema records enter
an artifact; ordinary compiler output stays in the job log. Verification also checks
per-OS doc discovery and completed outcomes against each job's assigned identities.
"""
from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import sys

from jacket_inventory import identity
from verify_jacket_inventory import inventory, records

PREFIX = "STDLIB_TARGET_INVENTORY "
DOC_OSES = ("linux", "macos", "windows", "wasm")
DOC_SKIP_CATEGORIES = {"target-filter", "mode-filter", "declared-ignore", "capability"}
FIELDS = {"full": {"event", "pairs"}, "assigned": {"event", "pairs"},
          "completed": {"event", "pair"}, "verified": {"event"}}


def pair(value):
    """A module import identity and target label, never a filesystem path or diagnostic."""
    if not isinstance(value, list) or len(value) != 2:
        raise ValueError("invalid module/target identity")
    module, target = value
    identity({"name": module, "kind": "test"})
    if not isinstance(target, str) or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", target):
        raise ValueError("invalid target label")
    return module, target


def record(value):
    event = value.get("event") if isinstance(value, dict) else None
    if not isinstance(event, str) or event not in FIELDS:
        raise ValueError("unknown target inventory record")
    if set(value) != FIELDS[event]:
        raise ValueError("unexpected target inventory fields")
    if event in {"full", "assigned"}:
        if not isinstance(value["pairs"], list):
            raise ValueError("invalid pair list")
        return {"event": event, "pairs": [list(pair(item)) for item in value["pairs"]]}
    if event == "completed":
        return {"event": event, "pair": list(pair(value["pair"]))}
    return {"event": event}


def unique(values, label):
    keys = [pair(value) for value in values]
    assert len(keys) == len(set(keys)), f"duplicate {label} identities"
    return set(keys)


def job_records(values):
    """Require returned compiler processes and the marker after expectation assertions."""
    values = [record(value) for value in values]
    events = [value["event"] for value in values]
    assert len(events) >= 3 and events[:2] == ["full", "assigned"], "missing target inventory header"
    assert events[-1] == "verified" and all(event == "completed" for event in events[2:-1]), "missing or misplaced verified marker"
    whole = unique(values[0]["pairs"], "full")
    assert whole, "empty module/target catalogue"
    assigned = unique(values[1]["pairs"], "assigned")
    completed = unique([value["pair"] for value in values[2:-1]], "completed")
    assert assigned <= whole, "assigned identity outside full catalogue"
    assert completed == assigned, "completed identities differ from assigned share"
    return whole, assigned


def collect(lines, emit=None):
    """Ignore nextest framing; retain partial valid records before rejecting incomplete proof."""
    values = []
    error = None
    for line in lines:
        line = re.sub(r"\x1b\[[0-9;]*m", "", line).lstrip()
        if line.startswith(PREFIX):
            try:
                value = record(json.loads(line[len(PREFIX):]))
            except (ValueError, TypeError) as problem:
                # Drain nextest even after malformed evidence: closing its pipe
                # could terminate the real test command and change its outcome.
                if error is None:
                    error = problem
                continue
            values.append(value)
            if emit is not None:
                emit(value)
    if error is not None:
        raise error
    job_records(values)
    return values


def module_inventory(artifacts):
    whole = None
    canonical = None
    assigned = set()
    for artifact in artifacts:
        status = json.loads((artifact / "module-status.json").read_text())
        assert set(status) == {"command", "tee", "capture"}, "missing command/capture status"
        assert all(type(value) is int and value == 0 for value in status.values()), "unsuccessful target command/capture"
        values = records(artifact / "modules.jsonl")
        full, part = job_records(values)
        ordered = [pair(value) for value in values[0]["pairs"]]
        if whole is None:
            whole, canonical = full, ordered
        assert full == whole and ordered == canonical, "different full module/target catalogues"
        metadata = json.loads((artifact / "metadata.json").read_text())
        partition = re.fullmatch(r"count:([1-9][0-9]*)/([1-9][0-9]*)", metadata["partition"])
        assert partition is not None, "invalid module/target partition"
        numerator, denominator = map(int, partition.groups())
        assert numerator <= denominator, "invalid module/target partition"
        expected = {value for index, value in enumerate(canonical) if index % denominator == numerator - 1}
        assert part == expected, "assigned module/target identities differ from declared partition"
        assert not assigned & part, "overlapping module/target shares"
        assigned |= part
    assert whole is not None and assigned == whole, "incomplete module/target union"
    return whole


def verify(root, full, expected_revision):
    """Require the requested head and every source-matrix row, with exact per-job outcomes."""
    assert re.fullmatch(r"[0-9a-f]{40}", expected_revision), "invalid expected revision"
    # Collection is a pure stream filter; only offline verification loads the CI model.
    import ci_changes

    expected = {}
    for row in ci_changes.test_matrix(full):
        if ",targets," in row["parts"]:
            expected.setdefault(row["runner"], []).append(row)
    groups = {}
    for path in root.glob("compiler-target-inventory-*"):
        if path.is_dir():
            platform = path.name.removeprefix("compiler-target-inventory-").rsplit("-", 1)[0]
            groups.setdefault(platform, []).append(path)
    assert groups.keys() == expected.keys(), "missing or unexpected target-work platform"
    result = {}
    source = None
    all_pairs = None
    for platform, artifacts in groups.items():
        metadata = [json.loads((path / "metadata.json").read_text()) for path in artifacts]
        assert all(set(item) == {"revision", "partition", "os", "arch", "rustc"} for item in metadata), "unexpected metadata fields"
        assert Counter(item["partition"] for item in metadata) == Counter(
            "count:" + row.get("targets_part", row["shard"]) for row in expected[platform]
        ), "missing or duplicate target-work shares"
        signatures = {(item["revision"], item["os"], item["arch"], item["rustc"]) for item in metadata}
        assert len(signatures) == 1, "different source/host/toolchain metadata"
        revision, os_name, arch, rustc = signatures.pop()
        assert revision == expected_revision, "stale source revision"
        assert arch == expected[platform][0]["architecture"], "unexpected runner architecture"
        assert os_name == ("Darwin" if platform.startswith("macos-") else "Linux"), "unexpected runner OS"
        if source is None:
            source = revision, rustc
        assert source == (revision, rustc), "different source/toolchain across platforms"
        pairs = module_inventory(artifacts)
        if all_pairs is None:
            all_pairs = pairs
        assert pairs == all_pairs, "different full module/target inventory across platforms"
        docs = {}
        for os in DOC_OSES:
            whole, outcomes = inventory(artifacts, f"doc-{os}")
            assert all(kind == "doc" for _, kind in whole), "non-doc identity in doc capture"
            for name, outcome in outcomes.items():
                event, reason, _ = outcome
                assert event != "flaky", f"flaky doc conformance: {name} {os}"
                if event == "skipped":
                    assert reason in DOC_SKIP_CATEGORIES, f"unexpected doc skip: {name} {os} {reason}"
            assert any(value[0] == "passed" for value in outcomes.values()), f"no verified doc execution: {os}"
            docs[os] = len(outcomes)
        result[platform] = {"module_pairs": len(pairs), "docs": docs}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("collect")
    check = sub.add_parser("verify")
    check.add_argument("root", type=Path)
    check.add_argument("--tier", choices=["fast", "full"], default="full")
    check.add_argument("--revision", required=True, help="Exact expected CI head SHA")
    args = parser.parse_args()
    if args.command == "collect":
        collect(sys.stdin, emit=lambda value: print(json.dumps(value, sort_keys=True), flush=True))
    else:
        print(json.dumps(verify(args.root, args.tier == "full", args.revision), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
