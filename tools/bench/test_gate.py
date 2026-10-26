import unittest

from bench import gate


class Parsing(unittest.TestCase):
    def test_allocation_totals_come_from_the_total_line(self):
        stderr = "jaic-alloc: front end 5 10\njaic-alloc: total 7 99\n"
        self.assertEqual(gate.parse_allocations(stderr), (7, 99))
        self.assertIsNone(gate.parse_allocations("nothing"))

    def test_cachegrind_summary(self):
        self.assertEqual(gate.parse_instructions("==1== I   refs:      1,234,567\n"), 1234567)


class Comparison(unittest.TestCase):
    def test_growth_past_the_threshold_regresses(self):
        regressions, improvements, _ = gate.compare({"w": {"alloc_blocks": 1000}}, {"w": {"alloc_blocks": 1030}})
        self.assertEqual(len(regressions), 1)
        self.assertEqual(improvements, [])

    def test_growth_within_the_threshold_passes(self):
        self.assertEqual(gate.compare({"w": {"alloc_blocks": 1000}}, {"w": {"alloc_blocks": 1010}})[0], [])

    def test_a_large_improvement_is_reported_not_failed(self):
        regressions, improvements, _ = gate.compare({"w": {"instructions": 1000}}, {"w": {"instructions": 900}})
        self.assertEqual(regressions, [])
        self.assertEqual(len(improvements), 1)

    def test_missing_entries_on_either_side_fail(self):
        self.assertEqual(len(gate.compare({}, {"w": {"alloc_bytes": 1}})[0]), 1)
        self.assertEqual(len(gate.compare({"gone": {"alloc_bytes": 1}}, {})[0]), 1)

    def test_scaling_the_threshold(self):
        self.assertEqual(gate.compare({"w": {"alloc_blocks": 1000}}, {"w": {"alloc_blocks": 1030}}, 2.0)[0], [])


if __name__ == "__main__":
    unittest.main()
