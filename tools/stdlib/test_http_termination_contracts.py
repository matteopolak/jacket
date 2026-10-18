"""Required termination outcomes are protocols, not generic exit-zero cases."""

from pathlib import Path
import unittest

from stdlib_runtime import contract_result
from stdlib_test_catalog import case_for_mode, unavailable_reason, validated_entries

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    "Extensions/Http/tests/wasm-termination-status": (37, "before exit 37\n"),
    "Extensions/Http/tests/wasm-termination-failure": (
        1,
        "FAIL: independent WASM termination control\n1 failure(s)\n",
    ),
}


class HttpTerminationContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cases = {case["id"]: case for case in validated_entries(ROOT)}

    def test_catalogue_records_exact_protocol_in_every_required_mode(self):
        for name, (status, output) in EXPECTED.items():
            for mode in ("interp", "native", "wasm-interp", "wasm-native"):
                with self.subTest(name=name, mode=mode):
                    case = case_for_mode(self.cases[name], mode)
                    self.assertIsNone(unavailable_reason(case, mode, "macos-arm64"))
                    self.assertEqual(case.get("exit_code"), status)
                    self.assertEqual(case.get("stdout"), output)
                    self.assertEqual(case.get("stderr"), "")
                    self.assertTrue(contract_result(case, status, output, ""))

    def test_protocol_mismatches_remain_failures(self):
        for name, (status, output) in EXPECTED.items():
            case = self.cases[name]
            with self.subTest(name=name):
                for actual in (None, 0, 2, -6, 120, 121):
                    self.assertFalse(contract_result(case, actual, output, ""))
                self.assertFalse(contract_result(case, status, "", ""))
                self.assertFalse(
                    contract_result(case, status, output + "ERROR: continued\n", "")
                )
                self.assertFalse(
                    contract_result(case, status, output, "unexpected diagnostic\n")
                )

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

    def test_app_boundary_codec_and_unknown_cases_are_unchanged(self):
        for name in (
            "Extensions/Http/tests/client-overload-lint",
            "Extensions/Http/tests/dependency-consumer",
        ):
            case = self.cases[name]
            self.assertIsNone(unavailable_reason(case, "native", "macos-arm64"))
            self.assertIsNotNone(unavailable_reason(case, "wasm-native", "macos-arm64"))
        case = self.cases["Extensions/Http/tests/codec-roundtrip"]
        self.assertIsNone(unavailable_reason(case, "wasm-native", "macos-arm64"))
        self.assertNotIn("exit_code", case)


if __name__ == "__main__":
    unittest.main()
