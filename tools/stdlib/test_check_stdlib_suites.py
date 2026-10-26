"""The suite driver selects, excludes and groups suites, and keeps failures nonzero."""

import os
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest import mock

import check_stdlib_suites as suites
class Driver(unittest.TestCase):
    def run_main(self, *extra, fail=()):
        started = []
        lock = threading.Lock()
        running, peak = [0], [0]

        def fake_run(command, **kwargs):
            with lock:
                started.append(command)
                running[0] += 1
                peak[0] = max(peak[0], running[0])
            threading.Event().wait(0.05)
            with lock:
                running[0] -= 1
            code = int(any(f in str(command[1]) for f in fail))
            return subprocess.CompletedProcess(command, code, stdout="out\n")

        with tempfile.TemporaryDirectory() as tmp:
            files = []
            for name in ("jaic", "jailint", "jaifmt"):
                path = Path(tmp) / name
                path.write_text("")
                files.append(str(path))
            argv = [
                "check_stdlib_suites.py",
                "--jaic", files[0],
                "--jailint", files[1],
                "--jaifmt", files[2],
                "--stdlib", str(suites.ROOT / "stdlib"),
                "--native-libs", tmp,
                "--font", files[0],
                *extra,
            ]
            with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": tmp}), mock.patch.object(sys, "argv", argv), mock.patch.object(
                suites.subprocess, "run", fake_run
            ):
                failed = suites.main()
        return failed, started, peak[0]

    def test_serial_by_default_and_exclude(self):
        failed, started, peak = self.run_main("--exclude", "tooling")
        self.assertFalse(failed)
        self.assertEqual(len(started), len(suites.SUITES) - 1)
        self.assertEqual(peak, 1)

    def test_tooling_part_reaches_only_the_tooling_command(self):
        failed, started, _ = self.run_main("--tooling-part", "2/4")
        self.assertFalse(failed)
        tails = {str(command[1]): command[-2:] for command in started}
        parted = [path for path, tail in tails.items() if tail == ["--part", "2/4"]]
        self.assertEqual([Path(path).name for path in parted], ["check_developer_tools.py"])
        with self.assertRaises(SystemExit):
            self.run_main("--tooling-part", "5/4")

    def test_groups_run_concurrently(self):
        failed, started, peak = self.run_main("--jobs", "4")
        self.assertFalse(failed)
        self.assertEqual(len(started), len(suites.SUITES))
        self.assertGreater(peak, 1)

    def test_a_failing_suite_fails_the_run_and_the_others_still_run(self):
        failed, started, _ = self.run_main("--jobs", "3", fail=("check_foundations",))
        self.assertTrue(failed)
        self.assertEqual(len(started), len(suites.SUITES))

    def test_groups_cover_every_suite_once(self):
        flat = [s for group in suites.GROUPS for s in group]
        self.assertEqual(sorted(flat), sorted(suites.SUITES))


if __name__ == "__main__":
    unittest.main()
