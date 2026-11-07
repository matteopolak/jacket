import contextlib
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import community_pipeline as pipeline


class CommunityProtocol(unittest.TestCase):
    def setUp(self):
        self.stack = contextlib.ExitStack()
        self.addCleanup(self.stack.close)
        folder = self.stack.enter_context(tempfile.TemporaryDirectory())
        self.stack.enter_context(mock.patch.object(sys, 'platform', 'linux'))
        self.stack.enter_context(mock.patch.object(sys, 'argv', ['pipeline', '--community', str(Path(folder) / 'community'), '--own-adapter', str(Path(folder) / 'own'), '--scratch', folder]))
        self.stack.enter_context(mock.patch.object(sys, 'stdin', io.TextIOWrapper(io.BytesIO(b'x=1\n'))))
        self.stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
        self.stack.enter_context(contextlib.redirect_stderr(io.StringIO()))
        sandbox = self.stack.enter_context(mock.patch.object(pipeline, 'Sandbox'))
        sandbox.return_value.wrap.return_value = ['community']
        self.stack.enter_context(mock.patch.object(pipeline, 'environment', return_value={}))
        self.run = self.stack.enter_context(mock.patch.object(pipeline.subprocess, 'run'))

    def result(self, code, output=b'', error=b''):
        self.run.return_value = subprocess.CompletedProcess([], code, stdout=output, stderr=error)

    def test_output_conversion_failure_is_not_read_rejection(self):
        self.result(0, b'x=[\n')
        self.assertEqual(pipeline.main(), 70)

    def test_unmarked_exit_one_is_not_read_rejection(self):
        self.result(1)
        self.assertEqual(pipeline.main(), 70)

    def test_explicit_read_rejection(self):
        self.result(1, error=b'TOML_READ_REJECTED\n')
        self.assertEqual(pipeline.main(), 1)

    def test_timeout_is_unexpected_failure(self):
        self.run.side_effect = subprocess.TimeoutExpired('community', 10)
        self.assertEqual(pipeline.cli_main(), 70)


if __name__ == '__main__':
    unittest.main()
