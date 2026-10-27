"""HTTP conformance harness: inventory, mode selection and failure accounting.

Mocked dispatch verifies what the harness runs, not native functionality.
"""

from pathlib import Path
import unittest
from unittest.mock import patch

from stdlib import http_conformance


class HttpCoverageTests(unittest.TestCase):
    def test_every_inventoried_test_exists(self):
        self.assertEqual(len(http_conformance.HTTP_CONFORMANCE_TESTS), 13)
        for name in http_conformance.HTTP_CONFORMANCE_TESTS:
            self.assertTrue(
                (http_conformance.HTTP_TESTS / (name + ".jai")).is_file(), name
            )

    def run_harness(self, modes, status=0):
        with (
            patch.object(
                http_conformance, "jacket_binary", return_value=Path("own-jacket")
            ),
            patch.object(http_conformance.subprocess, "run") as run,
        ):
            run.return_value.returncode = status
            result = http_conformance.run_harness(Path("own-jaic"), modes, 3)
        return result, [call.args[0] for call in run.call_args_list]

    def test_each_mode_runs_the_http_folder_once_with_its_options(self):
        ok, commands = self.run_harness("interp,native,wasm-interp,wasm-native")
        self.assertTrue(ok)
        self.assertEqual(len(commands), 4)
        for command in commands:
            self.assertEqual(command[:2], ["own-jacket", "test"])
            self.assertEqual(
                command[command.index("--path") + 1],
                str(http_conformance.HTTP_TESTS.parent),
            )
            self.assertEqual(command[command.index("-j") + 1], "3")
        interp, native, wasm_interp, wasm_native = commands
        self.assertIn("--interp", interp)
        self.assertNotIn("-os", interp)
        self.assertNotIn("--interp", native)
        self.assertNotIn("-os", native)
        self.assertIn("--interp", wasm_interp)
        self.assertEqual(wasm_interp[wasm_interp.index("-os") + 1], "wasm")
        self.assertNotIn("--interp", wasm_native)
        self.assertEqual(wasm_native[wasm_native.index("-os") + 1], "wasm")
        self.assertIn("--runner", wasm_native)

    def test_a_failing_mode_fails_the_run_but_later_modes_still_run(self):
        ok, commands = self.run_harness("interp,native", status=1)
        self.assertFalse(ok)
        self.assertEqual(len(commands), 2)

    def test_missing_http_test_fails_before_any_child_runs(self):
        names = tuple(
            name
            for name in http_conformance.HTTP_CONFORMANCE_TESTS
            if name != "codec-roundtrip"
        ) + ("no-such-test",)
        with (
            patch.object(http_conformance, "HTTP_CONFORMANCE_TESTS", names),
            patch.object(http_conformance.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(RuntimeError, "no-such-test"):
                http_conformance.run_harness(Path("own-jaic"), "native", 1)
        run.assert_not_called()

    def test_unknown_mode_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "check"):
            http_conformance.run_harness(Path("own-jaic"), "native,check", 1)


if __name__ == "__main__":
    unittest.main()
