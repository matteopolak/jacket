"""WASI report provenance must describe only the invocation that produced it."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import run_wasi


class WasiReceiptTests(unittest.TestCase):
    def test_existing_report_is_never_relabelled(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = root / 'previous.json'
            old = '{"pass": true, "compiler_sha256": "previous"}\n'
            report.write_text(old)
            (root / 'jaic').write_bytes(b'new compiler')
            (root / 'cache').mkdir()
            (root / 'cache/adapter.wasm').write_bytes(b'new wasm')
            argv = ['run_wasi.py', '--jaic', str(root / 'jaic'), '--cache', str(root / 'cache'), '--report', str(report)]
            with patch.object(sys, 'argv', argv), patch.object(run_wasi.subprocess, 'run') as run, patch.object(run_wasi.subprocess, 'check_output', return_value='node-test'):
                run.return_value.returncode = 70
                with self.assertRaisesRegex(RuntimeError, 'report already exists'):
                    run_wasi.main()
            run.assert_not_called()
            self.assertEqual(report.read_text(), old)


if __name__ == '__main__':
    unittest.main()
