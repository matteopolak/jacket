

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

from stdlib.stdlib_runtime import contract_result, run_mode
from stdlib.stdlib_test_catalog import validated_entries, unavailable_reason


class CatalogueTests(unittest.TestCase):
    def test_appledouble_metadata_is_not_an_independent_program(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            regression = root / "tests/compiler/regressions"
            regression.mkdir(parents=True)
            (regression / "actual.jai").write_text("main :: () {}")
            (regression / "._actual.jai").write_bytes(b"\x00\x05\x16\x07")
            module = root / "stdlib/Real/tests"
            module.mkdir(parents=True)
            (module / "actual.jai").write_text("main :: () {}")
            (module / "._actual.jai").write_bytes(b"\x00\x05\x16\x07")
            self.assertEqual(
                {c["id"] for c in validated_entries(root)},
                {"compiler/actual", "Real/tests/actual"},
            )

    def test_native_platform_requirement_is_not_a_success(self):
        case = {"family": "modules", "required_os": "windows"}
        self.assertIsNotNone(unavailable_reason(case, "native", "macos-arm64"))
        self.assertIsNone(unavailable_reason(case, "native", "windows-arm64"))
        self.assertIsNotNone(unavailable_reason(case, "wasm-interp", "windows-arm64"))
        case["required_os"] = ("linux", "macos")
        self.assertIsNone(unavailable_reason(case, "native", "macos-arm64"))
        self.assertIsNotNone(unavailable_reason(case, "native", "windows-x64"))

    def test_reviewed_and_independent_families_are_discovered(self):
        cases = validated_entries()
        # Module tests run under `jacket test`, so only the compiler regressions are left.
        self.assertEqual({c["family"] for c in cases}, {"compiler"})
        self.assertTrue(
            any(c["id"] == "compiler/array-bounds-check-opt-out" for c in cases)
        )
        self.assertFalse(any("tests/stdlib/" in str(c["path"]) for c in cases))

    def test_metaprogram_regressions_are_interpreter_only(self):
        only = [c["id"] for c in validated_entries() if c.get("interpreter_only")]
        self.assertIn("compiler/rules-workspaces", only)
        self.assertGreaterEqual(len(only), 30)
        # Everything that creates a workspace of its own is marked, so a new one is not run five times.
        for case in validated_entries():
            if case["family"] == "compiler":
                text = case["path"].read_text(encoding="utf-8")
                if "compiler_create_workspace" in text or "compiler_begin_intercept" in text:
                    self.assertTrue(case.get("interpreter_only"), case["id"])

    def test_precise_negative_and_positive_output_are_required(self):
        case = {
            "mode": "check-fail",
            "diagnostic_contains": "has no exported member `HASH_INIT`",
        }
        self.assertFalse(contract_result(case, 0, case["diagnostic_contains"]))
        self.assertFalse(contract_result(case, 1, "unrelated syntax error"))
        self.assertTrue(
            contract_result(case, 1, "error: " + case["diagnostic_contains"])
        )
        self.assertFalse(contract_result({"stdout": "ok\n"}, 0, "wrong\n"))
        self.assertIsNone(unavailable_reason({"family": "native"}, "native"))

    def test_expected_diagnostics_are_separate_from_program_stdout(self):
        case = {"stdout": "ok\n", "stderr": "expected diagnostic\n"}
        self.assertTrue(contract_result(case, 0, "ok\n", "expected diagnostic\n"))
        self.assertFalse(contract_result(case, 0, "ok\n", "unrelated failure\n"))
        self.assertFalse(contract_result(case, 0, "wrong\n", "expected diagnostic\n"))

    def test_missing_main_does_not_hide_other_compile_errors(self):
        args = SimpleNamespace(
            jaic="own-jaic", opt="O0", timeout=5, coverage=None, dlls=None
        )
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "program.jai"
            with patch(
                "stdlib.stdlib_runtime.run_command",
                side_effect=[
                    (
                        1,
                        "",
                        "program has no `main` procedure; unknown identifier genuine_error",
                    ),
                    (1, "", "unknown identifier genuine_error"),
                ],
            ):
                passed, detail = run_mode(args, "program", path, "native", temporary)
                self.assertIs(passed, False, detail)

    def test_timeout_is_not_reclassified_as_compile_time_only(self):
        args = SimpleNamespace(
            jaic="own-jaic", opt="O0", timeout=5, coverage=None, dlls=None
        )
        with TemporaryDirectory() as temporary:
            with patch(
                "stdlib.stdlib_runtime.run_command",
                return_value=(None, "", "program has no `main` procedure"),
            ):
                passed, detail = run_mode(
                    args,
                    "program",
                    Path(temporary) / "program.jai",
                    "native",
                    temporary,
                )
                self.assertIs(passed, False, detail)

    def test_nonzero_stdout_contract_is_supported_generally(self):
        case = {"exit_code": 23, "stdout": "done\n", "stderr": ""}
        self.assertTrue(contract_result(case, 23, "done\n", ""))
        self.assertFalse(contract_result(case, 0, "done\n", ""))
        self.assertFalse(contract_result(case, 23, "done\n", "extra"))

    def test_ordinary_positive_and_precise_negative_contracts_are_preserved(self):
        self.assertTrue(contract_result({}, 0, "ordinary output", ""))
        self.assertFalse(contract_result({}, 37, "ordinary output", ""))
        self.assertTrue(contract_result({"stdout": "ok\n"}, 0, "ok\n", ""))
        self.assertFalse(contract_result({"stdout": "ok\n"}, 1, "ok\n", ""))
        case = {"mode": "check-fail", "diagnostic_contains": "needs fibers"}
        self.assertTrue(contract_result(case, 1, "", "needs fibers"))
        self.assertFalse(contract_result(case, 1, "", "unrelated compiler failure"))
        self.assertFalse(contract_result(case, 0, "", "needs fibers"))


if __name__ == "__main__":
    unittest.main()
