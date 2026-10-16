"""Exact native Http fixtures; no production behavior or target execution."""

import importlib.util
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    "foundations_capabilities", ROOT / "tools/foundations_case_capabilities.py"
)
capabilities = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capabilities)


class HttpAppTargets(unittest.TestCase):
    def test_both_native_app_fixtures_have_explicit_wasm_boundary(self):
        for name in [
            "extensions/http-client-overload-lint-repro",
            "extensions/http-dependency-consumer",
        ]:
            with self.subTest(name=name):
                case = {
                    "id": name,
                    "expected_exit": 0,
                    "required_capabilities": ["existing"],
                }
                capabilities.apply_foundations_capabilities(case)
                self.assertEqual(
                    case.get("unsupported_modes"), ["wasm-interp", "wasm-native"]
                )
                self.assertEqual(case["expected_exit"], 0)
                self.assertNotIn("compile_only", case)
                self.assertNotIn("expected_error", case)
                self.assertEqual(case["required_capabilities"][0], "existing")

    def test_portable_codec_and_declared_wasm_cases_stay_required(self):
        for name in [
            "Extensions/Http/tests/codec",
            "extensions/http-codec-roundtrip",
            "extensions/http-wasm-termination-status",
            "extensions/http-wasm-termination-failure",
            "extensions/http-new-unclassified-case",
        ]:
            with self.subTest(name=name):
                case = {"id": name}
                capabilities.apply_foundations_capabilities(case)
                self.assertEqual(case, {"id": name})

    def test_capability_attachment_is_idempotent(self):
        case = {"id": "extensions/http-dependency-consumer"}
        capabilities.apply_foundations_capabilities(case)
        once = dict(case)
        capabilities.apply_foundations_capabilities(case)
        self.assertEqual(case, once)


if __name__ == "__main__":
    unittest.main()
