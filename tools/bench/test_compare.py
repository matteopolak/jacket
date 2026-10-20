import json
import tempfile
import unittest
from pathlib import Path

from compare import Limits, find_regressions
from results import Report, Result, Status, load_report
from stats import Stat


def result(wall, rss=100.0, mad=0.0, cpu=None):
    def stat(value, spread=0.0):
        return Stat(n=5, median=value, min=value - spread, max=value + spread, mad=spread, p95=value + spread)
    return Result(wall=stat(wall, mad), cpu=stat(cpu if cpu is not None else wall, mad), rss_mib=stat(rss))


def report(**results):
    return Report('compile', {}, results={key.replace('__', '/'): value for key, value in results.items()}, machine={})


LIMITS = Limits(threshold=0.10, min_seconds=0.10, min_mib=16)


class RegressionTests(unittest.TestCase):
    def regressions(self, before, after, limits=LIMITS):
        return [str(r) for r in find_regressions(report(a__b=before), report(a__b=after), limits)]

    def test_growth_needs_relative_and_absolute_size(self):
        old = result(2.0)
        self.assertEqual(self.regressions(old, result(2.1)), [])
        self.assertEqual(len(self.regressions(old, result(2.5))), 2)  # wall and cpu
        # 50% slower but only 0.05 s: noise on a tiny workload.
        self.assertEqual(self.regressions(result(0.1), result(0.15)), [])

    def test_memory_regression(self):
        found = self.regressions(result(1.0, rss=500), result(1.0, rss=700))
        self.assertEqual(len(found), 1)
        self.assertIn('rss_mib', found[0])
        self.assertEqual(self.regressions(result(1.0, rss=500), result(1.0, rss=510)), [])

    def test_noisy_measurements_do_not_regress(self):
        # The medians differ by 0.5 s, but each run wandered by about 0.3 s: not distinguishable.
        self.assertEqual(self.regressions(result(2.0, mad=0.3), result(2.5, mad=0.3)), [])
        self.assertNotEqual(self.regressions(result(2.0, mad=0.01), result(2.5, mad=0.01)), [])

    def test_interpreted_instruction_counts_regress_without_a_floor(self):
        def counted(count):
            r = result(1.0)
            r.info['interp_instructions'] = count
            return r
        found = self.regressions(counted(100_000_000), counted(120_000_000))
        self.assertEqual(len(found), 1)
        self.assertIn('interp_instructions', found[0])
        self.assertEqual(self.regressions(counted(100_000_000), counted(105_000_000)), [])

    def test_faster_is_fine(self):
        self.assertEqual(self.regressions(result(2.0), result(1.0)), [])

    def test_something_that_stopped_working_regresses(self):
        found = self.regressions(result(1.0), Result.failed('exit 1: boom'))
        self.assertEqual(len(found), 1)
        self.assertIn('failed', found[0])

    def test_skipped_before_or_after_is_not_compared(self):
        self.assertEqual(self.regressions(Result.skipped('no'), result(5.0)), [])
        self.assertEqual(self.regressions(result(1.0), Result.skipped('the other build rejects it')), [])

    def test_keys_in_only_one_report_are_ignored(self):
        old = report(a__b=result(1.0))
        new = report(c__d=result(9.0))
        self.assertEqual(find_regressions(old, new, LIMITS), [])

    def test_report_survives_json(self):
        original = report(a__b=result(1.5), c__d=Result.skipped('why'))
        again = Report.from_json(original.to_json())
        self.assertEqual(again.results['a/b'].wall, original.results['a/b'].wall)
        self.assertEqual(again.results['c/d'].status, Status.SKIPPED)
        self.assertEqual(again.results['c/d'].reason, 'why')


class LoadReportTests(unittest.TestCase):
    def test_a_report_in_an_older_format_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'old.json'
            path.write_text('{"format": 1, "jaic": {}, "results": {}}')
            with self.assertRaisesRegex(ValueError, 'format 3 .*found format 1'):
                load_report(path)

    def test_a_report_written_now_loads(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'new.json'
            path.write_text(json.dumps(Report('compile', {}).to_json()))
            self.assertEqual(load_report(path).tool, 'compile')


if __name__ == '__main__':
    unittest.main()
