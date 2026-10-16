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
