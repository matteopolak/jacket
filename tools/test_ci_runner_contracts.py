"""Runner policy tests only; SDK/compiler/native execution is deliberately absent."""

import ast
import argparse
from pathlib import Path
import tempfile
import os
import subprocess
import sys
from unittest.mock import patch
import unittest

import stdlib_runtime
import stdlib_coverage
from stdlib_test_catalog import validated_entries, unavailable_reason

ROOT = Path(__file__).resolve().parents[1]
SCRATCH = os.environ.get("RUNNER_TEMP") or os.environ.get("TMPDIR") or tempfile.gettempdir()


def load_function(path, name, namespace):
    tree = ast.parse(path.read_text())
    definition = next(
        node
        for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name == name
    )
    exec(
        compile(ast.Module(body=[definition], type_ignores=[]), str(path), "exec"),
        namespace,
    )
    return namespace[name]


class RunnerContracts(unittest.TestCase):
    def test_native_provider_refuses_existing_output_without_building_or_deleting(self):
        with tempfile.TemporaryDirectory(dir=SCRATCH) as directory:
            marker = Path(directory) / "preserved-receipt"
            marker.write_text("prior result")
            result = subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "tools/build_ci_native_bindings.py"),
                    "--output",
                    directory,
                ],
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("must be fresh", result.stderr)
            self.assertEqual(marker.read_text(), "prior result")

    def test_sweep_missing_clipboard_guard_is_a_required_failure(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "ci_sweep_guard_policy", ROOT / "tools/jaic-sweep.py"
        )
        sweep = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(sweep)
        with patch(
            "stdlib_runtime.runtime_prefix",
            side_effect=ValueError("real restoration guard required"),
        ):
            stdout, stderr, code = sweep.run_fixture(
                ["never-executed-child"], ROOT, 1, 1024, {"clipboard_guard": True}
            )
        self.assertEqual(code, 1)
        self.assertEqual(stdout, "")
        self.assertIn("restoration guard required", stderr)

    def test_module_platform_contracts_retain_required_native_hosts(self):
        cases = {case["id"]: case for case in validated_entries()}
        for name in [
            "Window_Creation/tests/macos-window",
            "Window_Creation/tests/macos-events",
            "Simp/tests/context-default-assertions",
        ]:
            self.assertIsNotNone(
                unavailable_reason(cases[name], "native", "windows-x64")
            )
            self.assertIsNone(unavailable_reason(cases[name], "native", "macos-arm64"))
        self.assertIsNotNone(
            unavailable_reason(
                cases["tests/portable-data-resource-windows"], "native", "macos-arm64"
            )
        )

    def test_real_resource_provider_is_required_instead_of_an_empty_directory(self):
        with self.assertRaisesRegex(ValueError, "JAIC_TEST_WINDOWS_RESOURCE_DIR"):
            stdlib_runtime.program_arguments(
                {"input": "windows-resource-directory"}, ".", {}
            )

    def test_font_provider_is_required(self):
        with self.assertRaisesRegex(ValueError, "JAIC_TEST_FONT_PATH"):
            stdlib_runtime.program_arguments({"input": "font"}, ".", {})

    def test_probe_wrapper_preserves_source_and_binds_the_actual_library(self):
        with tempfile.TemporaryDirectory(dir=SCRATCH) as directory:
            source = Path(directory) / "original.jai"
            body = "main :: () {}\n"
            source.write_text(body)
            case = {
                "id": "unit/probe",
                "fixture_libraries": {"GL_PROBE": "window_gl_probe"},
            }
            wrapper = stdlib_runtime.prepare_program(case, source, directory)
            self.assertEqual(source.read_text(), body)
            self.assertIn(
                'GL_PROBE :: #library "window_gl_probe";', wrapper.read_text()
            )
            self.assertIn(str(source.resolve()), wrapper.read_text())
            self.assertNotEqual(wrapper, source)

    def test_sanitizer_module_discovery_uses_canonical_fixture_metadata(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "ci_sweep_policy", ROOT / "tools/jaic-sweep.py"
        )
        sweep = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(sweep)
        list(sweep.cases("modules"))
        self.assertEqual(sweep.CASE_METADATA["Simp/tests/glyph-atlas"]["input"], "font")
        self.assertEqual(
            sweep.CASE_METADATA["Simp/tests/context-default-assertions"][
                "fixture_libraries"
            ],
            {"GL_PROBE": "window_gl_probe"},
        )
        self.assertEqual(
            sweep.CASE_METADATA["tests/portable-data-resource-windows"]["required_os"],
            "windows",
        )
        self.assertNotEqual(sweep.PLATFORM_UNAVAILABLE, sweep.HEADLESS_BUILD)

    def test_ci_builds_only_the_verified_profile_cohort_with_real_callbacks(self):
        import build_ci_native_bindings

        libraries, profiles, abi = build_ci_native_bindings.selected_libraries()
        self.assertEqual(abi, 8)
        self.assertEqual(len(profiles), 23)
        for profile in profiles:
            self.assertIn(profile["sdk"], libraries)
            self.assertIn(profile["callbacks"], libraries)
        self.assertEqual(len(profiles), len(build_ci_native_bindings.PROFILES))

    def test_failed_or_empty_run_has_a_real_empty_coverage_record(self):
        with tempfile.TemporaryDirectory(dir=SCRATCH) as directory:
            path = Path(directory) / "nested/coverage.txt"
            stdlib_runtime.initialize_coverage(path)
            self.assertTrue(path.is_file())
            self.assertEqual(path.read_bytes(), b"")
            self.assertEqual(stdlib_coverage.read_record(path), (set(), set()))

    def test_new_run_does_not_inherit_prior_hits(self):
        with tempfile.TemporaryDirectory(dir=SCRATCH) as directory:
            path = Path(directory) / "coverage.txt"
            path.write_text("previous run hits")
            stdlib_runtime.initialize_coverage(path)
            self.assertEqual(path.read_bytes(), b"")

    def test_bindings_parser_accepts_production_paths_without_staging_flags(self):
        parser = load_function(
            ROOT / "tools/native_bindings/verify.py",
            "argument_parser",
            {
                "argparse": argparse,
                "Path": Path,
                "ROOT": ROOT,
                "MODULES": {},
                "__doc__": "policy test",
            },
        )()
        args = parser.parse_args(
            [
                "--compiler",
                "/own/jaic",
                "--stdlib",
                "/own/stdlib",
                "--native-libs",
                "/own/native",
                "--work-dir",
                "/own/work",
            ]
        )
        self.assertEqual(args.stdlib, Path("/own/stdlib"))
        self.assertEqual(args.native_libs, Path("/own/native"))
        self.assertIsNone(args.existing_stdlib)
        self.assertIsNone(args.foundation)

    def evidence(self):
        namespace = {"UNAVAILABLE_EVIDENCE": [], "json": __import__("json")}
        return load_function(
            ROOT / "tools/check_developer_tools.py", "host_evidence", namespace
        ), namespace

    def test_mac_recipe_on_linux_is_unavailable_and_never_called_or_passed(self):
        run, namespace = self.evidence()
        called = []
        run("actual Mac SDK", lambda: called.append(True), ["Darwin"], host="Linux")
        self.assertEqual(called, [])
        self.assertIsNone(namespace["UNAVAILABLE_EVIDENCE"][0]["passed"])
        self.assertEqual(namespace["UNAVAILABLE_EVIDENCE"][0]["status"], "unavailable")

    def test_required_host_failure_propagates(self):
        run, namespace = self.evidence()

        def fail():
            raise RuntimeError("real fixture failure")

        with self.assertRaisesRegex(RuntimeError, "real fixture failure"):
            run("actual Mac SDK", fail, ["Darwin"], host="Darwin")
        self.assertEqual(namespace["UNAVAILABLE_EVIDENCE"], [])

    def test_supported_host_evidence_executes_once(self):
        run, namespace = self.evidence()
        called = []
        run(
            "host recipe",
            lambda: called.append(True),
            ["Darwin", "Linux"],
            host="Linux",
        )
        self.assertEqual(called, [True])
        self.assertEqual(namespace["UNAVAILABLE_EVIDENCE"], [])


if __name__ == "__main__":
    unittest.main()
