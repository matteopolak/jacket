"""The published-tree limit includes every version/asset; fixtures contain only a few bytes."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import stat
from types import SimpleNamespace
from unittest.mock import MagicMock, patch
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("check_docs_size.py")
spec = importlib.util.spec_from_file_location("check_docs_size", SCRIPT)
size = importlib.util.module_from_spec(spec)
spec.loader.exec_module(size)


class PublishedTreeBudget(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        (self.root / "versions/old/assets").mkdir(parents=True)
        (self.root / "index.html").write_bytes(b"123")
        (self.root / "versions/old/page.html").write_bytes(b"4567")
        (self.root / "versions/old/assets/runtime.wasm").write_bytes(b"89")
        (self.root / ".jacket-doc").write_bytes(b"")

    def tearDown(self):
        self.temporary.cleanup()

    def test_nested_versions_assets_hidden_and_exact_boundary(self):
        self.assertEqual(size.check_tree(self.root, 9), {"status": "pass", "files": 4, "bytes": 9, "limitBytes": 9})

    def test_one_byte_over_budget_is_failure_without_deletion(self):
        self.assertEqual(size.check_tree(self.root, 8), {"status": "fail", "files": 4, "bytes": 9, "limitBytes": 8})
        self.assertEqual((self.root / "versions/old/assets/runtime.wasm").read_bytes(), b"89")

    def test_same_inode_at_two_paths_counts_both_published_paths(self):
        os.link(self.root / "index.html", self.root / "second.html")
        self.assertEqual(size.check_tree(self.root, 12)["bytes"], 12)

    def test_symlink_file_and_directory_are_rejected_without_target_disclosure(self):
        for source in (self.root / "index.html", self.root / "versions"):
            link = self.root / "assets-link"
            try:
                link.symlink_to(source, target_is_directory=source.is_dir())
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"platform cannot create symlinks: {type(error).__name__}")
            with self.assertRaisesRegex(ValueError, "symlink.*assets-link") as caught:
                size.check_tree(self.root)
            self.assertNotIn(str(self.root), str(caught.exception))
            link.unlink()

    def test_root_symlink_is_rejected(self):
        link = self.root / "root-link"
        try:
            link.symlink_to(self.root, target_is_directory=True)
        except (OSError, NotImplementedError) as error:
            self.skipTest(f"platform cannot create symlinks: {type(error).__name__}")
        with self.assertRaisesRegex(ValueError, "symlink"):
            size.check_tree(link)

    @unittest.skipUnless(hasattr(os, "mkfifo"), "named pipes unavailable on this platform")
    def test_special_file_rejected_without_opening_it(self):
        os.mkfifo(self.root / "named-pipe")
        with self.assertRaisesRegex(ValueError, "special.*named-pipe"):
            size.check_tree(self.root)

    def test_windows_reparse_root_rejected_without_traversal(self):
        metadata = SimpleNamespace(st_mode=stat.S_IFDIR, st_file_attributes=0x400)
        with patch.object(Path, "lstat", return_value=metadata), patch.object(size.os, "scandir") as scan:
            with self.assertRaisesRegex(ValueError, "reparse"):
                size.check_tree(self.root)
            scan.assert_not_called()

    def test_windows_reparse_entry_rejected_without_traversal(self):
        entry = MagicMock()
        entry.path = str(self.root / "junction")
        entry.stat.return_value = SimpleNamespace(st_mode=stat.S_IFDIR, st_file_attributes=0x400)
        first, second = MagicMock(), MagicMock()
        first.__enter__.return_value = [entry]
        second.__enter__.return_value = []
        with patch.object(size.os, "scandir", side_effect=[first, second]) as scan:
            with self.assertRaisesRegex(ValueError, "reparse.*junction"):
                size.check_tree(self.root)
            self.assertEqual(scan.call_count, 1)
            entry.stat.assert_called_once_with(follow_symlinks=False)

    @unittest.skipUnless(os.name == "nt", "actual Windows junction creation requires Windows")
    def test_actual_windows_junction_rejected(self):
        with tempfile.TemporaryDirectory() as target:
            link = self.root / "junction"
            result = subprocess.run(["cmd", "/c", "mklink", "/J", str(link), target], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, "could not create the owned Windows junction fixture")
            try:
                with self.assertRaisesRegex(ValueError, "reparse.*junction"):
                    size.check_tree(self.root)
            finally:
                link.rmdir()

    def test_missing_root_and_negative_limit_rejected(self):
        with self.assertRaises(ValueError):
            size.check_tree(self.root / "absent")
        with self.assertRaises(ValueError):
            size.check_tree(self.root, -1)

    def test_cli_exit_and_compact_public_receipt(self):
        for limit, expected_exit in ((9, 0), (8, 1)):
            result = subprocess.run([sys.executable, str(SCRIPT), str(self.root), "--limit-bytes", str(limit)], text=True, capture_output=True)
            self.assertEqual(result.returncode, expected_exit, result.stderr)
            self.assertEqual(json.loads(result.stdout), size.check_tree(self.root, limit))
            self.assertNotIn(str(self.root), result.stdout + result.stderr)
        link = self.root / "bad-link"
        try:
            link.symlink_to(self.root / "index.html")
        except (OSError, NotImplementedError):
            return
        result = subprocess.run([sys.executable, str(SCRIPT), str(self.root)], text=True, capture_output=True)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout)["status"], "error")
        self.assertNotIn(str(self.root), result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
