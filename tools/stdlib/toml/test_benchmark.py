"""Fixed workload inventories and malformed child metrics fail closed."""
import importlib.util
import io
import json
import tarfile
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest

from toolkit.repo import ROOT

source = ROOT / 'benchmarks/compare/toml/measure.py'
spec = importlib.util.spec_from_file_location('toml_benchmark', source)
benchmark = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = benchmark
spec.loader.exec_module(benchmark)


class BenchmarkTests(unittest.TestCase):
    def test_missing_frozen_workload_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'keep.toml'
            path.write_text('x=1')
            plan = {'input_sha256': {path.name: benchmark.digest(path), 'removed.toml': 'old'}}
            with self.assertRaisesRegex(RuntimeError, 'workload manifest'):
                benchmark.validate_manifest([path], plan)

    def test_duplicate_workload_names_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'invalid').mkdir()
            paths = [root / 'case.toml', root / 'invalid/case.toml']
            for path in paths:
                path.write_text('x=1')
            with self.assertRaisesRegex(RuntimeError, 'duplicate workload'):
                benchmark.validate_manifest(paths, None)

    def test_malformed_success_metrics_become_retained_failure(self):
        job = benchmark.Job('own-read', [], None)
        for output in ('not-json', '{}', '[]', '{"iterations":1,"nanoseconds":"oops"}'):
            with self.subTest(output=output):
                run = SimpleNamespace(ok=True, stdout=output, wall=0.01, cpu=0.01, rss_mib=1, returncode=0)
                row = {}
                benchmark.apply_metrics(row, run, job, 1)
                self.assertIn('failure', row)

    def test_valid_metrics_remain_a_success(self):
        row = {}
        run = SimpleNamespace(ok=True, stdout='{"iterations":2,"nanoseconds":100,"output_bytes":6,"checksum":14}', wall=0.01, cpu=0.01, rss_mib=1, returncode=0)
        benchmark.apply_metrics(row, run, benchmark.Job('own-write', [], None, signature=(3, 7)), 2)
        self.assertNotIn('failure', row)
        self.assertEqual(row['ns_per_iteration'], 50)

    def test_formatter_schema_has_no_serialized_output(self):
        row = {}
        run = SimpleNamespace(ok=True, stdout='{"iterations":1,"nanoseconds":100,"checksum":8}', wall=0.01, cpu=0.01, rss_mib=1, returncode=0)
        benchmark.apply_metrics(row, run, benchmark.Job('formatter-candidate', [], None, checksum=8), 1)
        self.assertNotIn('failure', row)
        self.assertEqual(row['ns_per_iteration'], 100)

    def test_zero_cold_duration_is_censored_and_retained(self):
        run = SimpleNamespace(ok=True, stdout='{"iterations":1,"nanoseconds":0,"output_bytes":0,"checksum":2}', wall=0.01, cpu=0.01, rss_mib=1, returncode=0)
        row = {'phase': 'cold'}
        benchmark.apply_metrics(row, run, benchmark.Job('own-lookup', [], None, checksum=2), 1)
        self.assertNotIn('failure', row)
        self.assertTrue(row['below_clock_resolution'])
        self.assertEqual(row['batch']['nanoseconds'], 0)
        warm = {'phase': 'warm'}
        benchmark.apply_metrics(warm, run, benchmark.Job('own-lookup', [], None, checksum=2), 1)
        self.assertIn('failure', warm)
        self.assertEqual(warm['batch']['nanoseconds'], 0)

    def test_previous_build_requires_exact_binary_and_owned_source_archive(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'jai-batch'
            consumer = root / 'formatter-candidate'
            binary.write_bytes(b'baseline client')
            consumer.write_bytes(b'baseline consumer')
            archive = root / 'frozen-owned-sources.tar.gz'
            source = b'owned source'
            with tarfile.open(archive, 'w:gz') as tar:
                entry = tarfile.TarInfo('owned.jai')
                entry.size = len(source)
                tar.addfile(entry, io.BytesIO(source))
            build = root / 'build.json'
            build.write_text(json.dumps({'binary_sha256': benchmark.digest(binary), 'sources': {'owned.jai': benchmark.hashlib.sha256(source).hexdigest()}, 'consumers': {'candidate': {'binary_sha256': benchmark.digest(consumer)}}}))
            checked = benchmark.previous_receipt(build)
            self.assertEqual(checked['binary'], str(binary.resolve()))
            binary.write_bytes(b'changed client')
            with self.assertRaisesRegex(RuntimeError, 'previous client'):
                benchmark.previous_receipt(build)
            binary.write_bytes(b'baseline client')
            with tarfile.open(archive, 'w:gz') as tar:
                entry = tarfile.TarInfo('owned.jai')
                entry.size = 7
                tar.addfile(entry, io.BytesIO(b'changed'))
            with self.assertRaisesRegex(RuntimeError, 'previous source archive'):
                benchmark.previous_receipt(build)



if __name__ == '__main__':
    unittest.main()
