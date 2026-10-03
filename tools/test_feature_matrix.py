import json
import hashlib
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from check_corpus import Evidence, Stage, Status
from check_feature_matrix import load_cases, run_native, check
from inventory_corpus_features import mask_noncode, decode


class FeatureMatrixTests(unittest.TestCase):
    def test_reviewed_manifest_has_hashes_and_distinct_suites(self):
        manifest, cases = load_cases()
        self.assertIn('self-written', manifest['provenance'])
        self.assertEqual({case['suite'] for _, case in cases}, {'core', 'expanded', 'negative'})
        self.assertGreaterEqual(len(cases), 25)

    def test_native_requires_exact_output_and_exit(self):
        with TemporaryDirectory() as directory:
            output = Path(directory)/'program'
            output.write_bytes(b'generated executable placeholder')
            output.chmod(0o700)
            fingerprint = hashlib.sha256(output.read_bytes()).hexdigest()
            expected = {'exit_code':17, 'stdout':'', 'stderr':''}
            for code, stdout, stderr, status in [(17,b'',b'',Status.PASSED),(18,b'',b'',Status.FAILED),
                                               (-11,b'',b'',Status.FAILED),(17,b'extra',b'',Status.FAILED)]:
                with patch('check_feature_matrix.subprocess.run', return_value=subprocess.CompletedProcess([],code,stdout,stderr)):
                    self.assertEqual(run_native(output,expected,1,expected_sha256=fingerprint).status,status)
            with patch('check_feature_matrix.subprocess.run', side_effect=subprocess.TimeoutExpired([],1)):
                self.assertEqual(run_native(output,expected,1,expected_sha256=fingerprint).status,Status.FAILED)

    def test_runtime_requires_build_hash_and_real_executable(self):
        with TemporaryDirectory() as directory:
            output = Path(directory)/'program'
            output.write_bytes(b'known generated executable')
            output.chmod(0o700)
            fingerprint = hashlib.sha256(output.read_bytes()).hexdigest()
            expected = {'exit_code':0,'stdout':'','stderr':''}
            with patch('check_feature_matrix.subprocess.run') as runtime:
                for claimed in (None, 'stale fingerprint'):
                    self.assertEqual(run_native(output,expected,1,expected_sha256=claimed).status,Status.FAILED)
                output.chmod(0o600)
                self.assertEqual(run_native(output,expected,1,expected_sha256=fingerprint).status,Status.FAILED)
                output.chmod(0o700)
                link = Path(directory)/'symlink'
                link.symlink_to(output)
                self.assertEqual(run_native(link,expected,1,expected_sha256=fingerprint).status,Status.FAILED)
            runtime.assert_not_called()

    def test_runtime_hash_provenance_and_post_execution_change(self):
        with TemporaryDirectory() as directory:
            output = Path(directory)/'program'
            output.write_bytes(b'known generated executable')
            output.chmod(0o700)
            fingerprint = hashlib.sha256(output.read_bytes()).hexdigest()
            expected = {'exit_code':0,'stdout':'','stderr':''}
            with patch('check_feature_matrix.subprocess.run',return_value=subprocess.CompletedProcess([],0,b'',b'')):
                result = run_native(output,expected,1,expected_sha256=fingerprint)
            self.assertEqual(result.status,Status.PASSED)
            self.assertEqual(result.output_sha256,fingerprint)
            def altered(*args, **kwargs):
                output.write_bytes(b'changed during execution')
                return subprocess.CompletedProcess([],0,b'',b'')
            with patch('check_feature_matrix.subprocess.run',side_effect=altered):
                result = run_native(output,expected,1,expected_sha256=fingerprint)
            self.assertEqual(result.status,Status.FAILED)
            self.assertIn('changed during runtime',result.reason)
            self.assertNotEqual(result.output_sha256,fingerprint)

    def test_runtime_is_never_inferred_from_failed_build(self):
        _, cases = load_cases()
        source, case = cases[0]
        from check_corpus import Result
        result = Result(source.id,str(source.path),source.sha256,'program','host',[],
                        {stage.value:Evidence(Status.NOT_RUN) for stage in Stage})
        result.stages['build'] = Evidence(Status.FAILED)
        with patch('check_feature_matrix.evaluate', return_value=result), patch('check_feature_matrix.run_native') as native:
            checked = check(source,case,Path('jai-rs'),Stage.RUN,1,Path('out'))
        native.assert_not_called()
        self.assertEqual(checked.stages['build'].status,Status.FAILED)

    def test_inventory_masks_nested_comments_and_quoted_text_without_line_drift(self):
        text = '// #run\n/* outer /* #insert */ #run */\ntext := "#if \\\" #load"; #assert true;\n'
        masked = mask_noncode(text)
        self.assertEqual(len(masked),len(text))
        self.assertEqual(masked.count('\n'),text.count('\n'))
        for hidden in ('#run','#insert','#if','#load'):
            self.assertNotIn(hidden,masked)
        self.assertIn('#assert',masked)
        self.assertIn('//',decode(b'// legacy \xf1'))

if __name__ == '__main__':
    unittest.main()
