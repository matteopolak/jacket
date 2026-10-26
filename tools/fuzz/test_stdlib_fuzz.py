import unittest

import stdlib_fuzz as sf
class SignatureTests(unittest.TestCase):
    def test_runtime_error_keeps_file_line_and_message(self):
        err = 'x/stdlib/internal/string/bytes.jai:6:5: error: runtime error: assertion failed: `a` is false\n  code\n'
        self.assertEqual(sf.signature_of(1, err), 'bytes.jai:6:5: runtime error: assertion failed: `a` is false')

    def test_large_numbers_are_normalised(self):
        err = 'f.jai:2:1: error: runtime error: cast of 8195237237126968761 to `u8` overflows\n'
        self.assertIn('cast of N to', sf.signature_of(1, err))

    def test_timeout_and_signals(self):
        self.assertEqual(sf.signature_of(None, ''), 'timeout')
        self.assertEqual(sf.signature_of(-11, ''), 'exit -11')

    def test_last_case(self):
        out = 'CASE A.f 1 256\nCASE A.f 2 256\n'
        self.assertEqual(sf.last_case(out), ('A.f', 2, 256))
        self.assertIsNone(sf.last_case(''))

    def test_known_matches_by_short_name(self):
        settings = {'known': {'slice': ['index >= 0']}}
        self.assertTrue(sf.is_known(settings, 'String.slice', 'x: assertion failed: `index >= 0`'))
        self.assertFalse(sf.is_known(settings, 'String.split', 'x: assertion failed: `index >= 0`'))


if __name__ == '__main__':
    unittest.main()
