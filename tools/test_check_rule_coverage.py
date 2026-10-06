import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from check_rule_coverage import report


class RuleCoverageTests(unittest.TestCase):
    def fixture(self, root, doc, tests):
        (root / "docs/language").mkdir(parents=True)
        (root / "docs/metaprogramming").mkdir(parents=True)
        (root / "docs/language/enums.md").write_text(doc)
        (root / "tests/stdlib").mkdir(parents=True)
        for name, text in tests.items():
            (root / "tests/stdlib" / name).write_text(text)

    def test_covered_uncovered_and_code_blocks(self):
        doc = "Members count up {#enum.1}. Bases {#enum.2}.\n```jai\nx := 1; // {#enum.9}\n```\n"
        with TemporaryDirectory() as d:
            root = Path(d)
            self.fixture(root, doc, {"a.jai": "// rules: enum.1\nmain :: () {}\n"})
            r = report(root)
            self.assertEqual(r["rules"], 2)
            self.assertEqual(r["covered"], ["enum.1"])
            self.assertEqual(r["uncovered"], ["enum.2"])
            self.assertEqual(r["unknown"], {})

    def test_unknown_and_duplicate_ids(self):
        doc = "A {#enum.1}.\nB {#enum.1}.\n"
        with TemporaryDirectory() as d:
            root = Path(d)
            self.fixture(root, doc, {"a.jai": "// rules: enum.1, enum.7\n"})
            r = report(root)
            self.assertEqual(list(r["unknown"]), ["enum.7"])
            self.assertEqual(list(r["duplicates"]), ["enum.1"])


if __name__ == "__main__":
    unittest.main()
