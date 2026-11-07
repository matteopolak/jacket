import contextlib
import io
import unittest
from unittest import mock
import python_decoder


class DecoderProtocol(unittest.TestCase):
    def invoke(self, data):
        source = io.TextIOWrapper(io.BytesIO(data), encoding='utf-8')
        with mock.patch('sys.stdin', source), contextlib.redirect_stdout(io.StringIO()):
            return python_decoder.cli_main()

    def test_valid_and_expected_rejection_status(self):
        self.assertEqual(self.invoke(b'x=1\n'), 0)
        self.assertEqual(self.invoke(b'x=[\n'), 1)
        self.assertEqual(self.invoke(b'x=\xff\n'), 1)

    def test_unexpected_failure_cannot_pass_as_invalid(self):
        with mock.patch.object(python_decoder, 'main', side_effect=RuntimeError('probe')), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(python_decoder.cli_main(), 70)


if __name__ == '__main__':
    unittest.main()
