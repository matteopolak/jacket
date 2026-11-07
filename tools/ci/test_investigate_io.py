import contextlib
import io
import json
import os
import signal
import subprocess
import time
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import investigate_io


@unittest.skipUnless(os.name == "posix", "native Linux diagnostic process fixture")
class Attempts(unittest.TestCase):
    def test_failures_keep_the_predeclared_denominator_and_all_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            jacket = root / "jacket"
            jacket.write_text("#!/usr/bin/env python3\nimport sys\nif '--list' in sys.argv:\n print('{\"name\":\"backends\"}')\nelse:\n print('deliberate diagnostic failure', file=sys.stderr)\n raise SystemExit(1)\n")
            jacket.chmod(0o755)
            (root / "jaic").write_text("controlled branch compiler identity")
            output = root / "evidence"
            with mock.patch.object(sys, "argv", ["investigate_io", "--jacket", str(jacket), "--out", str(output), "--attempts", "5"]), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(investigate_io.main(), 1)
            records = json.loads((output / "outcomes.json").read_text())
            self.assertEqual(len(records), 15)
            self.assertTrue(all(record["state"] == "completed" for record in records))
            environment = json.loads((output / "environment.json").read_text())
            self.assertNotIn("uname", environment)
            self.assertNotIn("platform", environment)
            self.assertNotIn("/proc/version", environment)
            self.assertNotIn(str(root), json.dumps(environment))
            for mode in ["native", "interp", "o2"]:
                self.assertEqual([r["attempt"] for r in records if r["mode"] == mode], [1, 2, 3, 4, 5])
                for attempt in range(1, 6):
                    self.assertIn("deliberate diagnostic failure", (output / f"{mode}-{attempt}.stderr").read_text())
                    self.assertTrue((output / f"{mode}-{attempt}.command.json").is_file())

    def test_hung_supervisor_keeps_durable_running_state_without_advancing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            jacket = root / "jacket"
            jacket.write_text("#!/usr/bin/env python3\nimport sys,time\nif '--list' in sys.argv:\n print('{\"name\":\"backends\"}')\nelse:\n print('attempt began',flush=True)\n time.sleep(30)\n")
            jacket.chmod(0o755)
            (root / "jaic").write_text("controlled compiler")
            output = root / "evidence"
            command = [sys.executable, str(Path(investigate_io.__file__)), "--jacket", str(jacket), "--out", str(output)]
            process = subprocess.Popen(command, env={**os.environ, "PYTHONPATH": str(investigate_io.ROOT / "tools")}, start_new_session=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
            try:
                deadline = time.monotonic() + 5
                stream = output / "native-1.stdout"
                while time.monotonic() < deadline and (not stream.exists() or not stream.read_bytes()):
                    time.sleep(0.02)
                self.assertTrue(stream.exists(), process.poll())
                self.assertIn("attempt began", stream.read_text())
                records = json.loads((output / "outcomes.json").read_text())
                self.assertEqual(len(records), 15)
                self.assertEqual(records[0]["state"], "running")
                self.assertTrue(all(record["state"] == "planned" for record in records[1:]))
                self.assertIsNone(process.poll())
                self.assertFalse((output / "native-2.command.json").exists())
            finally:
                # This controlled fixture does not detach: its entire private group is ours.
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate(timeout=5)
