"""Independent verification-driver isolation regressions; no peer inputs."""
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/native_bindings'))
import verify


class IsolatedStagingTests(unittest.TestCase):
    def test_private_runtime_selection_uses_owning_facade(self):
        self.assertEqual(verify.private_unit_module('clipboard-owned-data'), 'X11')
        self.assertEqual(verify.private_unit_module('shader-bytes'), 'd3d_compiler')
        for name, placement in verify.PRIVATE_UNITS.items():
            self.assertEqual(verify.private_unit_module(name), Path(placement).parts[0])

    def test_existing_stage_never_discovers_peer_checkouts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = root / 'baseline'
            (baseline / 'prelude').mkdir(parents=True)
            (baseline / 'prelude/module.jai').write_text('// independent bootstrap\n')
            (baseline / 'sentinel.jai').write_text('// retained foundation\n')
            destination = root / 'result'
            with patch.object(verify, 'stage', side_effect=AssertionError('peer discovery forbidden')):
                verify.stage_for_checks(destination, baseline)
            self.assertEqual((destination / 'sentinel.jai').read_text(), '// retained foundation\n')
            self.assertEqual((destination / 'X11/wrappers.jai').read_bytes(),
                             (ROOT / 'stdlib/X11/wrappers.jai').read_bytes())

    def test_incomplete_existing_stage_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(SystemExit, 'bootstrap'):
                verify.stage_for_checks(root / 'result', root)
            self.assertFalse((root / 'result').exists())


if __name__ == '__main__':
    unittest.main()
