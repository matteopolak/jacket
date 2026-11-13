"""Which files the language server oracle leaves out."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lsp import oracle


class LoadedFileTests(unittest.TestCase):
    def tree(self, files):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        for name, text in files.items():
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            (root / name).write_text(text)
        return root

    def test_loaders_are_found_beside_the_file_and_one_folder_up(self):
        root = self.tree({
            "module.jai": '#load "other.jai";\n',
            "other.jai": "",
            "fixture.jai": '#load "support/helper.jai";\n',
            "support/helper.jai": "",
            "unrelated.jai": '#load "elsewhere.jai";\n',
        })
        self.assertEqual(oracle.loaders(root / "other.jai"), [root / "module.jai"])
        self.assertEqual(oracle.loaders(root / "support/helper.jai"), [root / "fixture.jai"])
        self.assertEqual(oracle.loaders(root / "unrelated.jai"), [])

    def test_a_file_only_a_rejected_loader_reaches_is_left_out(self):
        root = self.tree({
            "broken.jai": '#load "part.jai";\n',
            "fine.jai": '#load "other.jai";\n',
            "part.jai": "",
            "other.jai": "",
        })
        rejected = {root / "broken.jai"}
        with patch.object(oracle, "accepted", side_effect=lambda _, path: path not in rejected):
            self.assertTrue(oracle.only_in_rejected_programs(None, root / "part.jai"))
            self.assertFalse(oracle.only_in_rejected_programs(None, root / "other.jai"))


if __name__ == "__main__":
    unittest.main()
