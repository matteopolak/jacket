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
        notes = ["skip_on(.WINDOWS, cpu = .ARM64)"]
        self.assertFalse(runs_on(notes, "windows", "arm64"))
        self.assertTrue(runs_on(notes, "windows", "x64"))
        self.assertTrue(runs_on(notes, "macos", "arm64"))

    def test_several_skip_on_calls_each_remove_a_target(self):
        notes = ["skip_on(.WASM)", "skip_on(.WINDOWS, cpu = .ARM64)"]
        self.assertFalse(runs_on(notes, "wasm", "wasm"))
        self.assertFalse(runs_on(notes, "windows", "arm64"))
        self.assertTrue(runs_on(notes, "windows", "x64"))

    def test_mode_and_cpu_narrow_the_whole_call(self):
        self.assertFalse(runs_on(["skip_on(mode = .NATIVE)"], "linux", "x64"))
        self.assertFalse(runs_on(["skip_on(.WASM, mode = .NATIVE)"], "wasm", "wasm"))
        self.assertTrue(runs_on(["skip_on(.WASM, mode = .INTERP)"], "wasm", "wasm"))
        self.assertTrue(runs_on(["only(cpu = .X64)"], "linux", "x64"))
        self.assertFalse(runs_on(["only(cpu = .X64)"], "linux", "arm64"))

    def test_a_check_is_a_native_build(self):
        self.assertTrue(runs_on(["only(.WINDOWS, mode = .NATIVE)"], "windows", "x64"))
        self.assertFalse(runs_on(["only(.WINDOWS, mode = .INTERP)"], "windows", "x64"))
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
