"""The production-bindings stb libraries are built from the shared native manifest."""
import json
import tempfile
import unittest
from pathlib import Path

import build_dependencies
from toolkit.repo import ROOT


class StbUnitTests(unittest.TestCase):
    def test_units_are_the_manifest_code(self):
        libraries = json.loads((ROOT / 'tools/native/native-libs.json').read_text())['libraries']
        with tempfile.TemporaryDirectory() as directory:
            units = build_dependencies.write_stb_units(Path(directory))
            self.assertEqual(set(units), {'stb_image', 'stb_image_resize', 'stb_image_write'})
            for name, [path] in units.items():
                self.assertEqual(path.read_text(), libraries[name]['code'])
            self.assertIn('jai_stbiw_free', units['stb_image_write'][0].read_text())


if __name__ == '__main__':
    unittest.main()
