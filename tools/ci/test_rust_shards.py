"""Pinning of slow Rust tests to shards (tools/ci/rust_shards.py)."""

import unittest

from ci import rust_shards


def tests(**seconds):
    return {("bin", name): value for name, value in seconds.items()}


class PlanShards(unittest.TestCase):
    def test_slow_tests_go_longest_first_onto_the_lightest_shard(self):
        durations = tests(a=100, b=90, c=60, d=50, quick=1)
        pinned = rust_shards.plan_shards(durations, 2, min_seconds=5)
        self.assertEqual(pinned, [[("bin", "a"), ("bin", "d")], [("bin", "b"), ("bin", "c")]])

    def test_quick_tests_are_left_to_slice(self):
        pinned = rust_shards.plan_shards(tests(a=100, q1=1, q2=2), 2, min_seconds=5)
        self.assertEqual(sum(map(len, pinned)), 1)

    def test_slice_deals_the_sorted_tests_round_robin(self):
        loads = rust_shards.slice_loads(tests(a=1, b=2, c=4), 2)
        self.assertEqual(loads, [5, 2])


class ShardFilters(unittest.TestCase):
    plan = {"2": [[["x", "t1"]], [["y", "t2"], ["y", "t3"]]]}

    def test_a_shard_pass_names_its_own_tests_and_the_other_pass_excludes_all_pinned(self):
        own, every = rust_shards.shard_filters("2/2", self.plan)
        self.assertEqual(own, "test(=t2) | test(=t3)")
        for name in ("t1", "t2", "t3"):
            self.assertIn(f"test(={name})", every)

    def test_a_shard_count_without_a_plan_is_dealt_out_by_slice_alone(self):
        self.assertIsNone(rust_shards.shard_filters("1/3", self.plan))
        self.assertIsNone(rust_shards.shard_filters("1/3", {}))


if __name__ == "__main__":
    unittest.main()
