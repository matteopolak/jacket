"""Regression checks for loaded-source provenance and diagnostic amplification."""
import json
from pathlib import Path
import tempfile
import unittest

from check_corpus import digest
from classify_corpus_failures import classify


class FailureReportTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.directory = Path(self.scratch.name)
        self.dependency = self.directory / 'module.jai'
        self.dependency.write_text('value :: 1;\n')
        self.roots = [self.directory / f'root{i}.jai' for i in range(2)]
        for root in self.roots:
            root.write_text('main :: () {}\n')
        self.report = self.directory / 'report.json'

    def write_report(self, diagnostic=None):
        rows = []
        for i, root in enumerate(self.roots):
            rows.append({'id': f'project:root{i}.jai', 'source': str(root), 'sha256': digest(root),
                         'stages': {'check': {'status': 'failed', 'reason': 'compiler diagnostic',
                                              'diagnostic': diagnostic or f'{self.dependency}:1:2: error: unsupported syntax'}}})
        rows.append({'id': 'reference:module.jai', 'source': str(self.dependency),
                     'sha256': digest(self.dependency), 'stages': {'check': {'status': 'not-run'}}})
        self.report.write_text(json.dumps({'results': rows, 'compiler': {'sha256': 'binary'},
                                          'inventory': 3, 'totals': {'check': {'failed': 2}, 'parse': {'passed': 3}}}))

    def test_shared_dependency_counts_roots_and_keeps_source_identity(self):
        self.write_report()
        result = classify(self.report, 'check')
        group = result['groups'][0]
        self.assertEqual(group['count'], 2)
        dependency = group['dependency_locations'][0]
        self.assertEqual(dependency['source'], 'reference:module.jai')
        self.assertEqual(dependency['root_count'], 2)
        self.assertEqual(dependency['sha256'], digest(self.dependency))
        self.assertEqual(set(dependency['sample_roots']), {'project:root0.jai', 'project:root1.jai'})
        self.assertEqual(result['acceptance_report_sha256'], digest(self.report))

    def test_modified_dependency_invalidates_both_root_diagnostics(self):
        self.write_report()
        self.dependency.write_text('changed :: 2;\n')
        result = classify(self.report, 'check')
        self.assertEqual(result['groups'], [])
        self.assertEqual(result['stale_source_locations'], ['project:root0.jai', 'project:root1.jai'])

    def test_modified_root_and_unknown_dependency_are_not_verified(self):
        self.write_report(f'{self.directory / "unlisted.jai"}:1:2: error: bad')
        self.roots[0].write_text('changed :: 3;\n')
        result = classify(self.report, 'check')
        self.assertEqual(result['groups'], [])
        self.assertEqual(len(result['stale_source_locations']), 2)


if __name__ == '__main__':
    unittest.main()
