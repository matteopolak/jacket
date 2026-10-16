"""Target provider construction policies; actual cross links are separate proof."""

import unittest
from pathlib import Path
from unittest.mock import patch

import build_windows_binding_providers as providers


class TargetProviders(unittest.TestCase):
    def test_complete_profile_and_counter_inventory(self):
        plan = providers.provider_plan()
        self.assertEqual(len(plan), 52)
        self.assertEqual(len({item["name"] for item in plan}), 52)
        names = {item["name"] for item in plan}
        self.assertTrue(
            {
                "binding_callback_context",
                "binding_rpmalloc_sdk",
                "binding_rpmalloc_callbacks",
                "rpmalloc_callback_probe",
                "rpmalloc_live_config_probe",
                "rpmalloc_adaptive_probe",
                "rpmalloc_sdk_probe",
                "rpmalloc_storage_probe",
            }
            <= names
        )
        for item in plan:
            self.assertTrue(item["units"])
            for unit, _ in item["units"]:
                self.assertTrue(Path(unit).is_file())

    def test_target_machine_is_checked_without_host_fallback(self):
        for platform, machine in [
            ("windows-x64-mingw", "IMAGE_FILE_MACHINE_AMD64"),
            ("windows-arm64-mingw", "IMAGE_FILE_MACHINE_ARM64"),
        ]:
            providers.verify_machine("Machine: " + machine, platform)
            with self.assertRaisesRegex(ValueError, "COFF machine"):
                providers.verify_machine("Machine: IMAGE_FILE_MACHINE_I386", platform)
        with self.assertRaisesRegex(ValueError, "unsupported"):
            providers.toolchain("linux-arm64")

    def test_cross_compiler_and_archiver_are_target_tools(self):
        with patch.object(
            providers.shutil, "which", side_effect=lambda name: "/sdk/" + name
        ):
            compiler, archiver, _ = providers.toolchain("windows-arm64-mingw")
        self.assertIn("aarch64-w64-mingw32", compiler[0])
        self.assertIn("aarch64-w64-mingw32", archiver)
        self.assertNotIn("-w", compiler)


if __name__ == "__main__":
    unittest.main()
