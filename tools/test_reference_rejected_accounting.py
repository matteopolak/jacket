"""Catalogue reporting control only; no compiler/program execution."""

import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import stdlib_runtime as runtime


class ReferenceAccounting(unittest.TestCase):
    def test_rejected_positive_is_separate_from_pass_and_platform(self):
        with tempfile.TemporaryDirectory(
            dir=os.environ["CARGO_TARGET_DIR"]
        ) as directory:
            report = Path(directory) / "report.json"
            case = {
                "id": "compiler/proc-type-defaults-ifx",
                "family": "compiler",
                "path": runtime.ROOT
                / "tests/compiler-regressions/proc-type-defaults-ifx.jai",
            }
            argv = [
                "stdlib_runtime",
                "--jaic",
                __file__,
                "--modes",
                "interp",
                "--case",
                case["id"],
                "--report",
                str(report),
            ]
            with (
                patch("sys.argv", argv),
                patch.object(runtime, "validated_entries", return_value=[case]),
                patch.object(runtime, "run_mode") as run,
                patch.object(runtime, "run_command") as command,
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(runtime.main(), 0)
            run.assert_not_called()
            command.assert_not_called()
            data = json.loads(report.read_text())
            self.assertEqual(
                data["counts"]["interp"],
                {
                    "passed": 0,
                    "failed": 0,
                    "platform_unavailable": 0,
                    "compile_time_only": 0,
                    "expected_negative_checks": 0,
                    "reference_rejected": 1,
                },
            )
            row = data["results"][0]
            self.assertIsNone(row["passed"])
            self.assertIsNone(row["platform_unavailable"])
            self.assertIsNotNone(row["reference_rejected"])


if __name__ == "__main__":
    unittest.main()
