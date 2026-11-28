import tempfile
import unittest
from pathlib import Path

from check_guide_files import problems

GUIDE = "```jai\n// p/a.jai\nx :: 1;\n\ny :: 2;\n```\n"


class GuideFiles(unittest.TestCase):
    def run_case(self, guide, files):
        with tempfile.TemporaryDirectory() as tmp:
            for name, text in files.items():
                path = Path(tmp) / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            return problems("g", guide, Path(tmp))

    def test_the_whole_file_matches(self):
        self.assertEqual(self.run_case(GUIDE, {"sub/p/a.jai": "x :: 1;\ny :: 2;\n"}), [])

    def test_a_run_of_lines_matches(self):
        self.assertEqual(self.run_case(GUIDE, {"p/a.jai": "x :: 1;\ny :: 2;\nz :: 3;\n"}), [])

    def test_a_changed_line_is_reported(self):
        self.assertEqual(len(self.run_case(GUIDE, {"p/a.jai": "x :: 1;\ny :: 9;\n"})), 1)

    def test_lines_that_are_not_consecutive_are_reported(self):
        self.assertEqual(len(self.run_case(GUIDE, {"p/a.jai": "x :: 1;\nz :: 3;\ny :: 2;\n"})), 1)

    def test_a_missing_file_is_reported(self):
        self.assertEqual(len(self.run_case(GUIDE, {})), 1)

    def test_a_block_without_a_header_is_ignored(self):
        self.assertEqual(self.run_case("```jai\nx :: 1;\n```\n", {}), [])


if __name__ == "__main__":
    unittest.main()
