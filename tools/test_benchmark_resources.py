import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('benchmark_resources', Path(__file__).with_name('benchmark_resources.py'))
resources = importlib.util.module_from_spec(spec)
spec.loader.exec_module(resources)


@unittest.skipUnless(platform.system() in ('Darwin', 'Linux'), 'known getrusage units')
class BenchmarkResourceTests(unittest.TestCase):
    def test_timeout_kills_the_child_and_preserves_resource_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'resources.json'
            result = subprocess.run([
                sys.executable, str(Path(__file__).with_name('benchmark_resources.py')),
                str(path), '--', sys.executable, '-c', 'import time; time.sleep(10)',
            ], env={**os.environ, 'JAI_BENCH_TIMEOUT_SECONDS': '0.05'}, check=False)
            self.assertEqual(result.returncode, 124)
            self.assertTrue(json.loads(path.read_text())['timed_out'])

    def test_units_are_explicit_and_unknown_platforms_fail(self):
        self.assertEqual(resources.peak_rss_bytes(7, 'Darwin'), 7)
        self.assertEqual(resources.peak_rss_bytes(7, 'Linux'), 7168)
        with self.assertRaises(ValueError):
            resources.peak_rss_bytes(7, 'unknown')

    def test_real_child_peak_memory_and_failure_exit_are_retained(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'resources.json'
            child = 'import sys; owned = bytearray(16 * 1024 * 1024); owned[-1] = 1; sys.exit(7)'
            result = subprocess.run([
                sys.executable, str(Path(__file__).with_name('benchmark_resources.py')),
                str(path), '--', sys.executable, '-c', child,
            ], check=False)
            self.assertEqual(result.returncode, 7)
            record = json.loads(path.read_text())
            self.assertEqual(record['exit_code'], 7)
            self.assertGreater(record['maximum_child_peak_rss_bytes'], 0)
            self.assertGreaterEqual(record['user_cpu_seconds'], 0)
            self.assertIn('entire suite including preparation', record['scope'])
            self.assertFalse(path.with_suffix('.tmp').exists())


if __name__ == '__main__':
    unittest.main()
