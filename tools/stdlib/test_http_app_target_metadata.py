"""Exact native Http fixtures; no production behavior or target execution."""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import sys
import unittest

from stdlib.stdlib_test_catalog import unavailable_reason, validated_entries
from toolkit.repo import ROOT

sys.dont_write_bytecode = True
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
                    self.assertIsNone(
                        unavailable_reason(self.cases[name], mode, "macos-arm64")
                    )
                self.assertNotIn("unsupported_modes", self.cases[name])


if __name__ == "__main__":
    unittest.main()
