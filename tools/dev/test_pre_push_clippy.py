"""Tests for the pre-push clippy hook's package selection."""

import subprocess
import unittest
from unittest import mock

import pre_push_clippy


class PackagesTests(unittest.TestCase):
    def test_non_rust_changes_need_no_cargo(self):
        with mock.patch("shutil.which", return_value=None):
            self.assertEqual(pre_push_clippy.packages({"docs/a.md"}), [])

    def test_missing_cargo_fails_with_an_install_hint(self):
        with mock.patch("shutil.which", return_value=None):
            with self.assertRaisesRegex(SystemExit, "cargo not found: install Rust"):
                pre_push_clippy.packages({"crates/jaic/src/main.rs"})

    def test_failed_metadata_fails(self):
        failed = subprocess.CompletedProcess([], 101, "", "no lockfile")
        with mock.patch("shutil.which", return_value="/bin/cargo"), mock.patch("subprocess.run", return_value=failed):
            with self.assertRaisesRegex(SystemExit, "(?s)cargo metadata failed.*no lockfile"):
                pre_push_clippy.packages({"Cargo.toml"})


class NoCargoTests(unittest.TestCase):
    def test_opt_in_skips_clippy_without_reading_refs(self):
        with mock.patch.dict("os.environ", {"JACKET_HOOKS_NO_CARGO": "1"}), mock.patch("sys.stdin") as stdin:
            self.assertEqual(pre_push_clippy.main(), 0)
        stdin.read.assert_not_called()


if __name__ == "__main__":
    unittest.main()
