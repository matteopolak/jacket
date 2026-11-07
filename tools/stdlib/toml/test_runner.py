import json
from pathlib import Path
import subprocess
import sys
import struct
import tempfile
import unittest
from unittest.mock import patch
from run import PIN, main, runner_asset, windows_adapter_cpu


class RunnerArchitecture(unittest.TestCase):
    def test_published_host_aliases(self):
        self.assertEqual(runner_asset(system='Windows', machine='AMD64'), 'windows-amd64.exe')
        self.assertEqual(runner_asset(system='Darwin', machine='arm64'), 'darwin-arm64')

    def test_arm_windows_requires_explicit_published_controller(self):
        with self.assertRaises(ValueError):
            runner_asset(system='Windows', machine='ARM64')
        self.assertEqual(runner_asset('windows-amd64.exe', system='Windows', machine='ARM64'), 'windows-amd64.exe')
        with self.assertRaises(ValueError):
            runner_asset('windows-arm64.exe', system='Windows', machine='ARM64')

    def test_parser_cpu_comes_from_pe_not_controller(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'adapter.exe'
            for machine, cpu in [(0x8664, 'x64'), (0xaa64, 'arm64')]:
                data = bytearray(128)
                data[:2] = b'MZ'
                struct.pack_into('<I', data, 0x3c, 64)
                data[64:68] = b'PE\0\0'
                struct.pack_into('<H', data, 68, machine)
                path.write_bytes(data)
                self.assertEqual(windows_adapter_cpu(path), cpu)
            path.write_bytes(b'MZ')
            with self.assertRaises(ValueError):
                windows_adapter_cpu(path)


class RunnerEncoding(unittest.TestCase):
    def test_controller_json_uses_utf8_under_non_utf8_host(self):
        # U+0090 is valid in JSON but its UTF-8 contains an undefined cp1252 byte.
        payload = dict(version='test-' + PIN['release'], toml='1.1.0', skipped=0,
                       tests=[dict(path='valid/case.toml', skipped=False,
                                   failure=False, diagnostic='\u0090')])
        program = 'import os;os.write(1,' + repr(json.dumps(payload, ensure_ascii=False).encode('utf-8')) + ');os.write(2,b"\\xc2\\x90")'
        launch = subprocess.run
        def controller(command, **options):
            self.assertEqual(command[1], 'test')
            return launch([sys.executable, '-c', program], **options)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            adapter = root / 'adapter'
            adapter.write_bytes(b'fixture')
            report = root / 'report.json'
            args = ['run.py', '--adapter', str(adapter), '--cache', str(root),
                    '--report', str(report), '--expected-windows-cpu', 'arm64',
                    '--runner-asset', 'windows-amd64.exe']
            with patch('run.prepare', return_value=(Path(sys.executable), {'valid/case.toml': 'valid'})), \
                 patch('run.windows_adapter_cpu', return_value='arm64'), \
                 patch('run.subprocess.run', side_effect=controller), \
                 patch('subprocess._text_encoding', return_value='cp1252'), \
                 patch('sys.argv', args):
                self.assertEqual(main(), 0)
            data = json.loads(report.read_text(encoding='utf-8'))
            self.assertEqual(data['tests'][0]['diagnostic'], '\u0090')
            self.assertEqual(data['runner_stderr'], '\u0090')
            self.assertEqual(data['adapter_windows_cpu'], 'arm64')


if __name__ == '__main__':
    unittest.main()
