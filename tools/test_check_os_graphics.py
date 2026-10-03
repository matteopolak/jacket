import hashlib
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import check_os_graphics as checker


class SourceBoundaryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.compiler = self.root/'artifacts/integration-checkpoints/authored-test/jai-rs'
        self.compiler.parent.mkdir(parents=True)
        self.compiler.write_bytes(b'inert authored test bytes; never executable')
        self.sha256 = hashlib.sha256(self.compiler.read_bytes()).hexdigest()
        self.root_patch = patch.object(checker, 'ROOT', self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)

    def test_frozen_compiler_identity_must_match_before_execution(self):
        self.assertEqual(checker.validate_compiler(self.compiler, self.sha256), self.compiler.resolve())
        with patch.object(checker.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'SHA-256 differs'):
                checker.check(self.compiler, '0'*64, 1)
            run.assert_not_called()

    def test_original_compiler_paths_are_rejected_even_with_matching_bytes(self):
        original.parent.mkdir(parents=True)
        original.write_bytes(self.compiler.read_bytes())
        with self.assertRaisesRegex(ValueError, 'our built CLI'):
            checker.validate_compiler(original, self.sha256)

    def test_dependency_source_drift_is_rejected_before_any_root_executes(self):
        drift = SimpleNamespace(id='reference:modules/SomeDependency/module.jai', sha256='changed', expected_sha256='pinned')
        with patch.object(checker, 'inventory', return_value=[drift]), patch.object(checker.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'pinned source missing or changed'):
                checker.check(self.compiler, self.sha256, 1)
            run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
