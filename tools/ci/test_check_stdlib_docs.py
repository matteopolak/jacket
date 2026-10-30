import unittest
from pathlib import Path

import check_stdlib_docs as docs

ROOT = Path("/repo")
OUTPUT = (
    "/repo/stdlib/M/module.jai\t1\tmodule\t\t0\t0\n"
    "/repo/stdlib/M/module.jai\t5\tprocedure\tf\t1\t0\n"
    "/repo/stdlib/M/module.jai\t9\tstruct\tS\t0\t0\n"
    "/repo/stdlib/Big/module.jai\t3\tprocedure\tg\t0\t0\n"
    "/repo/stdlib/Big/module.jai\t7\talias\th\t0\t0\n"
    "/repo/stdlib/Big/module.jai\t8\talias\ti\t1\t1\n"
)


def rows():
    parsed = docs.parse(OUTPUT, ROOT)
    return {
        Path("M"): [r for r in parsed if r["file"].startswith("stdlib/M/")],
        Path("Big"): [r for r in parsed if r["file"].startswith("stdlib/Big/")],
    }


class Records(unittest.TestCase):
    def test_records_become_rows_with_repo_relative_files(self):
        first = docs.parse(OUTPUT, ROOT)[1]
        self.assertEqual(
            first,
            {"file": "stdlib/M/module.jai", "line": 5, "kind": "procedure", "symbol": "f",
             "documented": True, "example": False},
        )

    def test_undocumented_names_the_module_and_its_symbols(self):
        self.assertEqual(
            docs.undocumented(rows()),
            {"M", "M.S", "Big.g", "Big.h"},
        )


class Report(unittest.TestCase):
    def test_the_worst_module_comes_first(self):
        lines = docs.report(rows(), examples=False, top=None, only=None).splitlines()
        self.assertTrue(lines[0].lstrip().startswith("2  Big"), lines)
        self.assertTrue(lines[1].lstrip().startswith("2  M"), lines)
        self.assertEqual(lines[-1], "4 gaps in 2 modules")

    def test_examples_count_documented_items_without_one(self):
        text = docs.report(rows(), examples=True, top=1, only=None)
        self.assertIn("M", text.splitlines()[0])
        self.assertIn("1 without an example", text.splitlines()[0])

    def test_one_module_lists_each_gap_by_line(self):
        text = docs.report(rows(), examples=False, top=None, only="Big")
        self.assertIn("stdlib/Big/module.jai:3  procedure g  no docs", text)


if __name__ == "__main__":
    unittest.main()
