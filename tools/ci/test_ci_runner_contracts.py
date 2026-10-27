"""Runner policy tests only; SDK/compiler/native execution is deliberately absent."""

import ast
import argparse
from pathlib import Path
import tempfile
import os
import subprocess
import sys
import unittest

from toolkit.repo import ROOT

SCRATCH = (
    os.environ.get("RUNNER_TEMP") or os.environ.get("TMPDIR") or tempfile.gettempdir()
)


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
                    str(ROOT / "tools/native/build_ci_native_bindings.py"),
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

    def test_ci_builds_only_the_verified_profile_cohort_with_real_callbacks(self):
        from native import build_ci_native_bindings

        libraries, profiles, abi = build_ci_native_bindings.selected_libraries()
        self.assertEqual(abi, 8)
        self.assertEqual(len(profiles), 23)
        for profile in profiles:
            self.assertIn(profile["sdk"], libraries)
            self.assertIn(profile["callbacks"], libraries)
        self.assertEqual(len(profiles), len(build_ci_native_bindings.PROFILES))

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


if __name__ == "__main__":
    unittest.main()
