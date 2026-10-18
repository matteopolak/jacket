"""Diagnostic output tests only; never execute native or networking code."""

import io
import unittest
from contextlib import redirect_stderr

import io_stress


class BurstDiagnostics(unittest.TestCase):
    def test_late_stderr_is_reported_without_dumping_other_output(self):
        problem = (
            "prefix\n" * 100
            + "Io burst failure: backend=poll round=8\nIo burst client: connect=7 code=24\nprivate payload\n"
        )
        stream = io.StringIO()
        with redirect_stderr(stream):
            io_stress.report_burst_diagnostics("tcp [asan] round 2 copy 11", problem)
        self.assertIn("copy 11", stream.getvalue())
        self.assertIn("backend=poll round=8", stream.getvalue())
        self.assertIn("connect=7 code=24", stream.getvalue())
        self.assertNotIn("private payload", stream.getvalue())
        self.assertNotIn("prefix", stream.getvalue())

    def test_unrelated_failure_stays_in_existing_failure_summary(self):
        stream = io.StringIO()
        with redirect_stderr(stream):
            io_stress.report_burst_diagnostics("other", "real assertion failure")
        self.assertEqual(stream.getvalue(), "")
        self.assertIn(
            "real assertion failure", io_stress.short("real assertion failure")
        )


if __name__ == "__main__":
    unittest.main()
