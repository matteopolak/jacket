"""Pure harness controls, not native init/fault proof."""

import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import check_portable_crash_init as harness


class InitControls(unittest.TestCase):
    def test_linux_keeps_pie_exec_shared_and_interposition_at_both_architectures(self):
        for cpu in ("x64", "arm64"):
            variants = harness.control_variants("linux-" + cpu)
            self.assertEqual(
                set(variants),
                {"static-pie", "static-exec", "shared", "shared-interposed"},
            )
            self.assertIn("-pie", variants["static-pie"])
            self.assertIn("-no-pie", variants["static-exec"])
            self.assertIn("-DCRASH_INTERPOSE", variants["shared-interposed"])
        self.assertEqual(
            set(harness.control_variants("macos-arm64")), {"static", "shared"}
        )
        with self.assertRaises(ValueError):
            harness.control_variants("windows-arm64")

    def test_compiler_failure_stays_fatal_and_retains_receipt(self):
        with TemporaryDirectory() as temporary:
            output = Path(temporary)
            failed = subprocess.CompletedProcess(
                ["cc"], 42, b"", b"independent setup failure"
            )
            with (
                patch("sys.argv", ["check", "--out", str(output)]),
                patch.object(harness, "host_platform", return_value="macos-arm64"),
                patch.object(harness.subprocess, "run", return_value=failed),
            ):
                with self.assertRaisesRegex(RuntimeError, "provider-0 failed: 42"):
                    harness.main()
            receipt = json.loads((output / "receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["commands"][0]["exit"], 42)
            self.assertEqual(receipt["commands"][0]["stderr_hex"], failed.stderr.hex())
            self.assertFalse(
                any(c["label"].startswith("run-") for c in receipt["commands"])
            )


if __name__ == "__main__":
    unittest.main()
