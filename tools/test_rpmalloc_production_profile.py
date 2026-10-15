"""Verification must consume an explicit production configuration unchanged."""
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from native_bindings.configure_rpmalloc import configure, select_for_verification


class ProductionProfileTests(unittest.TestCase):
    def test_profile_mismatch_never_rewrites_production_sources(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'rpmalloc').mkdir()
            (root/'rpmalloc/module.jai').write_text('// source marker\n')
            selector = root/'internal/bindings/rpmalloc-build/module.jai'
            selector.parent.mkdir(parents=True)
            selector.write_text('FIRST_CLASS_HEAPS :: false;\n')
            original = selector.read_bytes()
            select_for_verification(root, False, True)
            self.assertEqual(selector.read_bytes(), original)
            with self.assertRaisesRegex(ValueError, 'configure it explicitly'):
                select_for_verification(root, True, True)
            self.assertEqual(selector.read_bytes(), original)
            configure(root, True)
            enabled = selector.read_bytes()
            select_for_verification(root, True, True)
            self.assertEqual(selector.read_bytes(), enabled)


if __name__ == '__main__':
    unittest.main()
