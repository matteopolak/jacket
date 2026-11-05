"""Docs benchmark accounting uses deterministic samples, never timed subprocesses."""
import unittest
from bench import doc_site


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


class RunOrderTests(unittest.TestCase):
    def test_repetitions_alternate_variant_order_and_keep_grouped_receipts(self):
        from pathlib import Path
        import tempfile
        from unittest.mock import patch
        calls = []

        def fake_measure(command, cwd, env, log, timeout):
            calls.append(command[0])
            return {'exit': 0, 'wallSeconds': 1, 'sampledTreePeakKiB': 1, 'memoryErrors': []}

        with tempfile.TemporaryDirectory() as temporary:
            plan = {'schema': 'jacket-doc-benchmark/1', 'repetitions': 3,
                    'identities': {'fixture': 'fixed'}, 'equivalence': {'fixture': 'fixed'},
                    'cases': [{'name': name, 'command': [name], 'cwd': temporary,
                               'cacheState': 'independently prepared'} for name in ['legacy', 'astro']]}
            with patch.object(doc_site, 'measure', side_effect=fake_measure):
                result = doc_site.run(plan, Path(temporary) / 'results')
        self.assertEqual(calls, ['legacy', 'astro', 'astro', 'legacy', 'legacy', 'astro'])
        self.assertEqual([row['name'] for row in result['cases']], ['legacy', 'astro'])
        self.assertEqual([len(row['runs']) for row in result['cases']], [3, 3])


if __name__ == '__main__':
    unittest.main()
