import tempfile
import unittest
from pathlib import Path

from generators import generate_tools_index as index

SCRIPT = '"""Does a thing | with a pipe.\n\nMore."""\n\nif __name__ == "__main__":\n    pass\n'


def write(root, name, text):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return Path(name)


class ToolsIndexTests(unittest.TestCase):
    def setUp(self):
        self._dir = tempfile.TemporaryDirectory()
        self.root = Path(self._dir.name)
        self.addCleanup(self._dir.cleanup)

    def test_entry_points_skip_tests_toolkit_and_modules(self):
        files = [
            write(self.root, "tools/a/run.py", SCRIPT),
            write(self.root, "tools/a/test_run.py", SCRIPT),
            write(self.root, "tools/a/lib.py", "x = 1\n"),
            write(self.root, "tools/toolkit/cli.py", SCRIPT),
            write(self.root, "tools/a/fixtures/f.py", SCRIPT),
            write(self.root, "tools/top.py", SCRIPT),
        ]
        found = [p.as_posix() for p in index.entry_points(self.root, files)]
        self.assertEqual(found, ["tools/a/run.py", "tools/top.py"])

    def test_purpose_is_the_first_docstring_line_with_pipes_escaped(self):
        script = write(self.root, "tools/a/run.py", SCRIPT)
        self.assertEqual(index.purpose(self.root, script), "Does a thing \\| with a pipe.")
        bare = write(self.root, "tools/a/bare.py", 'if __name__ == "__main__":\n    pass\n')
        self.assertEqual(index.purpose(self.root, bare), "(no docstring)")

    def test_callers_are_found_by_path_and_not_by_docs(self):
        script = write(self.root, "tools/a/run.py", SCRIPT)
        files = [
            script,
            write(self.root, ".github/workflows/ci.yml", "run: python3 tools/a/run.py\n"),
            write(self.root, "Justfile", "x:\n    python3 tools/a/run.py\n"),
            write(self.root, "docs/x.md", "tools/a/run.py\n"),
            write(self.root, "tools/b/other.py", "from a import run\nsubprocess.run(['tools/a/run.py'])\n"),
            write(self.root, "tools/a/test_run.py", "tools/a/run.py"),
        ]
        self.assertEqual(index.callers(self.root, files, [script])[script], ["CI", "just", "tools"])

    def test_a_script_nothing_names_has_no_caller(self):
        script = write(self.root, "tools/a/run.py", SCRIPT)
        files = [script, write(self.root, "tools/a/gen_run.py", "# gen_run.py runs alone\n")]
        self.assertEqual(index.callers(self.root, files, [script])[script], [])


if __name__ == "__main__":
    unittest.main()
