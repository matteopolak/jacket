import unittest

import check_native_python as native


class Problems(unittest.TestCase):
    def test_native_python_on_the_expected_machine_passes(self):
        self.assertEqual(native.problems(0, 0xAA64, 0xAA64), [])

    def test_emulated_python_and_the_wrong_machine_fail(self):
        self.assertEqual(len(native.problems(0x8664, 0xAA64, 0xAA64)), 1)
        self.assertEqual(len(native.problems(0, 0x8664, 0xAA64)), 1)


if __name__ == "__main__":
    unittest.main()
