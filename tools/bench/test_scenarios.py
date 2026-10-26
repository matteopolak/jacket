import argparse
import unittest

from bench import cmd_fmt
from bench import cmd_lint
from bench.scenarios import Scenario, measure, render_table, wanted


def args(repeat=2, warmup=1):
    return argparse.Namespace(repeat=repeat, warmup=warmup, timeout=10, only='')


class UnformatTests(unittest.TestCase):
    def test_indentation_and_space_runs_go_but_tokens_stay(self):
        text = 'main :: () {\n    x  :=   1;\n\tif x   > 0 {\n        y = 2;\n    }\n}\n'
        wrecked = cmd_fmt.unformat(text)
        self.assertEqual(wrecked, 'main :: () {\nx := 1;\nif x > 0 {\ny = 2;\n}\n}\n')
        self.assertEqual(wrecked.split(), text.split())

    def test_already_flat_text_is_unchanged(self):
        self.assertEqual(cmd_fmt.unformat('a := 1;\nb := 2;\n'), 'a := 1;\nb := 2;\n')


class ScenarioTests(unittest.TestCase):
    def test_runs_are_counted_after_warmup(self):
        result, last = measure(Scenario('x', ['/bin/echo', 'hi']), args(repeat=3, warmup=2))
        self.assertEqual(result.wall.n, 3)
        self.assertEqual(last.stdout.strip(), 'hi')

    def test_unexpected_exit_status_fails_the_scenario(self):
        result, _ = measure(Scenario('x', ['/bin/sh', '-c', 'exit 1']), args())
        self.assertEqual(result.status.value, 'failed')
        ok, _ = measure(Scenario('x', ['/bin/sh', '-c', 'exit 1'], ok_codes=(0, 1)), args())
        self.assertEqual(ok.status.value, 'ok')

    def test_prepare_runs_before_every_execution(self):
        calls = []
        measure(Scenario('x', ['/bin/echo'], prepare=lambda: calls.append(1)), args(repeat=2, warmup=1))
        self.assertEqual(len(calls), 3)

    def test_heavy_scenarios_need_to_be_named(self):
        heavy = Scenario('repo', ['/bin/echo'], heavy=True)
        self.assertFalse(wanted(heavy, ''))
        self.assertTrue(wanted(heavy, 'repo'))
        self.assertFalse(wanted(heavy, 'other'))
        self.assertTrue(wanted(Scenario('x', ['/bin/echo']), ''))

    def test_table_lists_only_timed_scenarios(self):
        good, _ = measure(Scenario('good', ['/bin/echo']), args())
        bad, _ = measure(Scenario('bad', ['/bin/sh', '-c', 'exit 2']), args())
        table = render_table({'good': good, 'bad': bad})
        self.assertIn('| good |', table)
        self.assertNotIn('| bad |', table)


class LintCaseTests(unittest.TestCase):
    def test_the_densest_case_is_a_real_file(self):
        self.assertTrue(cmd_lint.densest_case().is_file())


if __name__ == '__main__':
    unittest.main()
