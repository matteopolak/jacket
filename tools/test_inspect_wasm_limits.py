"""Pure metadata controls; no compiler, SDK or WebAssembly execution."""

import unittest

from inspect_wasm_limits import MAX_BYTES, inspect

HEADER = b"\x00asm\x01\x00\x00\x00"


class Limits(unittest.TestCase):
    def test_table32_and_memory64_are_separate(self):
        result = inspect(HEADER + bytes([4, 4, 1, 0x70, 0, 1, 5, 3, 1, 4, 1]))
        table, memory = result["defined_limits"]
        self.assertEqual((table["flags"], table["index_bits"]), (0, 32))
        self.assertEqual((memory["flags"], memory["index_bits"]), (4, 64))
        self.assertEqual(table["flags_offset"], 12)

    def test_table64_records_flags_and_maximum(self):
        table = inspect(HEADER + bytes([4, 5, 1, 0x70, 5, 1, 9]))["defined_limits"][0]
        self.assertEqual((table["flags"], table["index_bits"]), (5, 64))
        self.assertEqual((table["minimum"], table["maximum"]), (1, 9))

    def test_wide_table_limits_are_not_truncated_to_u32(self):
        wide = bytes([0x81, 0x80, 0x80, 0x80, 0x10])
        table = inspect(HEADER + bytes([4, 8, 1, 0x70, 4]) + wide)["defined_limits"][0]
        self.assertEqual(table["minimum"], (1 << 32) + 1)
        with self.assertRaises(ValueError):
            inspect(HEADER + bytes([4, 8, 1, 0x70, 0]) + wide)

    def test_capture_bound_is_enforced(self):
        with self.assertRaisesRegex(ValueError, "2 MiB"):
            inspect(HEADER + bytes(MAX_BYTES))

    def test_truncated_limits_remain_errors(self):
        for data in [bytes([4, 9, 1]), bytes([4, 3, 1, 0x70, 4])]:
            with self.assertRaises(ValueError):
                inspect(HEADER + data)

    def test_overflow_and_unknown_formats_remain_errors(self):
        for data in [
            bytes([4, 4, 1, 0x70, 8, 1]),
            bytes([4, 9, 1, 0x70, 0, 255, 255, 255, 255, 16]),
        ]:
            with self.assertRaises(ValueError):
                inspect(HEADER + data)


if __name__ == "__main__":
    unittest.main()
