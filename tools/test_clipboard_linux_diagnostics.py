"""CI diagnostic policy tests only; never execute Jai, native SDKs or X11."""

import io
from pathlib import Path
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout

from clipboard_linux_diagnostics import failure_diagnostic, report_failure
from check_clipboard_linux_package import require_runtime_verification


from ci_test_scratch import assigned_scratch


class ClipboardDiagnostics(unittest.TestCase):
    def test_compiler_header_survives_without_source_context(self):
        row = dict(
            name="linux-jai-build:-O0",
            exit=1,
            stderr=(
                "tests/portable-stdlib/clipboard-linux-roundtrip.jai:18:2: error: missing symbol `example`\n"
                "    source_body_marker = private_material;\n"
                "    ^^^^^^^^^^^^^^^^^^^\n"
                "stdlib/internal/clipboard/linux.jai:4:1: note: declared here\n"
            ),
            stdout="",
        )
        text = failure_diagnostic(row)
        self.assertIn("missing symbol `example`", text)
        self.assertIn("declared here", text)
        self.assertNotIn("source_body_marker", text)
        self.assertNotIn("^^^^", text)
        self.assertIn("exit=1", text)

    def test_linker_headers_are_visible(self):
        for header in [
            "ld.lld: error: undefined symbol: actual_symbol",
            "ld: library not found for -lrequired_library",
            "/usr/bin/ld: cannot find -lrequired_library: No such file or directory",
            "LLVM ERROR: genuine backend diagnostic",
            "clang: error: linker command failed with exit code 1",
            "some-object.o: undefined reference to `required_function`",
        ]:
            with self.subTest(header=header):
                text = failure_diagnostic(
                    dict(name="build", exit=1, stderr=header, stdout="")
                )
                self.assertIn(header, text)

    def test_ansi_and_credentials_never_pass_through(self):
        text = failure_diagnostic(
            dict(
                name="build",
                exit=1,
                stderr=(
                    "\x1b[31merror: dependency failed token=synthetic_token password=synthetic_password authorization: Bearer synthetic_bearer\x1b[0m\n"
                    "unrelated raw output synthetic_raw_payload\n"
                ),
                stdout="",
            )
        )
        self.assertNotIn("\x1b", text)
        self.assertNotIn("synthetic_token", text)
        self.assertNotIn("synthetic_password", text)
        self.assertNotIn("synthetic_bearer", text)
        self.assertNotIn("synthetic_raw_payload", text)

    def test_output_is_bounded_and_unknown_output_is_not_dumped(self):
        row = dict(
            name="build",
            exit=1,
            stdout="",
            stderr="\n".join("error: " + "é" * 900 for _ in range(100)),
        )
        self.assertLessEqual(len(failure_diagnostic(row).encode()), 4096)
        unknown = failure_diagnostic(
            dict(name="build", exit=1, stdout="source_marker\n", stderr="auth_marker\n")
        )
        self.assertNotIn("source_marker", unknown)
        self.assertNotIn("auth_marker", unknown)
        self.assertIn("preserved", unknown)

    def test_failure_summary_uses_stderr_and_leaves_full_capture_intact(self):
        base = assigned_scratch()
        with tempfile.TemporaryDirectory(
            prefix="clipboard-diagnostic-", dir=base
        ) as temporary:
            capture = Path(temporary) / "commands.json"
            original = '{"stderr": "original full source context"}\n'
            capture.write_text(original)
            stream = io.StringIO()
            row = dict(
                name="linux-jai-build:-O2",
                exit=1,
                stdout="",
                stderr="error: exact build failure",
            )
            report_failure(row, capture, stream=stream)
            self.assertIn("exact build failure", stream.getvalue())
            self.assertIn(str(capture), stream.getvalue())
            self.assertEqual(capture.read_text(), original)

    def test_package_still_rejects_failed_proof_and_prioritizes_jai_error(self):
        failures = [
            dict(
                name="other-failure",
                exit=1,
                passed=False,
                stderr="error: other",
                stdout="",
            )
        ]
        failures += [
            dict(
                name="linux-jai-build:" + mode,
                exit=1,
                passed=False,
                stderr="error: compile " + mode,
                stdout="",
            )
            for mode in ["-O0", "-O2"]
        ]
        # Deliberately FAILED synthetic tool metadata, never a fake SDK proof.
        prior = dict(passed=False, linux_runtime_executed=True, commands=failures)
        stderr = io.StringIO()
        stdout = io.StringIO()
        with redirect_stderr(stderr), redirect_stdout(stdout):
            with self.assertRaisesRegex(
                RuntimeError, "real Linux runtime verification is required"
            ):
                require_runtime_verification(Path("proof/summary.json"), prior)
            print("proof/summary.json")
        self.assertLess(
            stderr.getvalue().index("linux-jai-build:-O0"),
            stderr.getvalue().index("other-failure"),
        )
        self.assertIn("compile -O2", stderr.getvalue())
        self.assertEqual(stdout.getvalue(), "proof/summary.json\n")

    def test_package_still_requires_actual_runtime_even_if_checks_passed(self):
        prior = dict(passed=True, linux_runtime_executed=False, commands=[])
        with self.assertRaisesRegex(
            RuntimeError, "real Linux runtime verification is required"
        ):
            require_runtime_verification(Path("proof/summary.json"), prior)


if __name__ == "__main__":
    unittest.main()
