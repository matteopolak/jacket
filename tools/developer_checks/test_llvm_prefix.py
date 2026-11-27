"""Tests for finding the LLVM SDK prefix the developer tools record."""

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from developer_checks import build_developer_bindings


def sdk(root: Path) -> Path:
    (root / "include/clang-c").mkdir(parents=True)
    (root / "include/clang-c/Index.h").write_text("")
    (root / "bin").mkdir()
    (root / "bin/llvm-config").write_text("")
    return root


class LlvmPrefixTests(unittest.TestCase):
    def test_keeps_the_stable_folder_of_the_llvm_config_it_found(self):
        # A package manager's stable link (opt/llvm) points at a versioned folder that an upgrade
        # removes; the recorded tool paths must keep working after the upgrade.
        with tempfile.TemporaryDirectory() as directory:
            versioned = sdk(Path(directory) / "Cellar/llvm/23.1.2")
            stable = Path(directory) / "opt/llvm"
            stable.parent.mkdir(parents=True)
            stable.symlink_to(versioned)
            with (
                mock.patch.dict("os.environ", {}, clear=True),
                mock.patch.object(build_developer_bindings.llvm, "configured_prefix", return_value=None),
                mock.patch.object(build_developer_bindings.llvm, "llvm_config", return_value=stable / "bin/llvm-config"),
                mock.patch("subprocess.check_output", return_value=f"{versioned}\n"),
            ):
                self.assertEqual(build_developer_bindings.llvm_prefix(), stable)

    def test_uses_the_reported_prefix_when_llvm_config_is_linked_from_elsewhere(self):
        with tempfile.TemporaryDirectory() as directory:
            real = sdk(Path(directory) / "lib/llvm-23")
            linked = Path(directory) / "bin/llvm-config-23"
            linked.parent.mkdir(parents=True)
            linked.symlink_to(real / "bin/llvm-config")
            with (
                mock.patch.dict("os.environ", {}, clear=True),
                mock.patch.object(build_developer_bindings.llvm, "configured_prefix", return_value=None),
                mock.patch.object(build_developer_bindings.llvm, "llvm_config", return_value=linked),
                mock.patch("subprocess.check_output", return_value=f"{real}\n"),
            ):
                self.assertEqual(build_developer_bindings.llvm_prefix(), real)


if __name__ == "__main__":
    unittest.main()
