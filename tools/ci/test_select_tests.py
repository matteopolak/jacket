import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import select_tests


def listing(*records):
    return "\n".join(json.dumps({"type": "list", **r}) for r in records)


class Names(unittest.TestCase):
    def test_subtests_and_benchmarks_are_left_out(self):
        text = listing(
            {"kind": "test", "name": "Sort::a"},
            {"kind": "subtest", "name": "Sort::a/one"},
            {"kind": "bench", "name": "Sort::b"},
            {"kind": "doc", "name": "doc: Sort#1"},
            {"kind": "golden", "name": "Sort::g"},
        )
        self.assertEqual(
            select_tests.test_names(text), ["Sort::a", "doc: Sort#1", "Sort::g"]
        )

    def test_a_line_that_is_not_json_means_run_everything(self):
        self.assertIsNone(select_tests.test_names("jacket: oops\n"))


class Affected(unittest.TestCase):
    def fake(self, code, out="", err=""):
        seen = []

        def run(command, **_):
            seen.append(command)
            return subprocess.CompletedProcess(command, code, out, err)

        return run, seen

    def test_the_changed_files_become_flags(self):
        run, seen = self.fake(0, listing({"kind": "test", "name": "A::t"}))
        names = select_tests.affected("jacket", ["a.jai", "b.jai"], run)
        self.assertEqual(names, ["A::t"])
        self.assertEqual(
            seen[0][5:], ["--changed", "a.jai", "--changed", "b.jai"]
        )

    def test_a_failing_jacket_runs_everything(self):
        run, _ = self.fake(2, err="boom")
        self.assertIsNone(select_tests.affected("jacket", ["a.jai"], run))

    def test_no_changed_files_run_everything(self):
        run, seen = self.fake(0)
        self.assertIsNone(select_tests.affected("jacket", [], run))
        self.assertEqual(seen, [])


class Main(unittest.TestCase):
    def test_an_empty_selection_is_an_empty_file(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / "names.txt"
            original = select_tests.affected
            select_tests.affected = lambda *_: []
            try:
                code = select_tests.main(
                    ["--jacket", "j", "--out", str(out)],
                    {"JACKET_TEST_CHANGED": '["x.jai"]'},
                )
            finally:
                select_tests.affected = original
            self.assertEqual(code, 0)
            self.assertEqual(out.read_text(), "")

    def test_an_unset_variable_runs_everything(self):
        code = select_tests.main(["--jacket", "j", "--out", "unused"], {})
        self.assertEqual(code, select_tests.EVERYTHING)


if __name__ == "__main__":
    unittest.main()
