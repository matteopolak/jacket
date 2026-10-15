"""Coverage protocol failures must not become production lint passes."""
from pathlib import Path
import tempfile
import unittest
from cleanroom_lint_contexts import checked_coverage, resolve_contexts, target_coverage_gaps


class LintContexts(unittest.TestCase):
    def document(self):
        return {"version": 1, "kind": "source-check", "incomplete": [],
                "contexts": [{"complete": True, "visited_files": ["source.jai"]}],
                "unvisited_files": ["inactive.jai"]}

    def test_incomplete_context_cannot_pass_even_with_zero_process_status(self):
        data = self.document()
        data["contexts"][0]["complete"] = False
        coverage = checked_coverage(data, 0)
        self.assertFalse(coverage["clean"])
        self.assertEqual(coverage["visited_complete"], [])
        self.assertEqual(coverage["visited_partial"], [str(Path("source.jai").resolve())])

    def test_denied_findings_do_not_establish_clean_file_coverage(self):
        coverage = checked_coverage(self.document(), 1)
        self.assertFalse(coverage["clean"])
        self.assertEqual(coverage["visited_complete"], [])

    def test_unvisited_files_remain_distinct_from_complete_visited_sources(self):
        coverage = checked_coverage(self.document(), 0)
        self.assertTrue(coverage["clean"])
        self.assertEqual(coverage["unvisited"], ["inactive.jai"])
        self.assertNotIn(str(Path("inactive.jai").resolve()), coverage["visited_complete"])

    def test_real_context_keeps_target_and_source_without_rewriting(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "program.jai"
            source.write_text("main :: () {}\n")
            data = {"version": 1, "contexts": [{"id": "original", "source": "program.jai",
                    "paths": ["program.jai"], "targets": ["windows-arm64"]}]}
            jobs = resolve_contexts(data, root)
            self.assertEqual(jobs[0]["source"], source)
            self.assertEqual(jobs[0]["target"], "windows-arm64")
            self.assertEqual(source.read_text(), "main :: () {}\n")
            data["contexts"][0]["targets"] = ["invented"]
            with self.assertRaisesRegex(ValueError, "unknown source-check target"):
                resolve_contexts(data, root)

    def test_host_coverage_does_not_cover_inactive_cross_target_sources(self):
        source = Path("conditional.jai")
        coverage = {"macos-arm64": {str(source.resolve())}}
        gaps = target_coverage_gaps([source], coverage, ["macos-arm64", "windows-x64"])
        self.assertEqual(gaps["macos-arm64"], [])
        self.assertEqual(gaps["windows-x64"], [source])
        with self.assertRaisesRegex(ValueError, "nonempty"):
            target_coverage_gaps([source], coverage, [])

    def test_common_paths_preserve_real_context_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "library").mkdir()
            (root / "program.jai").write_text("main :: () {}\n")
            document = {"version": 1, "common_paths": ["library"], "contexts": [
                {"id": "program", "source": "program.jai", "paths": ["program.jai"],
                 "targets": ["macos-arm64"]}]}
            jobs = resolve_contexts(document, root)
            self.assertEqual(jobs[0]["paths"], [root / "library", root / "program.jai"])


if __name__ == "__main__":
    unittest.main()
