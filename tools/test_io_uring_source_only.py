"""Source-only intent/dispatch/accounting checks; no Linux syscall or fault run."""

import argparse
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from ci_test_scratch import assigned_scratch
import stdlib_runtime as runtime
from stdlib_test_catalog import validated_entries

ROOT = Path(__file__).resolve().parents[1]


class IoUringSourceOnly(unittest.TestCase):
    def test_only_authored_source_reducer_gets_source_only_metadata(self):
        cases = {c["id"]: c for c in validated_entries(ROOT)}
        self.assertIs(cases["Extensions/Io_Uring/tests/dependency"].get("source_check_only"), True)
        for name in ("raw", "protocol", "initialized-nop"):
            self.assertNotIn("source_check_only", cases["Extensions/Io_Uring/tests/" + name])

    def test_source_only_never_builds_or_executes_main(self):
        args = argparse.Namespace(jaic="metadata-only", timeout=12, coverage=None, platform="linux-x64")
        path = ROOT / "stdlib/Extensions/Io_Uring/tests/dependency.jai"
        for mode in runtime.MODES:
            with (
                self.subTest(mode=mode),
                patch.object(runtime, "run_command", return_value=(0, "", "")) as check,
                patch.object(
                    runtime,
                    "run_program",
                    side_effect=AssertionError("main must never run"),
                ),
            ):
                passed, _ = runtime.run_mode(
                    args,
                    "source-control",
                    path,
                    mode,
                    assigned_scratch(),
                    {"source_check_only": True},
                )
                self.assertIsNone(passed)
                self.assertEqual(check.call_count, 1)
                command = check.call_args.args[0]
                self.assertIn("-check", command)
                self.assertNotIn("-interp", command)
                self.assertNotIn("-exe", command)

    def test_source_error_or_timeout_remains_a_failure(self):
        args = argparse.Namespace(jaic="metadata-only", timeout=12, coverage=None, platform="linux-x64")
        for status in (1, None):
            with (
                self.subTest(status=status),
                patch.object(
                    runtime, "run_command", return_value=(status, "", "source error")
                ),
                patch.object(
                    runtime,
                    "run_program",
                    side_effect=AssertionError("main must never run"),
                ),
            ):
                passed, _ = runtime.run_mode(
                    args,
                    "source-control",
                    ROOT / "source.jai",
                    "interp",
                    assigned_scratch(),
                    {"source_check_only": True},
                )
                self.assertIs(passed, False)

    def test_success_count_is_source_only_and_failure_is_not_unavailable(self):
        for compiler_status in (0, 1):
            with (
                self.subTest(compiler_status=compiler_status),
                tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch,
            ):
                directory = Path(scratch)
                compiler = directory / "digest-only-not-executable"
                compiler.write_text("not executed; all compiler calls mocked")
                report = directory / "report.json"
                case = {
                    "id": "Extensions/Io_Uring/tests/dependency",
                    "path": ROOT / "stdlib/Extensions/Io_Uring/tests/dependency.jai",
                    "source_check_only": True,
                }
                with (
                    patch.object(
                        sys,
                        "argv",
                        [
                            "stdlib_runtime",
                            "--jaic",
                            str(compiler),
                            "--platform",
                            "linux-arm64",
                            "--modes",
                            "interp,native",
                            "--jobs",
                            "1",
                            "--report",
                            str(report),
                        ],
                    ),
                    patch.object(runtime, "host_platform", return_value="linux-arm64"),
                    patch.object(runtime, "validated_entries", return_value=[case]),
                    patch.object(runtime, "unavailable_reason", return_value=None),
                    patch.object(
                        runtime, "reference_rejected_reason", return_value=None
                    ),
                    patch.object(runtime, "use_native_libs"),
                    patch.object(
                        runtime, "run_command", return_value=(compiler_status, "", "")
                    ),
                    patch.object(
                        runtime,
                        "run_program",
                        side_effect=AssertionError("main must never run"),
                    ),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    status = runtime.main()
                self.assertEqual(status, 0 if compiler_status == 0 else 1)
                result = json.loads(report.read_text())
                for mode in ("interp", "native"):
                    counts = result["counts"][mode]
                    self.assertEqual(
                        counts["source_check_only"], 1 if compiler_status == 0 else 0
                    )
                    self.assertEqual(counts["passed"], 0)
                    self.assertEqual(counts["compile_time_only"], 0)
                    self.assertEqual(counts["platform_unavailable"], 0)
                    self.assertEqual(counts["failed"], compiler_status)


if __name__ == "__main__":
    unittest.main()
