import unittest

import compile_report
from results import Report, Result
from stages import Bucket
from stats import Stat


def stat(value):
    return Stat.of([value])


def timed(wall, **stages):
    return Result(wall=stat(wall), cpu=stat(wall), rss_mib=stat(50.0),
                  stages={Bucket(k.replace('_', ' ')).value: stat(v) for k, v in stages.items()},
                  info={'category': 'startup', 'source': {'lines': 2000}})


class RenderTests(unittest.TestCase):
    def report(self):
        results = {
            'hello/check/jaic': timed(0.5, front_end=0.4, total=0.5),
            'hello/check/ref': timed(1.0, total=1.0),
            'big/build/jaic': timed(2.0, total=2.0),
            'big/build/ref': Result.skipped('ref: exit 1: unknown directive'),
        }
        return Report('compile', {'repeat': 3, 'warmup': 1},
                      {'jaic': {'dialect': 'jaic', 'path': '/j'}, 'ref': {'dialect': 'plain', 'path': '/r'}},
                      results, machine={'cpus': 10})

    def test_compilers_get_columns_and_a_ratio_to_the_first(self):
        text = compile_report.render(self.report())
        self.assertIn('ref / jaic', text)
        self.assertIn('| hello | check | 2000 |', text)
        self.assertIn('2.00x', text)

    def test_stage_ratio_row_only_where_both_report_the_bucket(self):
        text = compile_report.render(self.report())
        self.assertIn('| hello/check | ref / jaic | - | - | - | - | - | 2.00x |', text)

    def test_jaic_timings_get_their_own_table_without_ratios(self):
        report = self.report()
        report.results['hello/check/jaic'].detail = {'front end': stat(0.3), 'total': stat(0.5)}
        text = compile_report.render(report)
        detail = text.split('## Compiler-reported stages')[1].split('\n## ')[0]
        self.assertIn('| hello/check | jaic | 0.300 |', detail)
        self.assertNotIn('| ref', detail)
        self.assertNotIn('Compiler-reported', compile_report.render(self.report()))

    def test_skipped_workloads_say_why(self):
        text = compile_report.render(self.report())
        self.assertIn('`big/build/ref`: skipped: ref: exit 1: unknown directive', text)

    def test_split_key(self):
        self.assertEqual(compile_report.split_key('gen-10k/run-build/jaic'), ('gen-10k', 'run-build', 'jaic'))


if __name__ == '__main__':
    unittest.main()
