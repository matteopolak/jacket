"""The complete negative-suite ledger must keep private error text in logs."""

import contextlib
import io
import json
from pathlib import Path
import runpy
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from bench import sandbox


class Peer:
    def __init__(self, command, **_kwargs):
        Path(command[-2]).write_text("12345")

    def poll(self):
        return None

    def wait(self, **_kwargs):
        return 0


class NegativeReports(unittest.TestCase):
    def test_setup_errors_keep_full_inventory_without_exporting_private_text(self):
        marker = "PRIVATE_PATH_SENTINEL synthetic-token-context"
        script = Path(__file__).parent / "conformance" / "run-negative.py"
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            arguments = [str(script)]
            for name, value in {
                "upstream": root / "upstream",
                "python": root / "venv/bin/python",
                "raw-client": root / "raw",
                "typed-client": root / "typed",
                "scratch": root / "scratch",
                "output": root / "output",
            }.items():
                arguments.extend(["--" + name, str(value)])
            with (
                patch.object(sys, "argv", arguments),
                patch.object(
                    sandbox,
                    "Sandbox",
                    return_value=SimpleNamespace(wrap=lambda command: command),
                ),
                patch(
                    "subprocess.check_output",
                    return_value="f2f2334fb937ec78b3dc629ccc163b06913deb40",
                ),
                patch("subprocess.Popen", Peer),
                patch(
                    "subprocess.run",
                    side_effect=FileNotFoundError(2, marker, "/private/fake-worktree"),
                ),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                with self.assertRaises(SystemExit) as stopped:
                    runpy.run_path(str(script), run_name="__main__")
            self.assertEqual(stopped.exception.code, 1)
            text = (root / "output/results.json").read_text()
            rows = json.loads(text)["results"]
            self.assertEqual(len(rows), 16)
            self.assertEqual(len({(row["mode"], row["case"]) for row in rows}), 16)
            self.assertTrue(all(row["outcome"] == "ERROR" for row in rows))
            self.assertNotIn(marker, text)
            self.assertNotIn("/private/fake-worktree", text)
            self.assertTrue(all(row["detail"] == "FileNotFoundError" for row in rows))
            logs = list((root / "output").glob("*.peer.log"))
            self.assertEqual(len(logs), 16)
            self.assertTrue(all(marker in log.read_text() for log in logs))
            self.assertTrue(
                all("/private/fake-worktree" in log.read_text() for log in logs)
            )


if __name__ == "__main__":
    unittest.main()
