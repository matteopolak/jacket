import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import run_tests


class Discovery(unittest.TestCase):
    def test_every_tools_folder_with_tests_is_discovered(self):
        names = {folder.relative_to(run_tests.TOOLS).as_posix() for folder in run_tests.folders()}
        expected = {p.parent.relative_to(run_tests.TOOLS).as_posix() for p in run_tests.TOOLS.rglob("test_*.py")}
        self.assertTrue(expected - {"."} <= names)
        self.assertTrue({"native_bindings", "numerics", "ci"} <= names)

    def test_a_pattern_narrows_the_folders(self):
        names = {folder.name for folder in run_tests.folders("test_bump_version.py")}
        self.assertEqual(names, {"tools"})

    def test_hand_run_scripts_do_not_match_the_test_pattern(self):
        self.assertFalse(list(run_tests.TOOLS.rglob("test_sdl3.py")))
        self.assertFalse(list(run_tests.TOOLS.rglob("test_webgpu_native.py")))

    def test_fixtures_and_timeouts_name_real_folders(self):
        existing = {folder.name for folder in run_tests.folders()}
        self.assertLessEqual(set(run_tests.FIXTURES), existing)
        self.assertLessEqual(set(run_tests.FOLDER_TIMEOUTS), existing)


class Processes(unittest.TestCase):
    def test_output_and_status_come_back(self):
        status, out, err = run_tests.run_process([sys.executable, "-c", "import sys; print('a'); print('b', file=sys.stderr); sys.exit(3)"], dict(os.environ), 30)
        self.assertEqual((status, out.strip(), err.strip()), (3, "a", "b"))

    def test_a_process_over_its_limit_is_killed_with_its_children(self):
        child = "import subprocess, sys, time; subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']); time.sleep(60)"
        status, _, _ = run_tests.run_process([sys.executable, "-c", child], dict(os.environ), 1)
        self.assertIsNone(status)


class Fixtures(unittest.TestCase):
    def make(self, prepare):
        fixture = run_tests.Fixture(("test_*.py",), prepare)
        return run_tests.Fixtures({"ci": fixture})

    def folder(self):
        return run_tests.TOOLS / "ci"

    def test_the_fixture_runs_once_and_its_environment_is_returned(self):
        calls = []
        fixtures = self.make(lambda: calls.append(1) or {"X": "1"})
        for _ in range(3):
            self.assertEqual(fixtures.environment(self.folder(), ["test_*.py"]), {"X": "1"})
        self.assertEqual(calls, [1])

    def test_a_fixture_only_runs_for_files_that_use_it(self):
        fixtures = run_tests.Fixtures({"ci": run_tests.Fixture(("test_sdl*.py",), lambda: self.fail("ran"))})
        self.assertEqual(fixtures.environment(self.folder(), ["test_pins.py"]), {})

    def test_a_skip_is_remembered_and_raised(self):
        def refuse():
            raise run_tests.Skip("no network")

        fixtures = self.make(refuse)
        for _ in range(2):
            with self.assertRaises(run_tests.Skip):
                fixtures.environment(self.folder(), ["test_*.py"])


class Runs(unittest.TestCase):
    def run_folder(self, pattern, table=None, only=("ci",), timeout=None):
        with mock.patch.object(run_tests, "report"):
            return run_tests.run_all([pattern], list(only), 2, False, timeout, table or {}, dict(os.environ))

    def test_a_folder_runs_in_its_own_process_with_tools_on_the_path(self):
        (outcome,) = self.run_folder("test_pins.py")
        self.assertEqual((outcome.status, outcome.name), ("ok", "ci"))
        self.assertGreater(outcome.tests, 0)

    def test_a_pattern_may_be_a_path_to_the_test_file(self):
        for given in ("tools/ci/test_pins.py", "ci/test_pins.py", "./tools/ci/test_pins.py"):
            (outcome,) = self.run_folder(given, only=())
            self.assertEqual((outcome.status, outcome.name), ("ok", "ci"), given)
            self.assertGreater(outcome.tests, 0)
        self.assertEqual(self.run_folder("tools/stdlib/test_pins.py", only=()), [])

    def test_a_skipped_fixture_skips_the_folder(self):
        def refuse():
            raise run_tests.Skip("not here")

        table = {"ci": run_tests.Fixture(("test_pins.py",), refuse)}
        (outcome,) = self.run_folder("test_pins.py", table)
        self.assertEqual(outcome.status, "skipped")
        self.assertIn("not here", outcome.output)

    def test_a_failing_folder_is_reported_failed(self):
        with tempfile.TemporaryDirectory() as tmp:
            failing = Path(tmp) / "test_always_fails.py"
            failing.write_text("import unittest\nclass T(unittest.TestCase):\n    def test_x(self):\n        self.fail('boom')\n")
            outcome = run_tests.run_task(Path(tmp), "test_always_fails.py", False, 60, run_tests.Fixtures({}), ["test_always_fails.py"], dict(os.environ))
        self.assertEqual(outcome.status, "failed")
        self.assertEqual(outcome.tests, 1)
        self.assertIn("boom", outcome.output)

    def test_a_folder_over_its_timeout_is_reported_not_hung(self):
        with tempfile.TemporaryDirectory() as tmp:
            slow = Path(tmp) / "test_slow.py"
            slow.write_text("import time, unittest\nclass T(unittest.TestCase):\n    def test_x(self):\n        time.sleep(60)\n")
            outcome = run_tests.run_task(Path(tmp), "test_slow.py", False, 1, run_tests.Fixtures({}), ["test_slow.py"], dict(os.environ))
        self.assertEqual(outcome.status, "timeout")
        self.assertIn("killed after 1 s", outcome.output)


if __name__ == "__main__":
    unittest.main()
