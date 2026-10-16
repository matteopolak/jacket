"""The native elapsed clock must not remove portable slice storage coverage."""

import unittest

from stdlib_test_catalog import unavailable_reason, validated_entries


class StorageClockTests(unittest.TestCase):
    def test_only_the_mixed_clock_program_requires_native_clock_services(self):
        cases = {case["id"]: case for case in validated_entries()}
        mixed = cases["contracts/basic-slice-resize-and-monotonic-time-contracts"]
        storage = cases["contracts/basic-slice-resize-contracts"]
        for mode in ("wasm-interp", "wasm-native"):
            reason = unavailable_reason(mixed, mode, "macos-arm64")
            self.assertIsNotNone(reason)
            self.assertIn("Basic monotonic/elapsed clock", reason)
            self.assertIsNone(unavailable_reason(storage, mode, "macos-arm64"))
        for platform in ("macos-arm64", "linux-x64", "windows-arm64"):
            self.assertIsNone(unavailable_reason(mixed, "native", platform))
            self.assertIsNone(unavailable_reason(storage, "native", platform))


if __name__ == "__main__":
    unittest.main()
