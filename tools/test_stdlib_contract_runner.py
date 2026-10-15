"""Negative acceptance must prove the intended diagnostic and valid public import."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("contract_runner", Path(__file__).resolve().parents[1] / "tests/stdlib-contract/run.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ContractRunner(unittest.TestCase):
    def test_negative_requires_failure_and_precise_diagnostic(self):
        case = {"diagnostic_contains": "has no exported member `HASH_INIT`"}
        self.assertFalse(runner.case_passes(case, "check-fail", 0, "", case["diagnostic_contains"]))
        self.assertFalse(runner.case_passes(case, "check-fail", 1, "", "module Hash not found"))
        self.assertTrue(runner.case_passes(case, "check-fail", 1, "", "error: " + case["diagnostic_contains"]))
        with self.assertRaises(ValueError):
            runner.case_passes({}, "check-fail", 1, "", "error")

    def test_selecting_privacy_also_selects_public_baseline(self):
        baseline = {"name": "public", "mode": "interp"}
        private = {"name": "private", "mode": "check-fail", "positive_baseline": "public"}
        self.assertEqual(runner.selected_cases([baseline, private], ["private"]), [baseline, private])
        self.assertEqual(runner.selected_cases([private, baseline], ["private"]), [baseline, private])
        with self.assertRaises(ValueError):
            runner.selected_cases([private], ["private"])
        with self.assertRaises(ValueError):
            runner.selected_cases([baseline, baseline], None)


if __name__ == "__main__":
    unittest.main()
