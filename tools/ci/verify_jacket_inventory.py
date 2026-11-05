#!/usr/bin/env python3
"""Compare downloaded jacket inventories before accepting a CI partition change.

    python3 tools/ci/verify_jacket_inventory.py BASELINE_DIR CANDIDATE_DIR \
        --baseline-revision BASELINE_SHA --candidate-revision CANDIDATE_SHA \
        --platform macos-arm64 --platform macos-x64

Each directory contains the extracted jacket-inventory-PLATFORM-JOB artifacts. Lists
prove assignment only; completed per-mode case outcomes separately prove execution.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

MODES = ("native", "interp", "o2", "wasm-interp", "wasm-native")
EVENTS = {"passed", "failed", "skipped", "xfail", "xpass", "timed_out", "flaky"}


def records(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def identity(case: dict) -> tuple[str, str]:
    return case["name"], case["kind"]


def inventory(artifacts: list[Path], mode: str) -> tuple[set, dict]:
    """Require a disjoint, complete discovery partition and one completed outcome per case."""
    whole = None
    assigned = set()
    outcomes = {}
    for artifact in artifacts:
        full_records = records(artifact / f"{mode}-all.jsonl")
        listed = {identity(c) for c in full_records}
        assert len(listed) == len(full_records), f"duplicate full discovery identities: {artifact} {mode}"
        if whole is None:
            whole = listed
        assert listed == whole, f"different full catalogues: {artifact} {mode}"
        part_records = records(artifact / f"{mode}-part.jsonl")
        part = {identity(c) for c in part_records}
        assert len(part) == len(part_records), f"duplicate assigned discovery identities: {artifact} {mode}"
        assert not assigned & part, f"overlapping discovery partition: {artifact} {mode}"
        assigned |= part
        actual = records(artifact / f"{mode}-outcomes.jsonl")
        finished = [c for c in actual if c.get("type") == "run" and c.get("event") == "finished"]
        assert len(finished) == 1, f"incomplete execution: {artifact} {mode}"
        assert not any(finished[0].get(k, 0) for k in ("failed", "timed_out", "not_run", "xpass")), finished
        local_outcomes = set()
        for case in actual:
            if case.get("type") != "test" or case.get("event") not in EVENTS:
                continue
            key = identity(case)
            assert key not in outcomes, f"duplicate execution: {key} {mode}"
            local_outcomes.add(key)
            # Reporter child names are local to their containing module; discovery
            # names include the module prefix. Keep duplicate children losslessly.
            prefix, _, local = key[0].rpartition("::")
            children = []
            for child in case.get("subtests", []):
                name = child["name"]
                if prefix and name.startswith(local + "/"):
                    name = prefix + "::" + name
                children.append((name, child["status"]))
            outcomes[key] = (case["event"], case.get("reason_category"), tuple(sorted(children)))
        expected_part = {key for key in part if key[1] not in {"bench", "subtest"}}
        assert local_outcomes == expected_part, f"missing or unexpected assigned case outcomes: {artifact} {mode}"
    assert whole is not None and assigned == whole, f"incomplete discovery union: {mode}"
    assert whole, f"empty discovery inventory: {mode}"
    expected = {key for key in whole if key[1] not in {"bench", "subtest"}}
    assert expected, f"empty executable inventory: {mode}"
    assert set(outcomes) == expected, f"missing or unexpected case outcomes: {mode}"
    parents = {key[0]: key for key in expected}
    assert len(parents) == len(expected), f"ambiguous parent discovery identities: {mode}"
    for name, kind in whole:
        if kind != "subtest":
            continue
        parent = name
        while "/" in parent:
            parent = parent.rsplit("/", 1)[0]
            if parent in parents:
                outcome = outcomes[parents[parent]]
                if outcome[0] in {"passed", "flaky"}:
                    assert any(child == name for child, _ in outcome[2]), f"missing discovered child: {name} {mode}"
                break
        else:
            raise AssertionError(f"subtest has no discovered parent: {name} {mode}")
    return whole, outcomes


def compare(baseline: Path, candidate: Path, *, baseline_revision: str, candidate_revision: str, expected_platforms: set[str]) -> dict:
    """Compare explicit heads and platform scope, rejecting stale or vacuous coverage proof."""
    assert all(re.fullmatch(r"[0-9a-f]{40}", value) for value in (baseline_revision, candidate_revision)), "invalid expected revision"
    assert expected_platforms, "empty expected platform scope"
    groups = []
    for directory in (baseline, candidate):
        platforms = {}
        for artifact in directory.glob("jacket-inventory-*"):
            platform = artifact.name.removeprefix("jacket-inventory-").rsplit("-", 1)[0]
            platforms.setdefault(platform, []).append(artifact)
        groups.append(platforms)
    assert groups[0] and groups[0].keys() == groups[1].keys() == expected_platforms, "platform coverage changed or missing"
    result = {}
    for platform in groups[0]:
        result[platform] = {}
        environments = []
        for group, expected_revision in zip(groups, (baseline_revision, candidate_revision)):
            versions = [json.loads((artifact / "metadata.json").read_text()) for artifact in group[platform]]
            assert {v["revision"] for v in versions} == {expected_revision}, f"stale source revision: {platform}"
            environments.append({(v["os"], v["arch"], v["rustc"]) for v in versions})
        assert len(environments[0]) == 1 and environments[0] == environments[1], f"runner/toolchain changed: {platform}"
        for mode in MODES:
            present = [(a / f"{mode}-all.jsonl").exists() for g in groups for a in g[platform]]
            required = mode in MODES[:3] or platform == "linux-x64"
            assert any(present) == required, f"missing or unexpected mode: {platform} {mode}"
            if not required:
                continue
            assert all(present), f"mode coverage changed: {platform} {mode}"
            old, new = (inventory(g[platform], mode) for g in groups)
            assert old == new, f"discovery or execution changed: {platform} {mode}"
            result[platform][mode] = {"discovered": len(old[0]), "executed_or_skipped": len(old[1])}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--baseline-revision", required=True)
    parser.add_argument("--candidate-revision", required=True)
    parser.add_argument("--platform", action="append", required=True, help="Expected platform; repeat for every required platform")
    args = parser.parse_args()
    print(json.dumps(compare(args.baseline, args.candidate, baseline_revision=args.baseline_revision,
                             candidate_revision=args.candidate_revision, expected_platforms=set(args.platform)), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
