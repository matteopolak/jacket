"""Which `jacket test` programs a cross-target source check covers."""

import unittest

from stdlib.jacket_tests import COMPILE_TIME_DRIVERS, golden_sources, runs_on
from toolkit.repo import ROOT


class RunsOn(unittest.TestCase):
    def test_no_notes_run_everywhere(self):
        self.assertTrue(runs_on([], "windows", "arm64"))

    def test_only_names_must_match_for_every_kind_of_target(self):
        notes = ["only(.LINUX,.X64)"]
        self.assertTrue(runs_on(notes, "linux", "x64"))
        self.assertFalse(runs_on(notes, "linux", "arm64"))
        self.assertFalse(runs_on(notes, "macos", "x64"))

    def test_os_alternatives_are_any_of(self):
        notes = ["only(.MACOS,.WINDOWS)"]
        self.assertTrue(runs_on(notes, "windows", "x64"))
        self.assertTrue(runs_on(notes, "macos", "arm64"))
        self.assertFalse(runs_on(notes, "linux", "x64"))

    def test_skip_on_removes_a_target(self):
        self.assertFalse(runs_on(["skip_on(.WASM)"], "wasm", "wasm"))
        self.assertTrue(runs_on(["skip_on(.WASM)"], "linux", "x64"))
        self.assertFalse(runs_on(["skip_on(.WINDOWS-ARM64)"], "windows", "arm64"))
        self.assertTrue(runs_on(["skip_on(.WINDOWS-ARM64)"], "windows", "x64"))

    def test_a_check_is_a_native_build(self):
        self.assertTrue(runs_on(["only(.WINDOWS-NATIVE)"], "windows", "x64"))
        self.assertFalse(runs_on(["skip_on(.NATIVE)"], "linux", "x64"))

    def test_wasm_has_its_own_cpu(self):
        self.assertFalse(runs_on(["only(.X64)"], "wasm", "wasm"))


class GoldenSources(unittest.TestCase):
    def test_expected_failures_and_drivers_are_left_out(self):
        sources = {path.relative_to(ROOT).as_posix() for path in golden_sources()}
        self.assertTrue(sources)
        self.assertFalse(sources & COMPILE_TIME_DRIVERS)
        self.assertNotIn("stdlib/Compiler/tests/invalid-workspace-diagnostic.jai", sources)


if __name__ == "__main__":
    unittest.main()
