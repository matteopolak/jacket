"""Regression checks for preserving failed subprocess evidence and immutable inventories."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from run import observed_result, run

HERE = Path(__file__).resolve().parent


class GateTests(unittest.TestCase):
    def command(self, source, allow_failure=False, timeout=None):
        """Execute a tiny real process using the same log contract as a decoder probe."""
        with tempfile.TemporaryFile(mode="w+") as log:
            result = run(
                [sys.executable, "-c", source],
                environment=os.environ,
                log=log,
                allow_failure=allow_failure,
                timeout=timeout,
            )
            log.seek(0)
            return result, log.read()

    def test_failure_can_be_recorded_without_becoming_a_success(self):
        (transcript, code), log = self.command(
            "print('RESULT example false'); raise SystemExit(7)", True
        )
        self.assertEqual(code, 7)
        self.assertIn("RESULT example false", transcript)
        self.assertEqual(log, transcript)

    def test_timeout_remains_a_failure_with_partial_output(self):
        (transcript, code), log = self.command(
            "import time; print('started',flush=True); time.sleep(30)", True, 0.2
        )
        self.assertEqual(code, "timeout")
        self.assertIn("started", transcript)
        self.assertIn("TIMEOUT", transcript)
        self.assertEqual(log, transcript)

    @unittest.skipUnless(os.name == "posix", "owned group cleanup requires POSIX")
    def test_timeout_terminates_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "child-survived"
            child = (
                "import pathlib,time; print('child started',flush=True); time.sleep(1.5); pathlib.Path("
                + repr(str(marker))
                + ").write_text('survived')"
            )
            source = (
                "import subprocess,sys,time; subprocess.Popen([sys.executable,'-c',"
                + repr(child)
                + "]); print('parent started',flush=True); time.sleep(30)"
            )
            (transcript, code), _ = self.command(source, True, 0.5)
            self.assertEqual(code, "timeout")
            import time

            self.assertIn("child started", transcript)
            time.sleep(1.6)
            self.assertFalse(marker.exists())

    def test_required_command_failure_remains_fatal_with_its_output(self):
        with self.assertRaises(subprocess.CalledProcessError) as caught:
            self.command("print('specific failure'); raise SystemExit(9)")
        self.assertEqual(caught.exception.returncode, 9)
        self.assertIn("specific failure", caught.exception.output)

    def test_successful_command_retains_its_output(self):
        (transcript, code), _ = self.command("print('RESULT example true')")
        self.assertEqual(code, 0)
        self.assertIn("RESULT example true", transcript)

    def test_complete_pinned_inventory_has_no_duplicate_or_unpinned_inputs(self):
        manifest = json.loads((HERE / "manifest.json").read_text())
        files = {item["path"]: item for item in manifest["files"]}
        self.assertEqual(len(files), len(manifest["files"]))
        self.assertEqual(len(manifest["png_cases"]), 176)
        self.assertEqual(sum(case["valid"] for case in manifest["png_cases"]), 162)
        for case in manifest["png_cases"]:
            key = "pngsuite/" + (
                case["name"]
                if case["name"] == "PngSuite.png"
                else "png/" + case["name"]
            )
            self.assertEqual(files[key]["sha256"], case["sha256"])
            self.assertEqual("oracle_sha256" in case, case["valid"])
        self.assertEqual(
            sum(
                path.startswith("pillow-ico/assets/") and path.endswith(".ico")
                for path in files
            ),
            10,
        )

    def test_missing_case_is_unexecuted_instead_of_rejected(self):
        self.assertEqual(observed_result("decode_corpus", None), {"executed": False})

    def test_accepted_bad_corpus_input_retains_its_actual_dimensions(self):
        result = observed_result("decode_corpus", "1 32 16 ")
        self.assertTrue(result["accepted"])
        self.assertEqual((result["width"], result["height"]), (32, 16))

    def test_existing_result_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            result = Path(directory) / "result.json"
            result.write_text("retained evidence")
            process = subprocess.run(
                [sys.executable, HERE / "run.py", "--output", directory],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(process.returncode, 0)
            self.assertIn("already contains a result", process.stderr)
            self.assertEqual(result.read_text(), "retained evidence")


if __name__ == "__main__":
    unittest.main()
