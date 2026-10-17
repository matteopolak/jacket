"""Mocked original-workload wiring; no compiler, native or network execution."""

import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import io_stress as stress
from ci_test_scratch import assigned_scratch


class FailureWiring(unittest.TestCase):
    def test_twelve_original_copies_and_original_failure_gate(self):
        for failed_copy in (None, 6):
            with (
                self.subTest(failed_copy=failed_copy),
                tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch,
            ):
                root = Path(scratch)
                test = root / "backends.jai"
                test.write_text(
                    "// metadata-only input; mocked build, never compiled\n"
                )
                groups = []

                class Group:
                    def __init__(self, report, identity):
                        self.records = {}
                        self.requests = []
                        self.failed = None
                        groups.append(self)

                    def run(self, copy, program, directory, timeout):
                        self.requests.append((copy, program, directory, timeout))
                        status = 1 if copy == failed_copy else 0
                        self.records[copy] = {
                            "exit_phase": "runtime_exit",
                            "exit": status,
                        }
                        return subprocess.CompletedProcess(
                            [str(program)], status, "", ""
                        )

                    def finish(self, copies):
                        self.failed = copies

                def build(compiler, source, variant, output):
                    output.write_text("digest input only; never executed")
                    return None

                with (
                    patch.object(stress.sys, "platform", "linux"),
                    patch.object(
                        sys,
                        "argv",
                        [
                            "io_stress",
                            "--jaic",
                            str(root / "unused-compiler"),
                            "--variants",
                            "default",
                            "--copies",
                            "12",
                            "--rounds",
                            "1",
                            "--observe-failures-report",
                            str(root / "report"),
                            str(test),
                        ],
                    ),
                    patch.object(stress, "ensure_native_libraries"),
                    patch.object(
                        stress,
                        "source_identity",
                        return_value={"source_sha256": "metadata-test"},
                    ),
                    patch.object(stress, "FailureGroup", Group),
                    patch.object(stress, "build", side_effect=build) as compile,
                    patch.object(
                        stress.tempfile, "mkdtemp", return_value=str(root / "work")
                    ),
                    patch.object(
                        stress.subprocess,
                        "run",
                        side_effect=AssertionError("unexpected extra launch"),
                    ),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    status = stress.main()
                self.assertEqual(status, 1 if failed_copy is not None else 0)
                compile.assert_called_once()
                self.assertEqual(len(groups), 1)
                requests = groups[0].requests
                self.assertEqual(len(requests), 12)
                self.assertEqual({r[0] for r in requests}, set(range(12)))
                self.assertTrue(all(r[3] == 180 for r in requests))
                self.assertTrue(all(r[2].name == str(r[0]) for r in requests))
                self.assertEqual(groups[0].failed, [] if failed_copy is None else [6])
                self.assertTrue(
                    json.loads((root / "report/receipt.json").read_text())[
                        "source_guard"
                    ]["matches"]
                )

    def test_non_linux_request_rejects_before_build_or_launch(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            root = Path(scratch)
            test = root / "backends.jai"
            test.write_text("// metadata-only input\n")
            error_output = io.StringIO()
            with (
                patch.object(stress.sys, "platform", "darwin"),
                patch.object(
                    sys,
                    "argv",
                    [
                        "io_stress",
                        "--observe-failures-report",
                        str(root / "report"),
                        str(test),
                    ],
                ),
                patch.object(stress, "ensure_native_libraries") as native,
                patch.object(stress.subprocess, "Popen") as launch,
                contextlib.redirect_stderr(error_output),
                self.assertRaises(SystemExit) as stopped,
            ):
                stress.main()
            self.assertEqual(stopped.exception.code, 2)
            self.assertIn("failure observation requires Linux", error_output.getvalue())
            native.assert_not_called()
            launch.assert_not_called()


if __name__ == "__main__":
    unittest.main()
