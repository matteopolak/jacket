"""Independent output-oracle regressions; no compiler/application execution.

Use an explicitly selected accepted/staged runner, not this older producer tree.
"""

import importlib.util
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch


class SummaryContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "acceptance_closure_runner", os.environ["UPSTREAM_CLOSURE_TEST_SOURCE"]
        )
        cls.runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.runner)

    def contract(self):
        return {
            "summary_counts": {
                "Passed": {"count": 6, "percent": 100},
                "Failed": {"count": 0, "percent": 0},
            }
        }

    def test_documented_counts_allow_only_formatting_differences(self):
        for text in [
            "Passed: 6 (100%)\nFailed: 0 (0%)\n",
            "Passed: 6 (100%)\nFailed: 0 ( 0%)\n",
            " Passed:  6 ( 100.0 % )\r\n Failed:\t0 (\t0.0%)\r\n",
        ]:
            with self.subTest(text=text):
                self.runner.check_output(text, self.contract(), "xml")

    def test_real_failures_or_changed_test_totals_remain_failures(self):
        for text in [
            "Passed: 6 (100%)\nFailed: 1 (0%)\n",
            "Passed: 5 (100%)\nFailed: 0 (0%)\n",
            "Passed: 6 (99%)\nFailed: 0 (0%)\n",
            "Passed: 6 (100%)\nFailed: 0 (1%)\n",
            "Passed: 6 (100%)\nFailed: -1 (0%)\n",
            "Passed: 6 (100%)\nFailed: 0 (NaN%)\n",
        ]:
            with self.subTest(text=text), self.assertRaises(self.runner.Failure):
                self.runner.check_output(text, self.contract(), "xml")

    def test_missing_partial_embedded_or_conflicting_summary_is_not_success(self):
        for text in [
            "",
            "Passed: 6 (100%)\n",
            "Failed: 0 (0%)\n",
            "example: Passed: 6 (100%)\nFailed: 0 (0%)\n",
            "Passed: 6 (100%)\nFailed: 0 (0%) ignored failures\n",
            "Passed: 6 (100%)\nFailed: 0 (0%)\nFailed: 3 (50%)\n",
        ]:
            with self.subTest(text=text), self.assertRaises(self.runner.Failure):
                self.runner.check_output(text, self.contract(), "xml")

    def test_other_output_assertions_remain_required(self):
        text = "Passed: 6 (100%)\nFailed: 0 ( 0%)\n"
        for extra in [
            {"contains": ["required marker"]},
            {"not_contains": ["Passed"]},
            {"ordered": ["Failed", "Passed"]},
        ]:
            with self.subTest(extra=extra), self.assertRaises(self.runner.Failure):
                self.runner.check_output(text, {**self.contract(), **extra}, "xml")

    def test_nonzero_process_status_remains_failure_despite_zero_failure_summary(self):
        step = {"argv": ["not-executed"], **self.contract()}
        ctx = {"jaic": "not-executed", "cwd": Path.cwd(), "work": Path.cwd(), "env": {}}
        result = SimpleNamespace(
            returncode=7, stdout="Passed: 6 (100%)\nFailed: 0 (0%)\n", stderr=""
        )
        with patch.object(self.runner.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(self.runner.Failure, "exit 7, expected 0"):
                self.runner.run_step(step, ctx)


if __name__ == "__main__":
    unittest.main()
