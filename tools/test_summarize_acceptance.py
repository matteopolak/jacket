"""Read-only reporting preserves stage, compiler, and negative-fixture evidence."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import summarize_acceptance as reports


class StageReportTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve()
        compiler_bytes = b'authored inert compiler snapshot fixture; never executed'
        self.compiler_hash = hashlib.sha256(compiler_bytes).hexdigest()
        self.compiler = self.root / 'target/corpus-snapshots' / self.compiler_hash / 'jai-rs'
        self.compiler.parent.mkdir(parents=True)
        self.compiler.write_bytes(compiler_bytes)
        self.source_hash = hashlib.sha256(b'authored source fixture').hexdigest()
        self.artifact_hash = hashlib.sha256(b'authored output fixture').hexdigest()
        self.record = {
            'format': 1,
            'compiler': {'binary': str(self.compiler), 'binary_sha256': self.compiler_hash,
                         'evidence_kind': 'integrated-cli', 'inputs_sha256': 'c' * 64},
            'bootstrap': 'off',
            'results': [{'id': 'authored:example.jai', 'kind': 'program',
                         'sha256': self.source_hash, 'stages': {}}],
        }
        self.path = self.root / 'measured.json'

    def evidence(self, status='passed', exit_code=0, artifact=False):
        return {'status': status, 'exit_code': exit_code,
                'output_sha256': self.artifact_hash if artifact else None}

    def summarize(self, record=None):
        self.path.write_text(json.dumps(self.record if record is None else record))
        return reports.summarize(self.path, self.root)

    def test_parse_success_does_not_promote_later_stages(self):
        self.record['results'][0]['stages']['parse'] = self.evidence()
        result = self.summarize()
        row = result['results'][0]
        self.assertEqual(row['stages']['parse']['status'], 'passed')
        for stage in ('check', 'codegen', 'build', 'run'):
            self.assertEqual(row['stages'][stage]['status'], 'not-run')
        self.assertEqual(result['cohorts'][0]['totals']['build'], {'not-run': 1})

    def test_intended_rejection_is_separate_from_success_and_unattempted_intent(self):
        negative = self.record['results'][0]
        negative.update(kind='negative', stages={'check': self.evidence('expected-rejection', 1)})
        pending = deepcopy(negative)
        pending.update(id='authored:unattempted-negative.jai', stages={})
        self.record['results'].append(pending)
        result = self.summarize()
        self.assertEqual(result['cohorts'][0]['totals']['check'], {'expected-rejection': 1, 'not-run': 1})
        self.assertEqual(result['results'][0]['matched_rejection_stages'], ['check'])
        self.assertEqual(result['results'][1]['negative_intent'], 'declared-negative')
        self.assertEqual(result['results'][1]['matched_rejection_stages'], [])
        table = reports.markdown({'reports': [result]})
        self.assertIn('| check | 0 | 1 | 0 | 0 | 0 | 1 |', table)

    def test_expected_rejection_cannot_hide_a_crash_or_success(self):
        for code in (0, 2, 101, -11, None):
            with self.subTest(exit_code=code):
                self.record['results'][0]['stages']['check'] = self.evidence('expected-rejection', code)
                with self.assertRaisesRegex(ValueError, 'ordinary compiler exit code one'):
                    self.summarize()

    def test_nonzero_runtime_result_requires_the_exact_built_artifact(self):
        stages = self.record['results'][0]['stages']
        stages['build'] = self.evidence(artifact=True)
        stages['run'] = self.evidence(exit_code=42, artifact=True)
        result = self.summarize()
        self.assertEqual(result['results'][0]['stages']['run']['exit_code'], 42)
        stages['run']['output_sha256'] = 'e' * 64
        with self.assertRaisesRegex(ValueError, 'exact successful build artifact'):
            self.summarize()
        stages['run']['output_sha256'] = self.artifact_hash
        del stages['build']
        with self.assertRaisesRegex(ValueError, 'exact successful build artifact'):
            self.summarize()

    def test_success_requires_source_and_output_fingerprints(self):
        row = self.record['results'][0]
        row['stages']['codegen'] = self.evidence()
        with self.assertRaisesRegex(ValueError, 'requires its artifact hash'):
            self.summarize()
        row['stages']['codegen'] = self.evidence(artifact=True)
        row['sha256'] = None
        with self.assertRaisesRegex(ValueError, 'requires its input hash'):
            self.summarize()

    def test_runtime_signal_or_language_rejection_cannot_be_a_success(self):
        stages = self.record['results'][0]['stages']
        stages['build'] = self.evidence(artifact=True)
        stages['run'] = self.evidence(exit_code=-9, artifact=True)
        with self.assertRaisesRegex(ValueError, 'requires a normal exit'):
            self.summarize()
        stages['run'] = self.evidence('expected-rejection', 1)
        with self.assertRaisesRegex(ValueError, 'compiler rejection contract'):
            self.summarize()

    def test_unattempted_stage_cannot_hide_execution_or_an_artifact(self):
        for evidence in (self.evidence('not-run', 101), self.evidence('not-run', None, artifact=True)):
            self.record['results'][0]['stages']['build'] = evidence
            with self.assertRaisesRegex(ValueError, 'unattempted stage cannot carry'):
                self.summarize()

    def test_mutable_missing_and_mismatched_compiler_files_are_rejected(self):
        mutable = self.root / 'target/debug/jai-rs'
        mutable.parent.mkdir(parents=True)
        mutable.write_bytes(self.compiler.read_bytes())
        self.record['compiler']['binary'] = str(mutable)
        with self.assertRaisesRegex(ValueError, 'content-addressed frozen'):
            self.summarize()
        self.record['compiler']['binary'] = str(self.compiler)
        self.compiler.write_bytes(b'changed bytes')
        with self.assertRaisesRegex(ValueError, 'bytes differ'):
            self.summarize()
        self.compiler.unlink()
        with self.assertRaisesRegex(ValueError, 'missing'):
            self.summarize()

    def test_reported_during_run_compiler_drift_and_frontend_adapter_are_rejected(self):
        self.record['compiler']['binary_unchanged'] = False
        with self.assertRaisesRegex(ValueError, 'changed during measurement'):
            self.summarize()
        self.record['compiler'].pop('binary_unchanged')
        self.record['compiler']['binary_sha256_after'] = 'f' * 64
        with self.assertRaisesRegex(ValueError, 'changed during measurement'):
            self.summarize()
        self.record['compiler'].pop('binary_sha256_after')
        self.record['compiler']['evidence_kind'] = 'isolated-frontend-adapter'
        self.record['results'][0]['stages']['check'] = self.evidence()
        with self.assertRaisesRegex(ValueError, 'frontend adapter cannot establish later'):
            self.summarize()

    def test_profiles_are_separate_and_retain_independent_input_observations(self):
        first = self.record['results'][0]
        first.update(profile='diagnostic', stages={'parse': self.evidence()})
        second = deepcopy(first)
        second.update(profile='preload', stages={'check': self.evidence('failed', 1)})
        self.record['results'].append(second)
        result = self.summarize()
        self.assertEqual([cohort['profile'] for cohort in result['cohorts']], ['diagnostic', 'preload'])
        self.assertEqual(result['compiler']['observed_inputs_sha256'], 'c' * 64)
        second['sha256'] = 'f' * 64
        with self.assertRaisesRegex(ValueError, 'same source bytes'):
            self.summarize()
        second['sha256'] = self.source_hash
        second['profile'] = 'diagnostic'
        with self.assertRaisesRegex(ValueError, 'duplicate case identity'):
            self.summarize()

    def test_upstream_negative_intent_does_not_manufacture_a_contract(self):
        row = self.record['results'][0]
        row.update(role='expected-negative-fixture-unreviewed', stages={'check': self.evidence('failed', 1)})
        result = self.summarize()
        self.assertEqual(result['results'][0]['negative_intent'], 'upstream-unreviewed')
        row['stages']['check'] = self.evidence('expected-rejection', 1)
        with self.assertRaisesRegex(ValueError, 'unreviewed negative fixture'):
            self.summarize()

    def test_report_contains_only_whitelisted_source_free_fields(self):
        secret = 'original source text, IR, environment value, and command'
        row = self.record['results'][0]
        row.update(source=secret, dependencies=[secret], purpose=secret, provenance=secret)
        row['stages']['check'] = {**self.evidence('failed', 1), 'diagnostic': secret,
                                 'reason': secret, 'command': [secret], 'environment': {'KEY': secret}}
        self.record.update(inventory=[secret], failure_groups=[secret])
        result = self.summarize()
        self.assertNotIn(secret, json.dumps(result))
        self.assertNotIn(secret, reports.markdown({'reports': [result]}))
        self.assertEqual(result['report_sha256'], reports.fingerprint(self.path))

    def test_input_report_or_compiler_drift_while_rendering_is_not_published(self):
        actual = reports.row_summary
        def mutate_report(*args):
            self.path.write_text('{}')
            return actual(*args)
        with patch.object(reports, 'row_summary', side_effect=mutate_report):
            with self.assertRaisesRegex(ValueError, 'report changed while rendering'):
                self.summarize()
        def mutate_compiler(*args):
            self.compiler.write_bytes(b'changed snapshot')
            return actual(*args)
        with patch.object(reports, 'row_summary', side_effect=mutate_compiler):
            with self.assertRaisesRegex(ValueError, 'compiler changed while rendering'):
                self.summarize()

    def test_malformed_report_rows_stages_and_hashes_are_rejected(self):
        for record in ([], {'format': 1, 'compiler': None, 'results': []},
                       {**self.record, 'results': [None]},
                       {**self.record, 'results': [{**self.record['results'][0], 'stages': {'typo': {}}}]},
                       {**self.record, 'results': [{**self.record['results'][0], 'sha256': 'not-a-hash'}]}):
            with self.subTest(record=record):
                with self.assertRaises(ValueError):
                    self.summarize(record)

    def test_output_cannot_replace_reports_or_protected_input_roots(self):
        with patch.object(reports, 'ROOT', self.root):
            with self.assertRaisesRegex(ValueError, 'distinct'):
                reports.destination(self.path, [self.path])
            for name in ('reference', 'vendor', 'corpus/upstream', '.git'):
                with self.subTest(root=name):
                    with self.assertRaisesRegex(ValueError, 'protected input'):
                        reports.destination(self.root / name / 'summary.json', [self.path])


if __name__ == '__main__':
    unittest.main()
