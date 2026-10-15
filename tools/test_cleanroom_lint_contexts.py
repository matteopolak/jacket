"""Coverage protocol failures must not become production lint passes."""
from pathlib import Path
import tempfile
import unittest
from cleanroom_lint_contexts import checked_coverage, resolve_contexts


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


if __name__ == "__main__":
    unittest.main()
