"""The target notes the manifest migrator writes."""

import unittest

from stdlib.migrate_to_jacket_test import notes_of


class TargetNotes(unittest.TestCase):
    def test_a_platform_names_its_os_and_cpu_with_cpu(self):
        notes, problems = notes_of({"unsupported_platforms": ["windows-arm64"]})
        self.assertEqual(problems, [])
        self.assertEqual(notes, ["skip_on(.WINDOWS, cpu = .ARM64)"])

    def test_one_mode_of_wasm_names_it_with_mode(self):
        notes, _ = notes_of({"unsupported_modes": ["wasm-native"]})
        self.assertEqual(notes, ["skip_on(.WASM, mode = .NATIVE)"])
        notes, _ = notes_of({"unsupported_modes": ["wasm-interp"]})
        self.assertEqual(notes, ["skip_on(.WASM, mode = .INTERP)"])

    def test_both_wasm_modes_are_the_whole_os(self):
        notes, _ = notes_of({"unsupported_modes": ["wasm-interp", "wasm-native"]})
        self.assertEqual(notes, ["skip_on(.WASM)"])

    def test_each_pair_is_its_own_call(self):
        row = {
            "unsupported_modes": ["wasm-interp", "wasm-native"],
            "unsupported_platforms": ["windows-arm64"],
        }
        notes, _ = notes_of(row)
        self.assertEqual(notes, ["skip_on(.WASM)", "skip_on(.WINDOWS, cpu = .ARM64)"])

    def test_plain_modes_stay_in_one_call(self):
        notes, _ = notes_of({"unsupported_modes": ["interp", "native"]})
        self.assertEqual(notes, ["skip_on(.INTERP,.NATIVE)"])


if __name__ == "__main__":
    unittest.main()
