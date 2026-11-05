"""Docs benchmark accounting uses deterministic samples, never timed subprocesses."""
import unittest
import doc_site


class ProcessTreeTests(unittest.TestCase):
    def test_tree_rss_sums_descendants_and_own_group_orphans_not_unrelated_processes(self):
        rows = [(10, 1, 10, 100), (11, 10, 10, 200), (12, 11, 12, 300),
                (13, 1, 10, 400), (44, 1, 1, 800)]
        self.assertEqual(doc_site.tree_rss(rows, 10), (1000, 4))

    def test_summary_retains_failures_and_reports_median_and_range(self):
        rows = [{"exit": 0, "wallSeconds": value, "sampledTreePeakKiB": memory}
                for value, memory in [(3.0, 600), (1.0, 200), (2.0, 400)]]
        summary = doc_site.summarize(rows)
        self.assertEqual(summary["wallSeconds"], {"median": 2.0, "min": 1.0, "max": 3.0})
        rows[1]["exit"] = 1
        self.assertFalse(doc_site.summarize(rows)["allPassed"])
        self.assertEqual(doc_site.summarize(rows)["failures"], [1])


if __name__ == "__main__":
    unittest.main()
