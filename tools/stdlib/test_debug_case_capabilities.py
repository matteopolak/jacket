"""Debug mode selection policies; no mocked result is native provider proof."""

import unittest

from stdlib_runtime import contract_result
from stdlib_test_catalog import unavailable_reason, validated_entries


class DebugCapabilities(unittest.TestCase):
    def test_actual_catalogue_keeps_both_cases_and_attaches_only_ownership_requirement(
        self,
    ):
        cases = {case["id"]: case for case in validated_entries()}
        owner = cases["Debug/tests/ignore-ownership"]
        for mode in ("wasm-interp", "wasm-native"):
            self.assertIsNotNone(unavailable_reason(owner, mode, "macos-arm64"))
        portable = cases["Debug/tests/target"]
        self.assertNotIn("unsupported_modes", portable)

    def ownership(self):
        return {c["id"]: c for c in validated_entries()}["Debug/tests/ignore-ownership"].copy()

    def test_ownership_remains_required_on_every_desktop_native_mode(self):
        case = self.ownership()
        for os_name in ("macos", "linux", "windows"):
            for cpu in ("x64", "arm64"):
                for mode in ("interp", "native"):
                    self.assertIsNone(
                        unavailable_reason(case, mode, os_name + "-" + cpu)
                    )

    def test_wasm_reports_ownership_unavailable_not_passed(self):
        case = self.ownership()
        for mode in ("wasm-interp", "wasm-native"):
            reason = unavailable_reason(case, mode, "macos-arm64")
            self.assertIn("native Debug diagnostic output", reason)
            self.assertIn("not passed", reason)

    def test_no_module_wide_exclusion_or_debugger_deferral(self):
        cases = {case["id"]: case for case in validated_entries()}
        for name in cases:
            if not name.startswith("Debug/tests/") or name.endswith("ignore-ownership"):
                continue
            for mode in ("interp", "native", "wasm-interp", "wasm-native"):
                self.assertIsNone(unavailable_reason(cases[name], mode, "macos-arm64"), name)

    def test_missing_real_provider_is_still_a_failed_native_result(self):
        case = self.ownership()
        for platform in ("macos-arm64", "linux-x64", "windows-arm64"):
            for mode in ("interp", "native"):
                self.assertIsNone(unavailable_reason(case, mode, platform))
                self.assertFalse(
                    contract_result(case, 1, "", "missing developer_debug_output")
                )
                self.assertFalse(contract_result(case, None, "", "provider timeout"))


if __name__ == "__main__":
    unittest.main()
