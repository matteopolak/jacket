"""Fresh/unset-Cargo CI setup controls; no native compiler or workload."""

import tempfile
from pathlib import Path
import unittest

from ci_test_scratch import assigned_scratch


class ScratchSelection(unittest.TestCase):
    def fixture(self):
        return tempfile.TemporaryDirectory(dir=assigned_scratch())

    def test_unset_cargo_and_tmpdir_use_explicit_ci_runner_scratch(self):
        with self.fixture() as root:
            base = Path(root) / "fresh-runner"
            self.assertEqual(
                assigned_scratch({"CI": "true", "RUNNER_TEMP": str(base)}), base
            )
            self.assertTrue(base.is_dir())

    def test_ci_workspace_is_explicit_when_runner_temp_is_absent(self):
        with self.fixture() as root:
            expected = Path(root) / "target/test-scratch"
            self.assertEqual(
                assigned_scratch({"CI": "true", "GITHUB_WORKSPACE": root}), expected
            )
            self.assertTrue(expected.is_dir())

    def test_explicit_local_target_or_tmpdir_is_created(self):
        with self.fixture() as root:
            for name in ("CARGO_TARGET_DIR", "TMPDIR"):
                base = Path(root) / name
                self.assertEqual(assigned_scratch({name: str(base)}), base)
                self.assertTrue(base.is_dir())

    def test_local_has_no_implicit_internal_disk_fallback(self):
        with self.assertRaisesRegex(RuntimeError, "assign CARGO_TARGET_DIR"):
            assigned_scratch({})
        with self.assertRaises(RuntimeError):
            assigned_scratch(
                {
                    "RUNNER_TEMP": "must-not-be-created",
                    "GITHUB_WORKSPACE": "must-not-be-created",
                }
            )


if __name__ == "__main__":
    unittest.main()
