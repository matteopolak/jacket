"""Semantic edits refresh exports; presentation edits stay with Astro's own watcher."""

from pathlib import Path
import tempfile
import unittest

import doc_watch


class DocWatchTests(unittest.TestCase):
    def test_semantic_edits_deletions_and_renames_refresh_without_reacting_to_css(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "lib"
            source.mkdir()
            api = source / "module.jai"
            api.write_text("owned source")
            css = root / "website.css"
            css.write_text("owned style")
            calls = []
            watcher = doc_watch.SourceWatcher(
                [source], lambda: calls.append("export") or 0
            )
            css.write_text("changed style")
            self.assertIsNone(watcher.refresh())
            api.write_text("changed source")
            self.assertTrue(watcher.refresh())
            api.rename(source / "renamed.jai")
            self.assertTrue(watcher.refresh())
            (source / "renamed.jai").unlink()
            self.assertTrue(watcher.refresh())
            self.assertEqual(calls, ["export", "export", "export"])

    def test_failed_export_is_stale_and_is_not_retried_without_another_source_change(
        self,
    ):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "guide.md"
            source.write_text("original")
            codes = iter([1, 0])
            watcher = doc_watch.SourceWatcher([source], lambda: next(codes))
            source.write_text("broken")
            self.assertFalse(watcher.refresh())
            self.assertIsNone(watcher.refresh())
            source.write_text("corrected")
            self.assertTrue(watcher.refresh())
            self.assertIsNone(watcher.refresh())

    def test_file_content_is_the_change_identity_not_a_timestamp_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "module.jai"
            source.write_text("before")
            before = doc_watch.snapshot([source])
            source.write_text("after!")
            self.assertNotEqual(doc_watch.snapshot([source]), before)
            same = doc_watch.snapshot([source])
            source.touch()
            self.assertEqual(doc_watch.snapshot([source]), same)


if __name__ == "__main__":
    unittest.main()
