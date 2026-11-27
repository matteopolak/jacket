"""Tests for the pre-push clippy hook's package selection."""

import io
import subprocess
import tempfile
import unittest
from pathlib import Path
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


class StdlibTests(unittest.TestCase):
    def test_stdlib_sources_and_golden_outputs_need_their_tests(self):
        files = {"stdlib/Basic/module.jai", "stdlib/Csv/tests/a.stdout", "stdlib/Csv/data.bin", "docs/a.md"}
        with mock.patch("shutil.which", return_value=None):
            self.assertEqual(pre_push_clippy.required(files), [pre_push_clippy.STDLIB_TESTS])
        self.assertEqual(pre_push_clippy.stdlib_files(files), ["stdlib/Basic/module.jai", "stdlib/Csv/tests/a.stdout"])

    def test_a_change_runs_the_tests_of_its_module(self):
        files = ["stdlib/Print_Vars/tests/log-vars.jai", "stdlib/Windows_Utf8/tests/basics/main.jai",
                 "stdlib/Extensions/Json/module.jai", "stdlib/Extensions/Json/tests/basics.stdout"]
        self.assertEqual(pre_push_clippy.stdlib_roots(files),
                         ["stdlib/Extensions/Json", "stdlib/Print_Vars", "stdlib/Windows_Utf8"])

    def test_a_stdlib_push_needs_a_stamp_with_its_tests(self):
        line = "refs/heads/x abc refs/heads/x " + "0" * 40
        tests = pre_push_clippy.STDLIB_TESTS
        with (
            mock.patch.object(pre_push_clippy, "changed_files", return_value={"stdlib/Basic/module.jai"}),
            mock.patch.object(pre_push_clippy, "tree_of", return_value="T"),
        ):
            self.assertEqual(pre_push_clippy.unlinted([line], {"T": set()}), [("abc", [tests])])
            self.assertEqual(pre_push_clippy.unlinted([line], {"T": {tests}}), [])


class NoCargoTests(unittest.TestCase):
    def test_opt_in_skips_clippy_without_reading_refs(self):
        with mock.patch.dict("os.environ", {"JACKET_HOOKS_NO_CARGO": "1"}), mock.patch("sys.stdin") as stdin:
            self.assertEqual(pre_push_clippy.main(), 0)
        stdin.read.assert_not_called()


class ChangedFilesTests(unittest.TestCase):
    def test_a_rebased_push_counts_only_the_branch_changes(self):
        with tempfile.TemporaryDirectory() as dir:
            def run(*args):
                return subprocess.run(["git", *args], cwd=dir, check=True, capture_output=True, text=True).stdout.strip()

            def commit(name):
                Path(dir, name).write_text(name)
                run("add", name)
                run("commit", "-q", "-m", name)
                return run("rev-parse", "HEAD")

            run("init", "-q", "-b", "main")
            run("config", "user.email", "t@t")
            run("config", "user.name", "t")
            run("config", "commit.gpgsign", "false")
            commit("base.rs")
            run("checkout", "-q", "-b", "x")
            pushed = commit("branch.rs")
            run("checkout", "-q", "main")
            commit("main.rs")
            run("update-ref", "refs/remotes/origin/main", "main")
            run("checkout", "-q", "-b", "rebased")
            rebased = commit("branch.rs")
            with mock.patch.object(pre_push_clippy, "ROOT", Path(dir)):
                files = pre_push_clippy.changed_files([f"refs/heads/x {rebased} refs/heads/x {pushed}"])
                self.assertEqual(files, {"branch.rs"})
                plain = pre_push_clippy.changed_files([f"refs/heads/x {pushed} refs/heads/x {run('rev-parse', 'main~1')}"])
                self.assertEqual(plain, {"branch.rs"})


class StampTests(unittest.TestCase):
    def test_stamps_round_trip_and_keep_the_latest(self):
        with tempfile.TemporaryDirectory() as dir:
            path = Path(dir) / "ok"
            self.assertEqual(pre_push_clippy.read_stamps(path), {})
            for n in range(pre_push_clippy.KEPT_STAMPS + 3):
                pre_push_clippy.write_stamp(path, f"t{n}", ["b", "a"])
            stamps = pre_push_clippy.read_stamps(path)
            self.assertEqual(len(stamps), pre_push_clippy.KEPT_STAMPS)
            self.assertNotIn("t0", stamps)
            self.assertEqual(stamps["t22"], {"a", "b"})

    def test_a_push_needs_a_stamp_covering_its_packages(self):
        line = "refs/heads/x abc refs/heads/x " + "0" * 40
        with (
            mock.patch.object(pre_push_clippy, "changed_files", return_value={"f"}),
            mock.patch.object(pre_push_clippy, "packages", return_value=["jaic", "jacket"]),
            mock.patch.object(pre_push_clippy, "tree_of", return_value="T"),
        ):
            self.assertEqual(pre_push_clippy.unlinted([line], {}), [("abc", ["jaic", "jacket"])])
            self.assertEqual(pre_push_clippy.unlinted([line], {"T": {"jaic"}}), [("abc", ["jaic", "jacket"])])
            self.assertEqual(pre_push_clippy.unlinted([line], {"T": {"jaic", "jacket", "x"}}), [])
            self.assertEqual(pre_push_clippy.unlinted([line], {"other": {"jaic", "jacket"}}), [("abc", ["jaic", "jacket"])])

    def test_the_hook_never_runs_cargo(self):
        line = "refs/heads/x abc refs/heads/x " + "0" * 40
        with (
            mock.patch.object(pre_push_clippy, "unlinted", return_value=[("abc", ["jaic"])]),
            mock.patch.object(pre_push_clippy, "stamp_file", return_value=Path("/nonexistent")),
            mock.patch("sys.stdin", io.StringIO(line)),
            mock.patch("sys.argv", ["x"]),
            mock.patch("subprocess.run") as run,
            mock.patch("sys.stderr", new_callable=io.StringIO) as err,
        ):
            self.assertEqual(pre_push_clippy.main(), 1)
        run.assert_not_called()
        self.assertIn("pre_push_clippy.py check", err.getvalue())


if __name__ == "__main__":
    unittest.main()
