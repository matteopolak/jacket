"""Cross-build rejection accounting only; no Windows runtime claim."""

import contextlib
import io
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import windows_cross


class ContractStreams(unittest.TestCase):
    def run_case(self, case, replies, baseline=False):
        with tempfile.TemporaryDirectory() as scratch:
            args = SimpleNamespace(
                out=scratch,
                stdlib=True,
                host=True,
                jaic=__file__,
                cpu="arm64",
                keep_going=False,
            )
            entries = [dict(case, id="negative", path=Path(__file__))]
            if baseline:
                entries[0]["positive_baseline"] = "positive"
                entries.append(dict(id="contracts/positive", path=Path(__file__)))
            output = io.StringIO()
            with (
                patch.object(
                    windows_cross.stdlib_runtime,
                    "validated_entries",
                    return_value=entries,
                ),
                patch.object(
                    windows_cross.stdlib_runtime, "run_command", side_effect=replies
                ),
                patch.object(windows_cross, "cases", return_value=[]),
                patch.object(windows_cross.shutil, "copytree"),
                patch.object(windows_cross, "c_structs_library", return_value=None),
                patch.object(windows_cross, "build_one", return_value=None),
                patch.object(windows_cross, "C_STRUCTS", {}),
                contextlib.redirect_stdout(output),
            ):
                code = windows_cross.build(args)
            return code, output.getvalue()

    def test_compile_diagnostic_in_stderr(self):
        self.assertEqual(
            self.run_case(
                dict(mode="check-fail", diagnostic_contains="owned"), [(1, "", "owned")]
            )[0],
            0,
        )

    def test_interpreter_runtime_diagnostic_in_stderr(self):
        self.assertEqual(
            self.run_case(
                dict(error_contains="owned", exit_code=1), [(1, "", "owned")]
            )[0],
            0,
        )

    def test_success_status_cannot_satisfy_rejection(self):
        self.assertEqual(
            self.run_case(
                dict(mode="check-fail", diagnostic_contains="owned"), [(0, "", "owned")]
            )[0],
            1,
        )

    def test_negative_timeout_is_not_rejection(self):
        code, output = self.run_case(
            dict(mode="check-fail", diagnostic_contains="owned"),
            [(None, "out", "owned")],
        )
        self.assertEqual(code, 1)
        self.assertIn("timeout", output)
        self.assertIn("outowned", output)

    def test_baseline_stderr_and_exit_are_retained(self):
        code, output = self.run_case(
            dict(mode="check-fail", diagnostic_contains="owned"),
            [(1, "", "owned"), (2, "baseline-out", "baseline-error")],
            True,
        )
        self.assertEqual(code, 1)
        self.assertIn("positive baseline failed (exit 2)", output)
        self.assertIn("baseline-outbaseline-error", output)

    def test_baseline_timeout_is_not_success(self):
        code, output = self.run_case(
            dict(mode="check-fail", diagnostic_contains="owned"),
            [(1, "", "owned"), (None, "", "waiting")],
            True,
        )
        self.assertEqual(code, 1)
        self.assertIn("positive baseline failed (timeout)", output)


if __name__ == "__main__":
    unittest.main()
