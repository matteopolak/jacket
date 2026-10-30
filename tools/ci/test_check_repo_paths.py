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
        self.assertEqual(
            self.check("see tools/gone.py#anchor"), ["x: missing path tools/gone.py"]
        )

    def test_globs_placeholders_and_generated_paths_are_skipped(self):
        self.assertEqual(
            self.check(
                "tools/*.py tools/<name>.py target/x docs/{a,b}.md corpus/upstream/y"
            ),
            [],
        )

    def test_doc_links_resolve_against_the_doc(self):
        docs = self.root / "docs"
        self.assertEqual(
            self.check("[a](a.md#top) [w](https://x/y.md) [p](print)", doc_dir=docs), []
        )
        self.assertEqual(self.check("[b](b.md)", doc_dir=docs), ["x: dead link b.md"])

    def test_docs_ignore_module_relative_test_paths(self):
        self.assertEqual(
            self.check("run tests/codec.jai", doc_dir=self.root / "docs"), []
        )

    def test_header_stops_after_the_docstring(self):
        text = '#!/usr/bin/env python3\n"""Uses tools/a.py.\n\nMore."""\nimport x  # tools/b.py\n'
        self.assertIn("tools/a.py", paths.header(text))
        self.assertNotIn("tools/b.py", paths.header(text))


class Symbols(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        src = self.root / "crates/my-crate/src"
        (src / "inner").mkdir(parents=True)
        (self.root / "crates/my-crate/Cargo.toml").write_text("")
        (src / "lib.rs").write_text("pub mod inner;\npub use inner::Reexported;\npub const LIMIT: u32 = 1;\n")
        (src / "inner.rs").write_text("mod deep;\npub struct Thing;\nimpl Thing { pub fn method(&self) {} }\n")
        (src / "inner/deep.rs").write_text("pub fn found() {}\n")
        self.crates = paths.crate_roots(self.root)

    def check(self, text):
        return paths.check_symbols("x.md", text, self.crates)

    def test_modules_items_and_members_resolve(self):
        self.assertEqual(
            self.check("`my_crate::inner::Thing::method` `my_crate::LIMIT` `my_crate::inner`"), []
        )
        self.assertEqual(self.check("`my_crate::Reexported` `my_crate::inner::{Thing, deep}`"), [])

    def test_a_missing_name_is_reported_with_the_part_that_is_missing(self):
        self.assertEqual(
            self.check("`my_crate::inner::Gone`"),
            ["x.md: `my_crate::inner::Gone` does not resolve: no `Gone`"],
        )
        self.assertEqual(
            self.check("`my_crate::inner::Thing::renamed`"),
            ["x.md: `my_crate::inner::Thing::renamed` does not resolve: no `renamed`"],
        )

    def test_other_crates_and_code_blocks_are_ignored(self):
        self.assertEqual(self.check("`std::fmt::Gone` `serde::Gone`\n```\n`my_crate::Gone`\n```"), [])


class Index(unittest.TestCase):
    def test_a_page_no_index_links_is_reported(self):
        with tempfile.TemporaryDirectory() as scratch:
            docs = Path(scratch) / "docs"
            (docs / "sub").mkdir(parents=True)
            (docs / "README.md").write_text("[sub](sub/) [a](a.md)")
            (docs / "a.md").write_text("")
            (docs / "sub/README.md").write_text("[b](b.md)")
            (docs / "sub/b.md").write_text("")
            (docs / "sub/lost.md").write_text("")
            self.assertEqual(
                paths.check_index(Path(scratch)),
                ["docs/sub/lost.md: not listed in docs/README.md (or an index page it links)"],
            )


class Repository(unittest.TestCase):
    def test_docs_and_tool_headers_name_existing_paths(self):
        self.assertEqual(paths.run(), [])


if __name__ == "__main__":
    unittest.main()
