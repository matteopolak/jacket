"""Selection regressions: unavailable is distinct from successful execution."""

import unittest

from stdlib_test_catalog import unavailable_reason, validated_entries


class FoundationsCapabilitiesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cases = {case["id"]: case for case in validated_entries()}

    def test_native_exit_assertion_paths_are_required_on_native(self):
        for name in (
            "compiler/type-field-constant",
            "compiler/discard-parameters",
            "Extensions/Args/tests/parse",
        ):
            case = self.cases[name]
            self.assertIsNone(unavailable_reason(case, "native", "macos-arm64"))
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIn(
                    "Basic.exit", unavailable_reason(case, mode, "macos-arm64")
                )

    def test_explicit_wasm_termination_and_portable_cases_are_preserved(self):
        for name in (
            "extensions/http-codec-roundtrip",
            "extensions/http-wasm-termination-status",
            "extensions/http-wasm-termination-failure",
            "compiler/for-loop-bounds",
            "compiler/asm-float-variables",
            "contracts/hash-private-seed-is-not-exported",
        ):
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNone(
                    unavailable_reason(self.cases[name], mode, "macos-arm64"), name
                )

    def test_native_fibers_and_sdl_have_exact_case_requirements(self):
        for name in (
            "extensions/io-fibers-context",
            "extensions/io-allocator-zero",
            "Extensions/Io/tests/pipe",
            "SDL/tests/sdl-layout",
        ):
            self.assertIsNotNone(
                unavailable_reason(self.cases[name], "wasm-native", "macos-arm64"), name
            )
            self.assertIsNone(
                unavailable_reason(self.cases[name], "native", "macos-arm64"), name
            )
        self.assertIsNone(
            unavailable_reason(self.cases["Extensions/Http/tests/codec"], "wasm-native")
        )

    def test_x64_assembly_is_not_counted_as_arm64_or_wasm_coverage(self):
        case = self.cases["compiler/lang-asm"]
        self.assertIn("x64", unavailable_reason(case, "native", "macos-arm64"))
        self.assertIsNone(unavailable_reason(case, "native", "linux-x64"))
        self.assertIsNotNone(unavailable_reason(case, "wasm-native", "linux-x64"))

    def test_unknown_compiler_failure_does_not_define_a_capability(self):
        case = {"id": "compiler/new-portable-case", "family": "compiler"}
        self.assertIsNone(unavailable_reason(case, "wasm-native", "macos-arm64"))

    def test_user_memory_uring_protocol_is_not_restricted_to_a_linux_kernel(self):
        case = self.cases["extensions/io-uring-protocol"]
        self.assertIsNone(unavailable_reason(case, "native", "macos-arm64"))
        self.assertIn(
            "linux",
            unavailable_reason(
                self.cases["extensions/io-uring-raw"], "native", "macos-arm64"
            ),
        )

    def test_explicit_case_inventory_stays_bound_to_real_sources(self):
        from foundations_case_capabilities import (
            LINUX_URING_CASES,
            NATIVE_EXIT_CASES,
            NATIVE_FIBER_CASES,
            NATIVE_SDL_CASES,
            X64_CASES,
        )

        required = (
            LINUX_URING_CASES
            | NATIVE_EXIT_CASES
            | NATIVE_FIBER_CASES
            | NATIVE_SDL_CASES
            | X64_CASES
        )
        self.assertFalse(required - self.cases.keys())

    def test_reviewed_native_dependencies_are_exact_and_still_native_required(self):
        from foundations_case_capabilities import NATIVE_SERVICE_CASES

        self.assertFalse(NATIVE_SERVICE_CASES.keys() - self.cases.keys())
        for name in NATIVE_SERVICE_CASES:
            case = self.cases[name]
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNotNone(
                    unavailable_reason(case, mode, "macos-arm64"), name
                )
            platform = (
                "windows-x64" if case.get("required_os") == "windows" else "macos-arm64"
            )
            for mode in ("interp", "native"):
                self.assertIsNone(unavailable_reason(case, mode, platform), name)

    def test_explicit_seconds_and_wire_controls_remain_portable(self):
        for name in (
            "extensions/http-date-target",
            "tests/portable-data-resource-wire",
            "compiler/foundations-portable-compile-time",
            "contracts/bootstrap-atomic-scalar-intrinsics",
            "Extensions/Http/tests/codec",
        ):
            for mode in ("interp", "native", "wasm-interp", "wasm-native"):
                self.assertIsNone(
                    unavailable_reason(self.cases[name], mode, "macos-arm64")
                )

    def test_required_wasm_rejection_is_not_an_unavailable_or_a_native_negative(self):
        from stdlib_test_catalog import case_for_mode
        from stdlib_runtime import contract_result

        original = self.cases["extensions/http-date-now-wasm-negative"]
        self.assertNotIn("mode", original)
        for mode in ("wasm-interp", "wasm-native"):
            case = case_for_mode(original, mode)
            self.assertEqual(case["mode"], "check-fail")
            self.assertIsNone(unavailable_reason(case, mode, "macos-arm64"))
            diagnostic = case["diagnostic_contains"]
            self.assertTrue(contract_result(case, 1, "", diagnostic))
            self.assertFalse(contract_result(case, 0, "", diagnostic))
            self.assertFalse(
                contract_result(case, 1, "", "unknown unrelated identifier")
            )
            self.assertFalse(contract_result(case, None, "", diagnostic))
        self.assertNotIn("mode", case_for_mode(original, "native"))

    def test_windows_provider_reproductions_do_not_remove_general_tls_coverage(self):
        for name in (
            "extensions/tls-windows-target",
            "extensions/json-windows-host-provider-repro",
        ):
            case = self.cases[name]
            self.assertEqual(case["required_os"], "windows")
            self.assertIsNone(unavailable_reason(case, "native", "windows-arm64"))
            self.assertIsNotNone(unavailable_reason(case, "native", "linux-x64"))
        self.assertIsNone(
            unavailable_reason(
                self.cases["Extensions/Tls/tests/tls"], "native", "linux-x64"
            )
        )

    def test_mode_contracts_do_not_mutate_shared_cases_or_swallow_invalid_policy(self):
        from stdlib_test_catalog import case_for_mode

        original = self.cases["extensions/http-date-now-wasm-negative"]
        case = case_for_mode(original, "wasm-native")
        case["mode"] = "changed"
        self.assertNotIn("mode", original)
        with self.assertRaises(ValueError):
            case_for_mode(
                {"mode_contracts": {"wasm-native": {"mode": "check-fail"}}},
                "wasm-native",
            )
        conflict = {**original, "unsupported_modes": ["wasm-native"]}
        with self.assertRaises(ValueError):
            unavailable_reason(conflict, "wasm-native", "macos-arm64")


class PolicyPreservation(unittest.TestCase):
    def test_reviewed_residual_native_cases_keep_independent_portable_controls(self):
        cases = {case["id"]: case for case in validated_entries()}
        for name in (
            "contracts/overwriting-allocator-fill-and-teardown",
            "contracts/sound-player-pure-public-contracts",
            "extensions/http-native-date-target",
        ):
            for mode in ("interp", "native"):
                self.assertIsNone(unavailable_reason(cases[name], mode, "linux-x64"))
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNotNone(unavailable_reason(cases[name], mode, "linux-x64"))
        for name in (
            "contracts/adpcm-public-constants-and-saturation",
            "extensions/http-date-target",
            "tests/portable-data-resource-wire",
        ):
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNone(unavailable_reason(cases[name], mode, "linux-x64"))

    def test_linux_x64_regressions_have_no_new_backend_deferral(self):
        cases = {case["id"]: case for case in validated_entries()}
        for name in ("compiler/lang-asm", "compiler/rules-simd-asm"):
            for mode in ("interp", "native"):
                self.assertIsNone(unavailable_reason(cases[name], mode, "linux-x64"))

    def test_other_owners_metadata_and_unrelated_negative_contracts_survive(self):
        from foundations_case_capabilities import apply_foundations_capabilities
        from stdlib_test_catalog import case_for_mode

        case = {
            "id": "compiler/threads-allocators",
            "family": "compiler",
            "required_capabilities": ["peer-provider"],
            "unsupported_modes": ["peer-mode"],
            "peer_marker": {"keep": True},
        }
        apply_foundations_capabilities(case)
        apply_foundations_capabilities(case)
        self.assertIn("peer-mode", case["unsupported_modes"])
        self.assertEqual(case["required_capabilities"].count("peer-provider"), 1)
        self.assertEqual(case_for_mode(case, "native")["peer_marker"], {"keep": True})
        private = next(
            c
            for c in validated_entries()
            if c["id"] == "contracts/hash-private-seed-is-not-exported"
        )
        for mode in ("interp", "native", "wasm-interp", "wasm-native"):
            self.assertEqual(case_for_mode(private, mode)["mode"], "check-fail")
            self.assertIsNone(unavailable_reason(private, mode, "macos-arm64"))


class CrossToolchainCpu(unittest.TestCase):
    def test_toolchain_suffix_does_not_change_actual_cpu(self):
        from stdlib_test_catalog import unavailable_reason

        case = {"id": "compiler/lang-asm", "family": "compiler", "required_cpu": "x64"}
        for platform in ("windows-x64", "windows-x64-mingw", "linux-x64", "macos-x64"):
            with self.subTest(platform=platform):
                self.assertIsNone(unavailable_reason(case, "native", platform))
        self.assertIsNotNone(unavailable_reason(case, "native", "windows-arm64-mingw"))


if __name__ == "__main__":
    unittest.main()
