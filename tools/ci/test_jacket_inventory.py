import io
import json
import unittest
from unittest import mock

from jacket_inventory import main, metadata, sanitize


class Allowlist(unittest.TestCase):
    def test_output_paths_arbitrary_text_and_environment_are_not_retained(self):
        record = {
            "type": "test", "event": "skipped", "name": "Io::backends", "kind": "golden",
            "file": "/Users/private/test.jai", "stdout": "secret", "stderr": "secret",
            "problems": ["secret"], "created": ["private"], "temp_dir": "/private/tmp/user",
            "reason": "missing test prerequisite: /Users/private/library", "environment": {"TOKEN": "secret"},
        }
        result = sanitize(record, "outcomes")
        self.assertEqual(result, {"type": "test", "event": "skipped", "name": "Io::backends", "kind": "golden", "reason_category": "prerequisite"})
        self.assertNotIn("secret", json.dumps(result))
        self.assertEqual(sanitize({**record, "type": "list"}, "list"), {"type": "list", "name": "Io::backends", "kind": "golden"})

    def test_declared_ignore_and_mode_filters_have_explicit_categories(self):
        for reason, category in [("ignored (`jai,ignore`)", "declared-ignore"), ("interp only", "mode-filter"), ("native only", "mode-filter")]:
            with self.subTest(reason=reason):
                result = sanitize({"type": "test", "event": "skipped", "name": "doc: example", "kind": "doc", "reason": reason}, "outcomes")
                self.assertEqual(result["reason_category"], category)

    def test_documented_slow_notice_is_progress_not_a_case_outcome(self):
        raw = {"type": "test", "event": "slow", "name": "Module::one", "seconds": 12.5}
        self.assertIsNone(sanitize(raw, "outcomes"))
        with self.assertRaises(ValueError):
            sanitize({**raw, "event": "unknown-event", "kind": "test"}, "outcomes")

    def test_absolute_identities_are_rejected(self):
        for name in ["/Users/private/test", "C:\\private\\test", "../private"]:
            with self.assertRaisesRegex(ValueError, "non-relative"):
                sanitize({"type": "list", "kind": "test", "name": name}, "list")

    def test_public_metadata_never_enumerates_environment_or_hostname(self):
        with mock.patch.dict("os.environ", {"GITHUB_SHA": "a" * 40, "TOKEN": "secret"}), mock.patch("jacket_inventory.subprocess.check_output", return_value="rustc 1.99.0-nightly (abcdef123 2026-01-01)\n"):
            result = metadata("hash:1/2")
        self.assertEqual(set(result), {"revision", "partition", "os", "arch", "rustc"})
        self.assertNotIn("secret", json.dumps(result))

    def test_completed_children_keep_only_identity_and_status(self):
        raw = {"type": "test", "event": "passed", "name": "M::one", "kind": "test", "subtests": [{"name": "one/a", "status": "pass", "duration": 12.5, "stdout": "secret"}]}
        result = sanitize(raw, "outcomes")
        self.assertEqual(result["subtests"], [{"name": "one/a", "status": "pass"}])
        self.assertNotIn("secret", json.dumps(result))


class Diagnostics(unittest.TestCase):
    def run_cli(self, record, enabled):
        stdout, stderr = io.StringIO(), io.StringIO()
        argv = ["jacket_inventory.py", "outcomes", *(["--diagnostics"] if enabled else [])]
        with mock.patch.dict("os.environ", {"GITHUB_ACTIONS": "true"}), mock.patch("sys.argv", argv), mock.patch("sys.stdin", io.StringIO(json.dumps(record) + "\n")), mock.patch("sys.stdout", stdout), mock.patch("sys.stderr", stderr):
            main()
        return stdout.getvalue(), stderr.getvalue()

    def test_slow_notice_does_not_emit_null_or_hide_terminal_outcome(self):
        progress = {"type": "test", "event": "slow", "name": "M::one", "seconds": 12.5}
        passed = {"type": "test", "event": "passed", "name": "M::one", "kind": "test"}
        stdout = io.StringIO()
        lines = "\n".join(map(json.dumps, [progress, passed])) + "\n"
        with mock.patch("sys.argv", ["jacket_inventory.py", "outcomes"]), mock.patch("sys.stdin", io.StringIO(lines)), mock.patch("sys.stdout", stdout):
            main()
        self.assertEqual([json.loads(line) for line in stdout.getvalue().splitlines()], [sanitize(passed, "outcomes")])

    def test_invalid_stream_is_drained_before_failure(self):
        passed = {"type": "test", "event": "passed", "name": "M::one", "kind": "test"}
        consumed = []
        def lines():
            for index, line in enumerate(["malformed producer text\n", json.dumps(passed) + "\n"]):
                consumed.append(index)
                yield line
        stdout = io.StringIO()
        with mock.patch("sys.argv", ["jacket_inventory.py", "outcomes"]), mock.patch("sys.stdin", lines()), mock.patch("sys.stdout", stdout):
            with self.assertRaises(ValueError):
                main()
        self.assertEqual(consumed, [0, 1])
        self.assertEqual(json.loads(stdout.getvalue()), sanitize(passed, "outcomes"))

    def test_failed_case_logs_survive_without_broadening_the_artifact(self):
        raw = {"type": "test", "event": "failed", "name": "M::one", "kind": "test",
               "file": "/home/runner/work/repo/test.jai", "stdout": "captured output", "stderr": "captured error",
               "environment": {"TOKEN": "never print this"}, "problems": ["original problem"]}
        stdout, stderr = self.run_cli(raw, True)
        self.assertEqual(json.loads(stdout), sanitize(raw, "outcomes"))
        self.assertIn("captured error", stderr)
        self.assertIn(raw["file"], stderr)
        self.assertIn("original problem", stderr)
        self.assertNotIn("never print this", stdout + stderr)
        self.assertNotIn("captured output", stdout)

    def test_default_and_passed_cases_do_not_print_diagnostics(self):
        raw = {"type": "test", "event": "failed", "name": "one", "kind": "test", "stdout": "output"}
        self.assertEqual(self.run_cli(raw, False)[1], "")
        self.assertEqual(self.run_cli({**raw, "event": "passed"}, True)[1], "")


    def test_local_diagnostics_are_refused_before_reading_any_input(self):
        stdout, stderr = io.StringIO(), io.StringIO()
        with mock.patch.dict("os.environ", {"GITHUB_ACTIONS": "false"}), mock.patch("sys.argv", ["jacket_inventory.py", "outcomes", "--diagnostics"]), mock.patch("sys.stdin", io.StringIO("local input must not be read")), mock.patch("sys.stdout", stdout), mock.patch("sys.stderr", stderr):
            with self.assertRaises(SystemExit):
                main()
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("hosted CI", stderr.getvalue())
