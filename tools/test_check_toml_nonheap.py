"""Nonheap receipt policy tests; these do not execute or qualify TOML programs."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from check_toml_nonheap import PROJECT, command_passed, stage_admitted


class ReceiptPolicy(unittest.TestCase):
    def test_runtime_diagnostic_is_not_success_even_with_zero_exit(self):
        self.assertFalse(
            command_passed(0, "continued\n", "assertion failed\n", runtime=True)
        )

    def test_runtime_marker_is_required(self):
        self.assertFalse(
            command_passed(0, "", "", runtime=True, expected_stdout="ok\n")
        )
        self.assertTrue(
            command_passed(0, "ok\n", "", runtime=True, expected_stdout="ok\n")
        )

    def test_original_negative_cases_may_emit_application_diagnostics(self):
        self.assertTrue(
            command_passed(
                0,
                "",
                "invalid input rejected",
                runtime=True,
                application_negatives=True,
            )
        )

    def test_original_negative_policy_never_accepts_runtime_failure_diagnostics(self):
        for message in ("Assertion failed", "runtime error: null pointer read"):
            self.assertFalse(
                command_passed(0, "", message, runtime=True, application_negatives=True)
            )
            self.assertFalse(
                command_passed(0, message, "", runtime=True, application_negatives=True)
            )

    def test_nonzero_exit_is_never_positive(self):
        for code in (1, -6):
            self.assertFalse(command_passed(code, "ok\n", "", runtime=True))

    def test_negative_requires_the_specific_typechecking_diagnostic(self):
        message = (
            "argument of type `Distinct_Number` does not match parameter type `float64`"
        )
        self.assertTrue(command_passed(1, "", message, reject=True))
        self.assertFalse(command_passed(1, "", "file not found", reject=True))
        self.assertFalse(command_passed(0, "", message, reject=True))


class AdmissionPolicy(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name) / "curated"
        self.root.mkdir()
        self.output = Path(self.scratch.name) / "stage"

    def admit(self, name, data):
        (self.root / "ACCESS.json").write_text(
            json.dumps(
                {"files": [{"path": name, "sha256": hashlib.sha256(data).hexdigest()}]}
            )
        )

    def test_only_admitted_inputs_are_staged(self):
        name = PROJECT + "/examples/input.jai"
        source = self.root / name
        source.parent.mkdir(parents=True)
        source.write_bytes(b"admitted")
        source.with_name("unadmitted.jai").write_bytes(b"excluded")
        self.admit(name, b"admitted")
        self.assertEqual(len(stage_admitted(self.root, self.output)), 1)
        self.assertEqual((self.output / name).read_bytes(), b"admitted")
        self.assertFalse((self.output / PROJECT / "examples/unadmitted.jai").exists())

    def test_changed_admitted_input_is_rejected(self):
        name = PROJECT + "/changed.jai"
        source = self.root / name
        source.parent.mkdir()
        source.write_bytes(b"changed")
        self.admit(name, b"original")
        with self.assertRaisesRegex(ValueError, "changed admitted input"):
            stage_admitted(self.root, self.output)

    def test_traversal_is_rejected(self):
        self.admit(PROJECT + "/../outside.jai", b"excluded")
        with self.assertRaisesRegex(ValueError, "unsafe admitted"):
            stage_admitted(self.root, self.output)

    def test_symlink_escape_is_rejected(self):
        external = Path(self.scratch.name) / "outside.jai"
        external.write_bytes(b"excluded")
        name = PROJECT + "/escape.jai"
        source = self.root / name
        source.parent.mkdir()
        source.symlink_to(external)
        self.admit(name, b"excluded")
        with self.assertRaisesRegex(ValueError, "escapes curated"):
            stage_admitted(self.root, self.output)


if __name__ == "__main__":
    unittest.main()
