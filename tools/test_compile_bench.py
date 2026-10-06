import unittest

from compile_bench import compare, markdown, parse_timings, summarize


def report(seconds, rss):
    return {'results': {'focus/check': {'warm_seconds': seconds, 'peak_rss_mib': rss}}}


class CompileBenchTests(unittest.TestCase):
    def test_timings_are_parsed_and_the_front_end_excludes_the_backend(self):
        phases = parse_timings('note\njaic-timing: codegen 0.5 2\njaic-timing: link 0.25 1\n'
                               'jaic-timing: front end 3.0 1\njaic-timing: total 3.1 1\n')
        self.assertEqual(phases['codegen'], 0.5)
        self.assertAlmostEqual(phases['front end only'], 2.35)

    def test_first_run_is_cold_and_the_rest_give_the_median(self):
        runs = [{'wall': w, 'rss_mib': r, 'phases': {'total': w}} for w, r in ((5.0, 90), (2.0, 100), (3.0, 80), (2.5, 85))]
        row = summarize(runs)
        self.assertEqual((row['cold_seconds'], row['warm_seconds']), (5.0, 2.5))
        self.assertEqual((row['peak_rss_mib'], row['phases_seconds']['total']), (100, 2.5))

    def test_regressions_need_relative_and_absolute_growth(self):
        old = report(2.0, 500)
        self.assertEqual(compare(old, report(2.1, 510), 0.10, 0.1, 16), [])
        self.assertEqual(len(compare(old, report(2.4, 500), 0.10, 0.1, 16)), 1)
        self.assertEqual(len(compare(old, report(2.0, 600), 0.10, 0.1, 16)), 1)
        # 50% slower but only 0.05 s: noise on a tiny workload.
        self.assertEqual(compare(report(0.1, 50), report(0.15, 50), 0.10, 0.1, 16), [])
        self.assertEqual(compare(old, {'results': {'other/check': {'warm_seconds': 9, 'peak_rss_mib': 9}}},
                                 0.10, 0.1, 16), [])

    def test_markdown_lists_every_workload(self):
        row = {'cold_seconds': 3, 'warm_seconds': 2, 'min_seconds': 2, 'max_seconds': 3, 'peak_rss_mib': 100,
               'phases_seconds': {'front end only': 1.5, 'codegen': 0.4}}
        text = markdown({'date': '2026-10-05T00:00:00', 'machine': {'machine': 'arm64', 'cpus': 10, 'system': 'Darwin',
                                                                  'release': '27'},
                         'jaic': {'commit': 'abc', 'dirty': False, 'rustc': 'rustc'}, 'settings': {'repeat': 3},
                         'results': {'focus/check': row}, 'failures': {'x/check': 'exit 1'}},
                        baseline={'results': {'focus/check': {'warm_seconds': 4, 'peak_rss_mib': 100}}})
        self.assertIn('| focus | check | 3.00 | 2.00 |', text)
        self.assertIn('0.50x time', text)
        self.assertIn('`x/check`: exit 1', text)


if __name__ == '__main__':
    unittest.main()
