"""Tests for documentation site comparison and difference reporting."""

import contextlib
import io
from pathlib import Path
import re
import tempfile
import unittest

from doc_diff import classify, compare, first_difference


class DocDiffTests(unittest.TestCase):
    def test_nested_modules_and_items(self):
        self.assertEqual(classify("api/Android/Toolchain/index.html", b'<div class="titlebar"><h1>Toolchain</h1><span class="badge">module</span></div>'), "module")
        self.assertEqual(classify("api/Android/Toolchain/run/index.html", b'<div class="titlebar"><h1>run</h1><span class="badge proc">procedure</span></div>'), "item")
        self.assertEqual(classify("index.html", b""), "index")
        self.assertEqual(classify("guides/cli/index.html", b'<div class="prose guide">'), "guide")
        self.assertEqual(classify("assets/site.js", b""), "other")

    def test_context_is_bounded_and_has_line_number(self):
        a, b = first_difference(b"same\n" + b"a" * 300 + b"x", b"same\n" + b"a" * 300 + b"y")
        self.assertEqual((a[0], b[0]), (2, 2))
        self.assertLessEqual(len(a[1]), 201)
        self.assertTrue(a[1].endswith("x"))
        self.assertTrue(b[1].endswith("y"))

    def test_missing_limit_ignore_and_exit_status(self):
        with tempfile.TemporaryDirectory() as folder:
            old, new = Path(folder) / "old", Path(folder) / "new"
            old.mkdir()
            new.mkdir()
            for path in (old, new):
                (path / "index.html").write_bytes(b"identical")
            with contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(compare(old, new), 0)
            self.assertIn("Identical: 1 files", output.getvalue())
            (old / "only-old.txt").write_bytes(b"old")
            (new / "only-new.txt").write_bytes(b"new")
            for n in range(5):
                (old / f"{n}.txt").write_bytes(b"before")
                (new / f"{n}.txt").write_bytes(b"after")
            with contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(compare(old, new, limit=2), 1)
            text = output.getvalue()
            self.assertLess(text.index("only OLD"), text.index("different"))
            self.assertIn("only NEW [other]: only-new.txt", text)
            self.assertEqual(text.count("different [other]"), 2)
            self.assertIn("other=5", text)
            (old / "only-old.txt").unlink()
            (new / "only-new.txt").unlink()
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(compare(old, new, ignores=[re.compile("before|after")]), 0)

    def test_prefix_and_binary_differences(self):
        self.assertEqual(first_difference(b"a", b"ab"), ((1, "a"), (1, "ab")))
        self.assertEqual(first_difference(b"\xff", b"\xfe")[0][0], 1)
