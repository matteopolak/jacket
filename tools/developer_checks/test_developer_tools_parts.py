import unittest

import check_developer_tools as runner


class DeveloperToolsParts(unittest.TestCase):
    def setUp(self):
        self.units = runner.units()
        self.names = [unit.name for unit in self.units]
        self.costs = runner.load_costs()

    def test_unit_names_are_unique(self):
        self.assertEqual(len(self.names), len(set(self.names)))

    def test_every_manifest_check_and_case_is_a_unit(self):
        self.assertEqual({n for n in self.names if n.startswith("manifest:")},
                         {"manifest:" + e["module"] for e in runner.manifest_checks()})
        grouped = [case for group in runner.case_groups() for case in group]
        self.assertEqual(grouped, runner.CASES)
        self.assertEqual({n for n in self.names if n.startswith("case:")},
                         {"case:" + group[0] for group in runner.case_groups()})

    def test_cost_list_names_exactly_the_units(self):
        self.assertEqual(set(self.costs), set(self.names), "re-measure: check_developer_tools.py --timings FILE, then update tooling_part_costs.json")

    def test_parts_cover_every_unit_exactly_once(self):
        for count in range(1, 9):
            parts = runner.assign_parts(self.units, count, self.costs)
            self.assertEqual(len(parts), count)
            flat = [name for part in parts for name in part]
            self.assertEqual(sorted(flat), sorted(self.names), f"N={count}")
            self.assertEqual(len(flat), len(set(flat)), f"N={count}")

    def test_assignment_is_deterministic_and_keeps_suite_order(self):
        order = {name: index for index, name in enumerate(self.names)}
        first = runner.assign_parts(self.units, 4, self.costs)
        self.assertEqual(first, runner.assign_parts(runner.units(), 4, self.costs))
        for part in first:
            self.assertEqual(part, sorted(part, key=order.__getitem__))

    def test_units_that_read_another_units_output_share_its_part(self):
        for count in range(1, 9):
            parts = runner.assign_parts(self.units, count, self.costs)
            home = {name: index for index, part in enumerate(parts) for name in part}
            for unit in self.units:
                for other in unit.after:
                    self.assertEqual(home[unit.name], home[other], f"{unit.name} after {other}, N={count}")

    def test_fixtures_and_prerequisites_exist(self):
        for unit in self.units:
            for name in (*unit.fixtures, *unit.after):
                self.assertIn(name, self.names)

    def test_part_argument(self):
        self.assertEqual(runner.parse_part("2/3"), (2, 3))
        for bad in ["0/3", "4/3", "x", "1/", "/2"]:
            with self.assertRaises(Exception):
                runner.parse_part(bad)


if __name__ == "__main__":
    unittest.main()
