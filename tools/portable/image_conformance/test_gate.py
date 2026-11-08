"""Regression checks for preserving failed subprocess evidence and immutable inventories."""

import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile
import unittest

from fetch import png_cases, pins
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

    def test_required_command_failure_remains_fatal_with_its_output(self):
        with self.assertRaises(subprocess.CalledProcessError) as caught:
            self.command("print('specific failure'); raise SystemExit(9)")
        self.assertEqual(caught.exception.returncode, 9)
        self.assertIn("specific failure", caught.exception.output)

    def test_successful_command_retains_its_output(self):
        (transcript, code), _ = self.command("print('RESULT example true')")
        self.assertEqual(code, 0)
        self.assertIn("RESULT example true", transcript)

    def test_pins_cover_the_complete_corpora(self):
        manifest = json.loads((HERE / "manifest.json").read_text())
        files = pins(manifest)
        self.assertEqual(len(files), sum(len(s["files"]) for s in manifest["sets"]))
        self.assertTrue(all(re.fullmatch("[0-9a-f]{64}", d) for _, d in files.values()))
        self.assertEqual(
            sum(path.startswith("pillow-ico/") and path.endswith(".ico") for path in files),
            10,
        )
        suite = [p for p in files if p.startswith("pngsuite/png/")]
        self.assertEqual(len(suite) + 1, 176)
        self.assertEqual(len(manifest["png_oracle"]), 162)

    def test_png_cases_come_from_the_downloaded_files(self):
        header = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR" + struct.pack(">II", 32, 16)
        with tempfile.TemporaryDirectory() as directory:
            suite = Path(directory) / "pngsuite"
            (suite / "png").mkdir(parents=True)
            (suite / "PngSuite.png").write_bytes(header)
            (suite / "png" / "basn0g01.png").write_bytes(header)
            (suite / "png" / "xs1n0g01.png").write_bytes(b"broken")
            cases = png_cases(Path(directory), {"PngSuite.png": "a", "basn0g01.png": "b"})
        self.assertEqual(
            [(c["name"], c["valid"], c["width"], c["height"]) for c in cases],
            [
                ("PngSuite.png", True, 32, 16),
                ("basn0g01.png", True, 32, 16),
                ("xs1n0g01.png", False, 0, 0),
            ],
        )
        self.assertEqual([c.get("oracle_sha256") for c in cases], ["a", "b", None])

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
