"""Canonical browser dispatch policy: no compiler execution or diagnostic inference."""

from pathlib import Path
import unittest
from unittest.mock import patch
import playground_policy as browser


class BrowserPolicy(unittest.TestCase):
    def test_compiler_host_clock_is_declared_before_dispatch_without_target_deferral(self):
        cases = {case["id"]: case for case in browser.browser_cases()}
        for name, required in browser.BROWSER_HOST_REQUIREMENTS.items():
            self.assertEqual(
                cases[name]["required_browser_host_capabilities"], list(required)
            )
            # Declared unavailable before dispatch, by the host capability or an earlier catalogue rule.
            self.assertIn("unavailable", cases[name]["browser_unavailable_reason"])
        # Needs native Io providers, not a compiler host capability.
        self.assertNotIn(
            "compiler host",
            cases["extensions/http-router-ownership"]["browser_unavailable_reason"] or "",
        )
        self.assertIsNone(cases["extensions/http-date-now-wasm-negative"]["browser_unavailable_reason"])
        self.assertEqual(cases["extensions/http-date-now-wasm-negative"]["mode"], "check-fail")

    def test_native_sdk_policy_is_exact_not_a_module_wide_deferral(self):
        root = Path("/own/workspace")
        case = {
            "id": "Bindings_Generator/tests/unclassified-control",
            "family": "modules",
            "path": root / "stdlib/Bindings_Generator/tests/control.jai",
        }
        with patch.object(browser, "validated_entries", return_value=[case]):
            self.assertIsNone(browser.browser_cases(root)[0]["browser_unavailable_reason"])

    def test_mode_overlay_and_eligibility_use_the_shared_catalogue(self):
        root = Path("/own/workspace")
        original = {
            "id": "example",
            "family": "compiler",
            "path": root / "main.jai",
            "mode_contracts": {
                "wasm-interp": {
                    "mode": "check-fail",
                    "diagnostic_contains": "precise rejection",
                }
            },
            "peer_metadata": {"preserved": True},
        }
        with patch.object(browser, "validated_entries", return_value=[original]):
            selected = browser.browser_cases(root)[0]
        self.assertEqual(selected["mode"], "check-fail")
        self.assertEqual(selected["diagnostic_contains"], "precise rejection")
        self.assertIsNone(selected["browser_unavailable_reason"])
        self.assertEqual(selected["path"], "main.jai")
        self.assertEqual(selected["peer_metadata"], {"preserved": True})
        self.assertNotIn("mode", original)

    def test_explicit_capability_os_cpu_and_family_policy_precedes_execution(self):
        root = Path("/own/workspace")
        for policy in (
            {
                "unsupported_modes": ["wasm-interp"],
                "required_capabilities": ["native service"],
            },
            {"required_os": "windows"},
            {"required_cpu": "x64"},
            {"family": "native"},
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
        negative = cases["extensions/http-date-now-wasm-negative"]
        self.assertEqual(negative["mode"], "check-fail")
        self.assertIsNone(negative["browser_unavailable_reason"])
        self.assertIn("http_date_now", negative["diagnostic_contains"])
        for name in (
            "extensions/http-date-target",
            "tests/portable-data-resource-wire",
            "compiler/foundations-portable-compile-time",
        ):
            self.assertIsNone(cases[name]["browser_unavailable_reason"], name)
        self.assertTrue(
            cases["tests/portable-data-resource"]["browser_unavailable_reason"]
        )

    def test_invalid_mode_contracts_are_errors_not_unavailable(self):
        case = {
            "id": "example",
            "family": "compiler",
            "path": Path("/own/main.jai"),
            "mode_contracts": {"wasm-interp": {"mode": "check-fail"}},
        }
        with (
            patch.object(browser, "validated_entries", return_value=[case]),
            self.assertRaises(ValueError),
        ):
            browser.browser_cases(Path("/own"))


if __name__ == "__main__":
    unittest.main()
