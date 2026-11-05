"""Check that bounded decoder processes retain output and release their owned group."""

import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "jpeg_gate", Path(__file__).with_name("run.py")
)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


@unittest.skipUnless(os.name == "posix", "JPEG process isolation requires POSIX")
class ProcessTests(unittest.TestCase):
    def execute(self, source, timeout):
        with tempfile.TemporaryDirectory() as directory:
            return gate.process(
                [sys.executable, "-c", source],
                cwd=directory,
                env=os.environ.copy(),
                timeout=timeout,
            )

    def test_timeout_closes_descendant_output_pipes(self):
        source = (
            "import subprocess,sys,time; "
            "subprocess.Popen([sys.executable,'-c',"
            "\"import time; print('child started',flush=True); time.sleep(30)\"]); "
            "time.sleep(30)"
        )
        started = time.monotonic()
        result = self.execute(source, 0.5)
        self.assertEqual(result["exit"], "timeout")
        self.assertIn("child started", result["stdout"])
        self.assertLess(time.monotonic() - started, 5)

    def test_exit_during_timeout_cleanup_preserves_evidence(self):
        fake = unittest.mock.Mock()
        fake.pid = 123
        fake.communicate.side_effect = [
            subprocess.TimeoutExpired("probe", 1),
            (b"partial output", b"diagnostic"),
        ]
        with (
            patch.object(gate.subprocess, "Popen", return_value=fake),
            patch.object(gate.os, "killpg", side_effect=ProcessLookupError),
        ):
            result = self.execute("", 1)
        self.assertEqual(result["exit"], "timeout")
        self.assertEqual(result["stdout"], "partial output")
        self.assertEqual(result["stderr"], "diagnostic")


if __name__ == "__main__":
    unittest.main()
