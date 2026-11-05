from pathlib import Path
import struct
import tempfile
import unittest
from run import runner_asset, windows_adapter_cpu


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


if __name__ == '__main__':
    unittest.main()
