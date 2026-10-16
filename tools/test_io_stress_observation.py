"""Supervisor/configuration tests only; no Jai/native/network execution."""

import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import MagicMock, patch

import io_stress as stress


class Observation(unittest.TestCase):
    def fixture(self):
        base = Path(os.environ["CARGO_TARGET_DIR"])
        base.mkdir(parents=True, exist_ok=True)
        return tempfile.TemporaryDirectory(dir=base)

    def test_same_child_cwd_streams_timeout_and_exit_without_retry(self):
        with self.fixture() as temporary:
            root = Path(temporary)
            program = root / "own-metadata-only"
            program.write_bytes(b"not executed")
            process = MagicMock(pid=42, returncode=-6)
            process.communicate.return_value = ("before\n", "real failure")
            context = MagicMock()
            context.__enter__.return_value = process
            with (
                patch.object(stress.subprocess, "Popen", return_value=context) as popen,
                patch.object(stress, "observe_process") as observe,
            ):
                result = stress.observed_run(program, root, 17, root / "report")
            self.assertEqual(result.returncode, -6)
            self.assertEqual(
                (result.stdout, result.stderr), ("before\n", "real failure")
            )
            popen.assert_called_once_with(
                [str(program)],
                cwd=root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                errors="replace",
            )
            process.communicate.assert_called_once_with(timeout=17)
            process.kill.assert_not_called()
            observe.assert_called_once()
            self.assertEqual(
                json.loads((root / "report/receipt.json").read_text())["exit"], -6
            )

    def test_only_original_supervisor_timeout_kills_child(self):
        with self.fixture() as temporary:
            root = Path(temporary)
            program = root / "own-metadata-only"
            program.write_bytes(b"not executed")
            process = MagicMock(pid=42, returncode=-9)
            process.communicate.side_effect = [
                subprocess.TimeoutExpired(
                    [str(program)], 17, output=b"before\n", stderr=b"actual stderr"
                ),
                ("before\n", "actual stderr"),
            ]
            context = MagicMock()
            context.__enter__.return_value = process
            with (
                patch.object(stress.subprocess, "Popen", return_value=context),
                patch.object(stress, "observe_process"),
                self.assertRaises(subprocess.TimeoutExpired),
            ):
                stress.observed_run(program, root, 17, root / "report")
            process.kill.assert_called_once()
            self.assertEqual(
                process.communicate.call_args_list[0].kwargs, {"timeout": 17}
            )
            self.assertEqual(
                (root / "report/program.stderr").read_text(), "actual stderr"
            )
            self.assertTrue(
                json.loads((root / "report/receipt.json").read_text())[
                    "supervisor_timeout"
                ]
            )

    def test_observer_failure_is_reported_and_not_counted_as_success(self):
        with self.fixture() as temporary:
            root = Path(temporary)
            program = root / "own-metadata-only"
            program.write_bytes(b"not executed")
            process = MagicMock(pid=42, returncode=0)
            process.communicate.return_value = ("", "")
            context = MagicMock()
            context.__enter__.return_value = process
            with (
                patch.object(stress.subprocess, "Popen", return_value=context),
                patch.object(
                    stress,
                    "observe_process",
                    side_effect=RuntimeError("observer broke"),
                ),
            ):
                problem = stress.run_copy(program, root / "cwd", 17, root / "report")
            self.assertIn("observer broke", problem)
            self.assertEqual(
                json.loads((root / "report/receipt.json").read_text())["exit"], 0
            )

    def test_single_observation_preserves_existing_twelve_copy_workload(self):
        with self.fixture() as temporary:
            root = Path(temporary)
            source = root / "backends.jai"
            source.write_text("main :: () {}")
            argv = [
                "io_stress",
                "--copies",
                "12",
                "--rounds",
                "3",
                "--variants",
                "default,o2,asan",
                "--observe-copy",
                "backends:o2:1:0",
                "--observe-report",
                str(root / "report"),
                str(source),
            ]
            with (
                patch("sys.argv", argv),
                patch.object(stress.sys, "platform", "linux"),
                patch.object(stress, "ensure_native_libraries"),
                patch.object(
                    stress.tempfile, "mkdtemp", return_value=str(root / "work")
                ),
                patch.object(stress, "build", return_value=None),
                patch.object(stress, "run_copy", return_value=None) as run,
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(stress.main(), 0)
            self.assertEqual(run.call_count, 3 * 3 * 12)
            observed = [call for call in run.call_args_list if call.args[3] is not None]
            self.assertEqual(len(observed), 1)
            self.assertEqual(observed[0].args[2], 180)
            self.assertEqual(observed[0].args[0].parent.name, "o2")
            self.assertEqual(observed[0].args[1].name, "0")


if __name__ == "__main__":
    unittest.main()
