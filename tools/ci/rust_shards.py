#!/usr/bin/env python3
"""Balance the Rust test shards of the `test` job by measured test durations.

    python3 tools/ci/rust_shards.py plan  JUNIT_DIR [--shards N] [--min-seconds S]
    python3 tools/ci/rust_shards.py show  JUNIT_DIR [--shards N]

`dev.py test --partition slice:K/N` deals the tests out one at a time, so a handful of slow tests
can land in the same shard. `tools/ci/rust-test-shards.json` pins the slow tests to shards instead
(longest first, always onto the lightest shard); every other test is still dealt out by `slice`.
A shard count with no entry in the file is dealt out by `slice` alone, as is any test the file
does not name, so a new test needs no edit here.

`plan` reads the junit files that CI uploads (the `rust-test-durations-*` artifacts of a
passing run on `main`, from `gh run download RUN -p 'rust-test-durations-ubuntu-24.04-*'`; one
directory per shard, each holding `junit.xml` and, when tests are pinned, `junit-pinned.xml`) and rewrites the
entry for `--shards`. `show` prints the load of each shard in those files before and after the pinning.

Plans per count: 6 from Linux x64 (`ubuntu-24.04-*`), 2 from the fast-tier macOS legs, and 3 from
the full tier's Linux arm64, macOS arm64 and macOS x64 legs together (a nightly run), since those
hosts share the count.
"""

from __future__ import annotations

import argparse
import json
import xml.etree.ElementTree as ET
from pathlib import Path

PLAN = Path(__file__).resolve().parent / "rust-test-shards.json"
MIN_SECONDS = 5.0


def read_durations(directory):
    """{(binary id, test name): seconds} from every junit.xml below `directory`. A test seen more than
    once (the full tier runs the same shards on three hosts) keeps its slowest time."""
    durations = {}
    for path in sorted(Path(directory).rglob("junit*.xml")):
        for case in ET.parse(path).getroot().iter("testcase"):
            test = (case.get("classname"), case.get("name"))
            durations[test] = max(durations.get(test, 0.0), float(case.get("time", 0)))
    return durations


def slice_loads(durations, shards, skip=()):
    """The time each shard spends on the tests `slice:K/N` gives it: nextest deals the sorted tests round-robin."""
    loads = [0.0] * shards
    kept = sorted(test for test in durations if test not in skip)
    for index, test in enumerate(kept):
        loads[index % shards] += durations[test]
    return loads


def plan_shards(durations, shards, min_seconds=MIN_SECONDS):
    """Pin the tests of at least `min_seconds` to shards, longest first onto the lightest one.
    Every shard starts with an even share of the rest, which `slice` deals out."""
    heavy = sorted((t for t, d in durations.items() if d >= min_seconds), key=lambda t: (-durations[t], t))
    rest = sum(d for t, d in durations.items() if t not in set(heavy))
    loads = [rest / shards] * shards
    pinned = [[] for _ in range(shards)]
    for test in heavy:
        lightest = min(range(shards), key=lambda i: (loads[i], i))
        pinned[lightest].append(test)
        loads[lightest] += durations[test]
    return [sorted(tests) for tests in pinned]


def pinned_load(durations, pinned, shards):
    heavy = {t for tests in pinned for t in tests}
    tail = slice_loads(durations, shards, heavy)
    return [sum(durations[t] for t in pinned[i]) + tail[i] for i in range(shards)]


def load_plan(path=PLAN):
    return json.loads(Path(path).read_text()) if Path(path).is_file() else {}


def test_filter(tests):
    """A nextest filterset naming exactly `tests`."""
    return " | ".join(f"(binary_id(={binary}) & test(={name}))" for binary, name in tests)


def shard_filters(shard, plan=None):
    """For `K/N`: (the filterset of the tests pinned to shard K, the filterset of every unpinned
    test) or None when the plan has nothing for N shards."""
    plan = load_plan() if plan is None else plan
    index, count = (int(part) for part in shard.split("/"))
    pins = plan.get(str(count))
    if not pins or not any(pins):
        return None
    everything = [tuple(test) for tests in pins for test in tests]
    return test_filter(tuple(t) for t in pins[index - 1]), test_filter(everything)


def spread(loads):
    return f"max {max(loads):.0f} s, min {min(loads):.0f} s, max/mean {max(loads) / (sum(loads) / len(loads)):.2f}"


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("command", choices=("plan", "show"))
    parser.add_argument("junit", help="directory holding the downloaded junit.xml files")
    parser.add_argument("--shards", type=int, default=6)
    parser.add_argument("--min-seconds", type=float, default=MIN_SECONDS)
    args = parser.parse_args()
    durations = read_durations(args.junit)
    if not durations:
        raise SystemExit(f"no junit.xml below {args.junit}")
    pinned = plan_shards(durations, args.shards, args.min_seconds)
    print(f"{len(durations)} tests, {sum(durations.values()):.0f} s in total")
    print("slice only:", " ".join(f"{x:.0f}" for x in slice_loads(durations, args.shards)), "|", spread(slice_loads(durations, args.shards)))
    after = pinned_load(durations, pinned, args.shards)
    print("pinned:    ", " ".join(f"{x:.0f}" for x in after), "|", spread(after))
    if args.command == "plan":
        plan = load_plan()
        plan[str(args.shards)] = pinned
        PLAN.write_text(json.dumps(plan, indent=1, sort_keys=True) + "\n")
        print(f"wrote {PLAN} ({sum(map(len, pinned))} pinned tests)")


if __name__ == "__main__":
    main()
