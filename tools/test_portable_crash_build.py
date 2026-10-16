"""Portable runtime build contracts; source checks, never fault execution."""

import importlib.util
import os
from pathlib import Path
import unittest


class PortableCrashBuild(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "portable_crash_build_contract", os.environ["PORTABLE_CRASH_TEST_SOURCE"]
        )
        cls.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.tool)

    def test_static_and_shared_outputs_are_built_from_one_matching_object(self):
        for system in ["macos", "linux"]:
            for cpu in ["x64", "arm64"]:
                with self.subTest(system=system, cpu=cpu):
                    plan = self.tool.build_plan(
                        system + "-" + cpu,
                        Path("/stdlib"),
                        Path("/target"),
                        "clang",
                        "llvm-ar",
                    )
                    self.assertEqual(len(plan["commands"]), 3)
                    compile_, archive, shared = plan["commands"]
                    self.assertIn("-c", compile_)
                    self.assertEqual(archive[0:2], ["llvm-ar", "rcs"])
                    self.assertIn(str(plan["object"]), archive)
                    self.assertIn(str(plan["object"]), shared)
                    self.assertEqual(plan["archive"].name, "libportable_crash.a")
                    self.assertIn("-Werror", compile_)
                    self.assertNotIn(str(plan["archive"]), shared)
                    if system == "macos":
                        self.assertEqual(
                            compile_[compile_.index("-arch") + 1],
                            "arm64" if cpu == "arm64" else "x86_64",
                        )
                        self.assertIn("-dynamiclib", shared)
                    else:
                        triple = "aarch64" if cpu == "arm64" else "x86_64"
                        self.assertIn(f"--target={triple}-unknown-linux-gnu", compile_)
                        self.assertIn("-shared", shared)

    def test_windows_is_not_substituted_with_posix(self):
        with self.assertRaisesRegex(ValueError, "POSIX"):
            self.tool.build_plan(
                "windows-arm64", Path("/stdlib"), Path("/target"), "clang", "llvm-ar"
            )


if __name__ == "__main__":
    unittest.main()
