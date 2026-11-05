"""The suite driver selects, excludes and groups suites, and keeps failures nonzero."""

import os
import io
import shutil
import subprocess
import sys
import tempfile
import threading
from concurrent.futures import ThreadPoolExecutor
import unittest
from pathlib import Path
from unittest import mock

import check_stdlib_suites as suites
class Driver(unittest.TestCase):
    def run_main(self, *extra, fail=()):
        started = []
        self.environments = environments = {}
        lock = threading.Lock()
        running, peak = [0], [0]

        def fake_run(command, **kwargs):
            with lock:
                started.append(command)
                environments[str(command[1])] = kwargs.get("env", {})
                running[0] += 1
                peak[0] = max(peak[0], running[0])
            threading.Event().wait(0.05)
            with lock:
                running[0] -= 1
            code = int(any(f in str(command[1]) for f in fail))
            return subprocess.CompletedProcess(command, code, stdout="out\n")

        with tempfile.TemporaryDirectory() as tmp:
            files = []
            path = Path(tmp) / "jaic"
            path.write_text("")
            files.append(str(path))
            argv = [
                "check_stdlib_suites.py",
                "--jaic", files[0],
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
        parted = [path for path, env in self.environments.items() if env.get("JACKET_TOOLING_PART") == "2/4"]
        self.assertEqual([Path(path).name for path in parted], ["check_developer_tools.py"])
        with self.assertRaises(SystemExit):
            self.run_main("--tooling-part", "5/4")

    def test_groups_run_concurrently(self):
        failed, started, peak = self.run_main("--jobs", "4")
        self.assertFalse(failed)
        self.assertEqual(len(started), len(suites.SUITES))
        self.assertGreater(peak, 1)

    def test_a_failing_suite_fails_the_run_and_the_others_still_run(self):
        failed, started, _ = self.run_main("--jobs", "3", fail=("check_portable_stdlib",))
        self.assertTrue(failed)
        self.assertEqual(len(started), len(suites.SUITES))

    def test_native_helper_snapshot_cannot_overlap_tooling_rebuild(self):
        # Model the linker removing its output while bindings takes a real tree snapshot.
        # An independent resource group may overlap; bindings must observe the completed tool.
        mutation_started, neighbour_finished = threading.Event(), threading.Event()
        commands = []
        peak, running = [0], [0]
        lock = threading.Lock()
        pool_type = ThreadPoolExecutor
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            library = root / "stdlib"
            helper = library / "internal/developer-tools/clang-bindings/developer-tools-clang"
            helper.parent.mkdir(parents=True)
            helper.write_bytes(b"complete helper")
            compiler = root / "jaic"
            compiler.write_text("")
            destination = root / "snapshot"

            class CoordinatedPool:
                def __init__(self, workers):
                    self.pool = pool_type(workers)
                def __enter__(self):
                    return self
                def __exit__(self, *args):
                    self.pool.shutdown(wait=True)
                def map(self, operation, groups):
                    def run(group):
                        if "tooling" in group:
                            return operation(group)
                        if not mutation_started.wait(5):
                            raise AssertionError("tooling did not reach controlled rebuild")
                        try:
                            return operation(group)
                        finally:
                            neighbour_finished.set()
                    return self.pool.map(run, groups)

            def fake_run(command, **kwargs):
                name = Path(command[1]).name
                with lock:
                    commands.append(name)
                    running[0] += 1
                    peak[0] = max(peak[0], running[0])
                code, output = 0, ""
                try:
                    if name == "check_developer_tools.py":
                        helper.unlink()
                        mutation_started.set()
                        if not neighbour_finished.wait(5):
                            raise AssertionError("independent group did not complete")
                        helper.write_bytes(b"complete helper")
                    elif name == "verify.py":
                        try:
                            shutil.copytree(library, destination)
                            self.assertEqual((destination / helper.relative_to(library)).read_bytes(), b"complete helper")
                        except (OSError, shutil.Error) as error:
                            code, output = 1, str(error)
                    return subprocess.CompletedProcess(command, code, stdout=output)
                finally:
                    with lock:
                        running[0] -= 1

            argv = ["check_stdlib_suites.py", "--jaic", str(compiler), "--stdlib", str(library),
                    "--native-libs", str(root), "--font", str(compiler), "--jobs", "2"]
            with mock.patch.object(suites, "ROOT", root), mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": str(root / "target")}), mock.patch.object(sys, "argv", argv), mock.patch.object(suites.subprocess, "run", fake_run), mock.patch.object(suites.concurrent.futures, "ThreadPoolExecutor", CoordinatedPool), mock.patch("sys.stdout", io.StringIO()):
                failed = suites.main()
            self.assertFalse(failed, "bindings copied the native helper during its rebuild")
            self.assertEqual(len(commands), len(suites.SUITES))
            self.assertEqual(len(commands), len(set(commands)))
            self.assertGreater(peak[0], 1, "independent resource groups must remain concurrent")

    def test_groups_cover_every_suite_once(self):
        flat = [s for group in suites.GROUPS for s in group]
        self.assertEqual(sorted(flat), sorted(suites.SUITES))


if __name__ == "__main__":
    unittest.main()
