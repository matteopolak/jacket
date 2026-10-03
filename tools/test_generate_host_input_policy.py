"""Inert byte fixtures only; no native executable is invoked."""
import hashlib
import tempfile
import unittest
from pathlib import Path
import generate_host_input_policy as policy

class NativeInventoryTests(unittest.TestCase):
    def test_static_native_inspection_preserves_bytes_and_skips_symlink_escape(self):
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            binary = b'\x7fELF' + bytes(range(256))
            program.write_bytes(binary)
            (Path(outside) / 'unrelated.o').write_bytes(b'inert external object')
            before = program.stat().st_mtime_ns
            inventory = policy.refresh_native(root)
            self.assertEqual(inventory['missing_roots'], ['vendor', 'corpus/upstream'])
                'bytes': len(binary), 'sha256': hashlib.sha256(binary).hexdigest()}])
            self.assertEqual(program.read_bytes(), binary)
            self.assertEqual(program.stat().st_mtime_ns, before)

    def test_object_archive_suffix_and_magic_are_static_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, data in [('inert.o', b'inert object'), ('suffixless', b'!<arch>\nfixture'),
                               ('data.bin', b'ordinary data')]:
                path = root / name
                path.write_bytes(data)
            self.assertTrue(policy.native_file(root / 'inert.o'))
            self.assertTrue(policy.native_file(root / 'suffixless'))
            self.assertFalse(policy.native_file(root / 'data.bin'))

if __name__ == '__main__':
    unittest.main()
