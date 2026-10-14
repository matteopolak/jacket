"""Runner protocol tests; synthetic workers are never corpus coverage evidence."""
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest

from lsp_partial_edits import ROOT, run_file


class WorkerProtocol(unittest.TestCase):
    def setUp(self):
        directory = ROOT / "artifacts/lsp-edits/runner-tests"
        directory.mkdir(parents=True, exist_ok=True)
        self.scratch = tempfile.TemporaryDirectory(dir=directory)
        self.addCleanup(self.scratch.cleanup)
        self.file = Path(self.scratch.name) / "worker.py"
        self.args = SimpleNamespace(stdlib=None, max_lines=1, timeout=1)

    def run_worker(self, source):
        self.file.write_text(source)
        return run_file(sys.executable, self.file, self.args)

    def final_line(self, **changes):
        report = dict(file=str(self.file), failures=[], edits=1, assertions=2)
        report.update(changes)
        return "print(" + repr(json.dumps(report)) + ")\n"

    def test_valid_progress_and_final_report(self):
        report = self.run_worker('print(\'{"progress":"hover"}\')\n' + self.final_line())
        self.assertEqual(report["failures"], [])
        self.assertEqual(report["assertions"], 2)

    def test_invalid_stdout_is_a_failure_even_with_a_final_report(self):
        report = self.run_worker('print("not JSON")\n' + self.final_line())
        self.assertTrue(report["failures"])

    def test_invalid_report_shapes_are_failures(self):
        for line in ["[]", "null", "{}", '{"progress": 12}']:
            with self.subTest(line=line):
                report = self.run_worker("print(" + repr(line) + ")\n")
                self.assertTrue(report["failures"])
                self.assertIsNone(report["edits"])

    def test_fabricated_counts_and_wrong_file_are_rejected(self):
        for changes in [dict(edits=-1), dict(assertions=True), dict(file="another-file")]:
            with self.subTest(changes=changes):
                report = self.run_worker(self.final_line(**changes))
                self.assertTrue(report["failures"])
                self.assertIsNone(report["edits"])

    def test_nonzero_exit_is_a_failure_even_with_a_final_report(self):
        report = self.run_worker(self.final_line() + "raise SystemExit(2)\n")
        self.assertTrue(report["failures"])

    def test_progress_resets_the_stage_deadline(self):
        report = self.run_worker(
            "import time\n"
            "for _ in range(3):\n"
            "    print('{\"progress\":\"hover\"}', flush=True)\n"
            "    time.sleep(0.6)\n" + self.final_line()
        )
        self.assertEqual(report["failures"], [])

    def test_invalid_output_does_not_reset_a_stalled_stage(self):
        self.args.timeout = 0.2
        report = self.run_worker(
            "import time\n"
            "for _ in range(20):\n"
            "    print('invalid', flush=True)\n"
            "    time.sleep(0.05)\n" + self.final_line()
        )
        self.assertTrue(any("stalled" in failure for failure in report["failures"]))
        self.assertIsNone(report["edits"])

    def test_stall_is_killed_and_counts_are_unknown(self):
        report = self.run_worker("import time\ntime.sleep(30)\n")
        self.assertTrue(report["failures"])
        self.assertIn("stalled", report["failures"][0])
        self.assertIsNone(report["edits"])
        self.assertIsNone(report["assertions"])


if __name__ == "__main__":
    unittest.main()
