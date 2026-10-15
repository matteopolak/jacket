"""Owned checks include edited fixtures and reject source/tool drift."""
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest

from check_cleanroom_quality import input_snapshot, owned_sources


class QualityInputTests(unittest.TestCase):
    def test_modified_and_untracked_fixtures_are_owned(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", root], check=True)
            (root / "tests").mkdir()
            existing = root / "tests/old fixture.jai"
            existing.write_text("old :: 1;\n")
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                            "commit", "-qm", "independent test baseline"], cwd=root, check=True)
            baseline = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
            existing.write_text("old :: 2;\n")
            added = root / "tests/new.jai"
            added.write_text("new :: 3;\n")
            legacy = root / "stdlib/Extensions/Retained/module.jai"
            legacy.parent.mkdir(parents=True)
            legacy.write_text("retained :: 4;\n")
            self.assertEqual(owned_sources(root, baseline), sorted([existing, added]))
            tool = root / "test-tool"
            tool.write_bytes(b"first compiler")
            before = input_snapshot(root, baseline, [tool], [])
            tool.write_bytes(b"second compiler")
            after = input_snapshot(root, baseline, [tool], [])
            self.assertEqual(before["sources"], after["sources"])
            self.assertNotEqual(before, after)
            extra = root / "tests/later.jai"
            extra.write_text("later :: 5;\n")
            self.assertNotEqual(after["sources"], input_snapshot(root, baseline, [tool], [])["sources"])


if __name__ == "__main__":
    unittest.main()
