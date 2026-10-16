"""Catalogue/ASan dispatch policy only; mocked dispatch is not native proof."""

import contextlib
import importlib.util
import io
import os
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

from stdlib_test_catalog import unavailable_reason, validated_entries

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    "GetRect/tests/native_clipboard": {"macos"},
    "GetRect/tests/native_cursors": {"macos", "windows"},
    "GetRect/tests/clipboard_unavailable": {"linux", "windows", "wasm"},
    "GetRect/tests/cursors_unavailable": {"linux", "wasm"},
}


def load_sweep():
    spec = importlib.util.spec_from_file_location(
        "numerics_asan_dispatch_test", ROOT / "tools/jaic-sweep.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class NumericsCaseCapabilities(unittest.TestCase):
    def setUp(self):
        self.cases = {case["id"]: case for case in validated_entries()}

    def test_complete_module_inventory_and_all_execution_modes(self):
        self.assertEqual(
            {name for name in self.cases if name.startswith("GetRect/tests/")},
            set(EXPECTED),
        )
        for name, targets in EXPECTED.items():
            for target in ("linux", "macos", "windows"):
                for cpu in ("x64", "arm64"):
                    for mode in ("interp", "native"):
                        with self.subTest(name=name, target=target, cpu=cpu, mode=mode):
                            reason = unavailable_reason(
                                self.cases[name], mode, f"{target}-{cpu}"
                            )
                            self.assertEqual(reason is None, target in targets)
                            if reason:
                                self.assertIn("not passed", reason)
            for mode in ("wasm-interp", "wasm-native"):
                with self.subTest(name=name, mode=mode):
                    self.assertEqual(
                        unavailable_reason(self.cases[name], mode, "macos-arm64")
                        is None,
                        "wasm" in targets,
                    )

    def test_malformed_target_or_mode_is_configuration_failure(self):
        case = self.cases["GetRect/tests/native_clipboard"]
        for platform in (
            None,
            "",
            "darwin-arm64",
            "unknown-x64",
            "macos",
            "macos-wrong",
            "macos-arm64-extra",
        ):
            with self.subTest(platform=platform):
                with self.assertRaises(ValueError):
                    unavailable_reason(case, "native", platform)
        with self.assertRaises(ValueError):
            unavailable_reason(case, "wasm-native-typo", "macos-arm64")

    def test_unknown_contract_marker_fails_instead_of_defaulting_to_unavailable(self):
        case = {
            **self.cases["GetRect/tests/native_clipboard"],
            "numerics_target_source": "stdlib/GetRect/tests/unreviewed.jai",
        }
        with self.assertRaisesRegex(ValueError, "uninventoried numerics"):
            unavailable_reason(case, "native", "macos-arm64")

    def test_unrelated_cases_and_missing_providers_remain_required(self):
        for name in (
            "contracts/math-vector-float16-edge-vectors",
            "Extensions/Byte_Search/tests/search",
        ):
            self.assertIsNone(
                unavailable_reason(self.cases[name], "native", "macos-arm64")
            )
        for name in EXPECTED:
            self.assertNotIn("unsupported_modes", self.cases[name])

    def sweep(self, platform, opt, dispatch_result):
        sweep = load_sweep()
        output = io.StringIO()
        with (
            patch.object(
                sys,
                "argv",
                [
                    "jaic-sweep",
                    "--jaic",
                    "/never-executed-policy-test",
                    "--jobs",
                    "1",
                    "--sanitize",
                    "address,undefined",
                    "--opt",
                    opt,
                    "--headless",
                    "--filter",
                    "GetRect/tests/",
                    "modules",
                ],
            ),
            patch.dict(os.environ, dict(os.environ)),
            patch.object(sweep, "use_native_libs"),
            patch.object(sweep, "stale_sources", return_value=None),
            patch.object(sweep, "symbolizer", return_value=None),
            patch("stdlib_runtime.host_platform", return_value=platform),
            patch.object(sweep, "run_native", return_value=dispatch_result) as dispatch,
            contextlib.redirect_stdout(output),
            self.assertRaises(SystemExit) as exit_status,
        ):
            sweep.main()
        return exit_status.exception.code, output.getvalue(), dispatch.call_args_list

    def test_actual_sanitizer_selector_uses_contracts_before_dispatch(self):
        for platform in ("macos-arm64", "linux-x64", "linux-arm64", "windows-x64"):
            target = platform.split("-")[0]
            for opt in ("O0", "O2"):
                with self.subTest(platform=platform, opt=opt):
                    status, output, calls = self.sweep(platform, opt, ("", "", 0))
                    self.assertEqual(status, 0)
                    selected = {"GetRect/tests/" + call.args[1].stem for call in calls}
                    self.assertEqual(
                        selected,
                        {
                            name
                            for name, targets in EXPECTED.items()
                            if target in targets
                        },
                    )
                    for call in calls:
                        self.assertEqual(
                            call.args[3], ["-sanitize", "address,undefined", "-" + opt]
                        )
                    self.assertIn("2 passed, 0 failed, 2 platform unavailable", output)
                    for name in set(EXPECTED) - selected:
                        self.assertIn("UNAVAILABLE " + name + ":", output)

    def test_real_provider_failures_are_not_availability_decisions(self):
        for provider in ("clang", "developer_tools_debug"):
            status, output, calls = self.sweep(
                "macos-arm64",
                "O0",
                ("", "error: required native library " + provider + " unavailable", 1),
            )
            self.assertEqual(status, 1)
            self.assertEqual(len(calls), 2)
            self.assertIn("0 passed, 2 failed, 2 platform unavailable", output)
            self.assertIn("FAIL GetRect/tests/native_clipboard:", output)
            self.assertIn(provider, output)


if __name__ == "__main__":
    unittest.main()
