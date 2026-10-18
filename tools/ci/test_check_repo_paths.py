import tempfile
import unittest
from pathlib import Path

import check_repo_paths as paths


class Mentions(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        (self.root / "tools").mkdir()
        (self.root / "tools/real.py").write_text("")
        (self.root / "docs").mkdir()
        (self.root / "docs/a.md").write_text("")

    def check(self, text, **kw):
        return paths.check_text("x", text, root=self.root, **kw)

    def test_existing_path_passes_and_missing_path_fails(self):
        self.assertEqual(self.check("see `tools/real.py`."), [])
        self.assertEqual(self.check("see tools/gone.py#anchor"), ["x: missing path tools/gone.py"])

    def test_globs_placeholders_and_generated_paths_are_skipped(self):
        self.assertEqual(self.check("tools/*.py tools/<name>.py target/x docs/{a,b}.md corpus/upstream/y"), [])

    def test_doc_links_resolve_against_the_doc(self):
        docs = self.root / "docs"
        self.assertEqual(self.check("[a](a.md#top) [w](https://x/y.md) [p](print)", doc_dir=docs), [])
        self.assertEqual(self.check("[b](b.md)", doc_dir=docs), ["x: dead link b.md"])

    def test_docs_ignore_module_relative_test_paths(self):
        self.assertEqual(self.check("run tests/codec.jai", doc_dir=self.root / "docs"), [])

    def test_header_stops_after_the_docstring(self):
        text = '#!/usr/bin/env python3\n"""Uses tools/a.py.\n\nMore."""\nimport x  # tools/b.py\n'
        self.assertIn("tools/a.py", paths.header(text))
        self.assertNotIn("tools/b.py", paths.header(text))


class Repository(unittest.TestCase):
    def test_docs_and_tool_headers_name_existing_paths(self):
        self.assertEqual(paths.run(), [])


if __name__ == "__main__":
    unittest.main()
