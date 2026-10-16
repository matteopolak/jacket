"""Failure-contract policy checks only; no Jai/native/fault execution."""

import subprocess
import unittest

from check_basic_any_read import expected_case_result


class AnyOutcome(unittest.TestCase):
    def test_required_fault_needs_specific_cause_and_exact_prefix(self):
        cause = "cannot format an Any without payload storage"
        good = subprocess.CompletedProcess([], 1, "7\ntrue\n", cause)
        self.assertTrue(expected_case_result(good, True, "7\ntrue\n", cause))
        for result in [
            subprocess.CompletedProcess([], 1, "7\ntrue\n", "missing library"),
            subprocess.CompletedProcess([], 0, "7\ntrue\n", cause),
            subprocess.CompletedProcess([], 1, "7\ntrue\nunreachable\n", cause),
            subprocess.CompletedProcess([], 1, "", cause),
        ]:
            self.assertFalse(expected_case_result(result, True, "7\ntrue\n", cause))

    def test_positive_needs_clean_success_and_exact_output(self):
        self.assertTrue(
            expected_case_result(
                subprocess.CompletedProcess([], 0, "continued\n", ""),
                False,
                "continued\n",
                None,
            )
        )
        self.assertFalse(
            expected_case_result(
                subprocess.CompletedProcess([], 0, "continued\n", "error"),
                False,
                "continued\n",
                None,
            )
        )
        self.assertFalse(
            expected_case_result(
                subprocess.CompletedProcess([], 1, "continued\n", ""),
                False,
                "continued\n",
                None,
            )
        )


if __name__ == "__main__":
    unittest.main()
