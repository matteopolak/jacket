"""Exact native Http fixtures; no production behavior or target execution."""

from pathlib import Path
import sys
import unittest

from stdlib_test_catalog import unavailable_reason, validated_entries

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
WASM = ("wasm-interp", "wasm-native")


class HttpAppTargets(unittest.TestCase):
    def setUp(self):
        self.cases = {case["id"]: case for case in validated_entries(ROOT)}

    def test_both_native_app_fixtures_have_explicit_wasm_boundary(self):
        for name in [
            "Extensions/Http/tests/client-overload-lint",
            "Extensions/Http/tests/dependency-consumer",
        ]:
            with self.subTest(name=name):
                case = self.cases[name]
                self.assertEqual(case.get("unsupported_modes"), list(WASM))
                self.assertNotIn("compile_only", case)
                self.assertNotIn("expected_error", case)
                self.assertTrue(case["required_capabilities"])
                for mode in WASM:
                    self.assertIsNotNone(unavailable_reason(case, mode, "macos-arm64"))

    def test_portable_codec_and_declared_wasm_cases_stay_required(self):
        for name in [
            "Extensions/Http/tests/codec",
            "Extensions/Http/tests/codec-roundtrip",
        ]:
            with self.subTest(name=name):
                for mode in WASM:
                    self.assertIsNone(unavailable_reason(self.cases[name], mode, "macos-arm64"))
                self.assertNotIn("unsupported_modes", self.cases[name])


if __name__ == "__main__":
    unittest.main()
