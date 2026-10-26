import unittest

from bench.library_compare import LOOPBACK, report, scaled


def result(name: str, estimate: float, items: int = 0) -> dict:
    return {
        'name': name,
        'time_ns': {'estimate': estimate, 'low': estimate * 0.9, 'high': estimate * 1.1},
        'items_per_iteration': items,
    }


class LibraryCompareTests(unittest.TestCase):
    def test_scaled_picks_a_unit(self):
        self.assertEqual(scaled(12.0), '12 ns')
        self.assertEqual(scaled(2500.0), '2.5 us')
        self.assertEqual(scaled(3.0e6), '3 ms')
        self.assertEqual(scaled(2.0e9), '2 s')

    def test_report_groups_by_workload_and_gives_ratios_to_the_first_library(self):
        document = {'results': [
            result('decode__ours', 100.0),
            result('decode__theirs', 250.0),
            result('encode__ours', 10.0),
        ]}
        lines = report(document, ['ours', 'theirs']).splitlines()
        self.assertEqual(lines[0], '| Workload | ours | theirs |')
        self.assertIn('2.50x', lines[2])
        self.assertTrue(lines[3].endswith('| n/a |'))

    def test_report_per_item_divides_by_the_item_count(self):
        document = {'results': [result('insert__ours', 1000.0, items=10)]}
        self.assertIn('100 ns', report(document, ['ours'], per_item=True))

    def test_only_postgres_and_http_may_use_the_loopback_interface(self):
        self.assertEqual(LOOPBACK, {'postgres', 'http'})


if __name__ == '__main__':
    unittest.main()
