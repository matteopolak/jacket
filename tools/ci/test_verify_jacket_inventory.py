from functools import partial
import json
import tempfile
import unittest
from pathlib import Path

from verify_jacket_inventory import compare as compare_runs

compare = partial(compare_runs, baseline_revision="a" * 40, candidate_revision="a" * 40, expected_platforms={"macos-arm64"})


class Coverage(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def write(self, tree, part, whole, assigned, outcomes):
        directory = self.root / tree / f"jacket-inventory-macos-arm64-{part}"
        directory.mkdir(parents=True)
        (directory / "metadata.json").write_text(json.dumps({"revision": "a" * 40, "partition": "all", "os": "Darwin", "arch": "arm64", "rustc": "public pinned version"}))
        for mode in ("native", "interp", "o2"):
            for suffix, records in [("all", whole), ("part", assigned), ("outcomes", outcomes)]:
                (directory / f"{mode}-{suffix}.jsonl").write_text("".join(json.dumps(r) + "\n" for r in records))

    def test_empty_and_single_case_disjoint_union_and_outcomes(self):
        for cases in ([], [{"name": "one", "kind": "test"}]):
            with self.subTest(cases=cases):
                self.root = Path(self.tmp.name) / str(len(cases))
                outcomes = [{**c, "type": "test", "event": "passed"} for c in cases]
                finished = {"type": "run", "event": "finished", "failed": 0}
                self.write("old", 0, cases, cases, [*outcomes, finished])
                self.write("new", 0, cases, [], [finished])
                self.write("new", 1, cases, cases, [*outcomes, finished])
                if not cases:
                    with self.assertRaisesRegex(AssertionError, "empty"):
                        compare(self.root / "old", self.root / "new")
                else:
                    result = compare(self.root / "old", self.root / "new")
                    self.assertEqual(result["macos-arm64"]["native"]["discovered"], len(cases))

    def test_overlap_and_missing_outcome_are_rejected(self):
        case = {"name": "one", "kind": "test"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        outcomes = [{**case, "type": "test", "event": "passed"}, finished]
        self.write("old", 0, [case], [case], outcomes)
        self.write("new", 0, [case], [case], [finished])
        with self.assertRaisesRegex(AssertionError, "case outcomes"):
            compare(self.root / "old", self.root / "new")
        self.write("duplicate", 0, [case], [case], outcomes)
        self.write("duplicate", 1, [case], [case], outcomes)
        with self.assertRaisesRegex(AssertionError, "overlapping"):
            compare(self.root / "old", self.root / "duplicate")

    def test_progress_notice_cannot_replace_a_completed_case(self):
        case = {"name": "one", "kind": "test"}
        finished = {"type": "run", "event": "finished", "failed": 0, "passed": 1}
        passed = {**case, "type": "test", "event": "passed"}
        progress = {"type": "test", "event": "slow", "name": "one", "seconds": 1.0}
        self.write("old", 0, [case], [case], [passed, finished])
        self.write("new", 0, [case], [case], [progress, finished])
        with self.assertRaisesRegex(AssertionError, "assigned case outcomes"):
            compare(self.root / "old", self.root / "new")

    def test_equal_count_cross_share_execution_is_rejected(self):
        cases = [{"name": name, "kind": "test"} for name in ["one", "two"]]
        finished = {"type": "run", "event": "finished", "failed": 0}
        outcomes = [{**case, "type": "test", "event": "passed"} for case in cases]
        self.write("old", 0, cases, cases, [*outcomes, finished])
        self.write("new", 0, cases, cases[:1], [outcomes[1], finished])
        self.write("new", 1, cases, cases[1:], [outcomes[0], finished])
        with self.assertRaisesRegex(AssertionError, "assigned case outcomes"):
            compare(self.root / "old", self.root / "new")

    def test_duplicate_discovery_is_not_silently_collapsed(self):
        case = {"name": "one", "kind": "bench"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        self.write("old", 0, [case, case], [case, case], [finished])
        self.write("new", 0, [case, case], [case, case], [finished])
        with self.assertRaisesRegex(AssertionError, "duplicate full discovery"):
            compare(self.root / "old", self.root / "new")

    def test_lost_or_changed_completed_children_are_rejected(self):
        parent = {"name": "M::one", "kind": "test"}
        child = {"name": "M::one/a", "kind": "subtest"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        before = {**parent, "type": "test", "event": "passed", "subtests": [{"name": "one/a", "status": "pass"}]}
        self.write("old", 0, [parent, child], [parent, child], [before, finished])
        for tree, children in [("missing", []), ("changed", [{"name": "one/a", "status": "skip"}]), ("different", [{"name": "one/b", "status": "pass"}])]:
            with self.subTest(tree=tree):
                self.write(tree, 0, [parent, child], [parent, child], [{**before, "subtests": children}, finished])
                with self.assertRaises(AssertionError):
                    compare(self.root / "old", self.root / tree)

    def test_identically_missing_discovered_children_are_rejected(self):
        parent = {"name": "M::one", "kind": "test"}
        child = {"name": "M::one/a", "kind": "subtest"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        outcome = {**parent, "type": "test", "event": "passed"}
        for tree in ["old", "new"]:
            self.write(tree, 0, [parent, child], [parent, child], [outcome, finished])
        with self.assertRaisesRegex(AssertionError, "missing discovered child"):
            compare(self.root / "old", self.root / "new")

    def test_ambiguous_parent_names_are_not_silently_collapsed(self):
        parents = [{"name": "one", "kind": kind} for kind in ["test", "doc"]]
        child = {"name": "one/a", "kind": "subtest"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        outcomes = [{**parent, "type": "test", "event": "passed", "subtests": [{"name": "one/a", "status": "pass"}]} for parent in parents]
        for tree in ["old", "new"]:
            self.write(tree, 0, [*parents, child], [*parents, child], [*outcomes, finished])
        with self.assertRaisesRegex(AssertionError, "ambiguous parent"):
            compare(self.root / "old", self.root / "new")


    def test_stale_heads_and_identically_missing_expected_platform_fail_closed(self):
        case = {"name": "one", "kind": "test"}
        finished = {"type": "run", "event": "finished", "failed": 0}
        outcomes = [{**case, "type": "test", "event": "passed"}, finished]
        for tree in ("old", "new"):
            self.write(tree, 0, [case], [case], outcomes)
        with self.assertRaisesRegex(AssertionError, "stale source"):
            compare_runs(self.root / "old", self.root / "new", baseline_revision="a" * 40, candidate_revision="b" * 40, expected_platforms={"macos-arm64"})
        with self.assertRaisesRegex(AssertionError, "platform"):
            compare_runs(self.root / "old", self.root / "new", baseline_revision="a" * 40, candidate_revision="a" * 40, expected_platforms={"macos-arm64", "macos-x64"})
