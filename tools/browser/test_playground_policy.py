"""Canonical browser dispatch policy: no compiler execution or diagnostic inference."""

from pathlib import Path
import unittest
from unittest.mock import patch
import playground_policy as browser


class BrowserPolicy(unittest.TestCase):
    def test_compiler_host_clock_is_declared_before_dispatch_without_target_deferral(self):
        cases = {case["id"]: case for case in browser.browser_cases()}
        declared = [c for c in cases.values() if c["required_browser_host_capabilities"]]
        self.assertTrue(declared)
        for case in declared:
            self.assertRegex(
                case["browser_unavailable_reason"],
                "compiler host unavailable|platform unavailable",
            )
        self.assertIn(
            "compiler-monotonic-clock",
            cases["compiler/compiler-performance-and-dump-messages"]["required_browser_host_capabilities"],
        )
        negative = cases["compiler/negative/inserted-procedure-called"]
        self.assertIsNone(negative["browser_unavailable_reason"])
        self.assertEqual(negative["mode"], "check-fail")

    def test_native_sdk_policy_is_exact_not_a_module_wide_deferral(self):
        root = Path("/own/workspace")
        case = {
            "id": "Bindings_Generator/tests/unclassified-control",
            "family": "modules",
            "path": root / "stdlib/Bindings_Generator/tests/control.jai",
        }
        with patch.object(browser, "validated_entries", return_value=[case]):
            self.assertIsNone(browser.browser_cases(root)[0]["browser_unavailable_reason"])

    def test_explicit_capability_os_cpu_and_family_policy_precedes_execution(self):
        root = Path("/own/workspace")
        for policy in (
            {
                "unsupported_modes": ["wasm-interp"],
                "required_capabilities": ["native service"],
            },
            {"required_os": "windows"},
            {"required_cpu": "x64"},
        ):
            case = {
                "id": "example",
                "family": "compiler",
                "path": root / "main.jai",
                **policy,
            }
            with (
                self.subTest(policy=policy),
                patch.object(browser, "validated_entries", return_value=[case]),
            ):
                self.assertTrue(
                    browser.browser_cases(root)[0]["browser_unavailable_reason"]
                )

    def test_actual_admitted_catalogue_preserves_controls_and_required_negatives(self):
        cases = {case["id"]: case for case in browser.browser_cases()}
        negative = cases["compiler/negative/inserted-procedure-called"]
        self.assertEqual(negative["mode"], "check-fail")
        self.assertIsNone(negative["browser_unavailable_reason"])
        self.assertIn("Missing_Buffer", negative["diagnostic_contains"])
        self.assertIsNone(
            cases["compiler/foundations-portable-compile-time"]["browser_unavailable_reason"]
        )
        # Native services are declared per case, not for a whole family.
        self.assertTrue(
            cases["compiler/socket-select-canonical-timeval"]["browser_unavailable_reason"]
        )


if __name__ == "__main__":
    unittest.main()
