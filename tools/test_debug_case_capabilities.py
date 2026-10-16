"""Debug mode selection policies; no mocked result is native provider proof."""

import unittest

from debug_case_capabilities import apply_debug_capabilities
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
        portable = cases["Debug/tests/portable-target-contract"]
        self.assertNotIn("unsupported_modes", portable)

    def ownership(self):
        case = {"id": "Debug/tests/ignore-ownership", "family": "modules"}
        apply_debug_capabilities(case)
        return case

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
        for name in (
            "Debug/tests/portable-target-contract",
            "Debug/tests/new-portable",
        ):
            case = {"id": name, "family": "modules"}
            apply_debug_capabilities(case)
            self.assertEqual(case, {"id": name, "family": "modules"})
            for mode in ("interp", "native", "wasm-interp", "wasm-native"):
                self.assertIsNone(unavailable_reason(case, mode, "macos-arm64"))

    def test_missing_real_provider_is_still_a_failed_native_result(self):
        case = self.ownership()
        for platform in ("macos-arm64", "linux-x64", "windows-arm64"):
            for mode in ("interp", "native"):
                self.assertIsNone(unavailable_reason(case, mode, platform))
                self.assertFalse(
                    contract_result(case, 1, "", "missing developer_debug_output")
                )
                self.assertFalse(contract_result(case, None, "", "provider timeout"))

    def test_merge_preserves_other_owners_requirements(self):
        case = self.ownership()
        case["required_capabilities"].append("separate reviewed requirement")
        case["unsupported_modes"].append("independent-mode")
        apply_debug_capabilities(case)
        self.assertIn("separate reviewed requirement", case["required_capabilities"])
        self.assertIn("independent-mode", case["unsupported_modes"])
        self.assertEqual(len(case["required_capabilities"]), 2)


if __name__ == "__main__":
    unittest.main()
