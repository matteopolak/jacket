"""NEW receipt/protocol unit tests. These are not native SDK/device evidence."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("audio_builder", ROOT / "tools/audio_build_native.py")
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class ReceiptTests(unittest.TestCase):
    def test_child_scratch_uses_configured_build_root(self):
        environment = builder.native_environment()
        scratch = builder.build_root() / "audio-native/tmp"
        for name in ("TMPDIR", "TMP", "TEMP"):
            self.assertEqual(environment[name], str(scratch))

    def test_failure_is_written_before_it_is_raised(self):
        scratch = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "audio-provider-proof/receipt-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        record = {"kind": "receipt unit test (not native evidence)", "exit_code": 0,
                  "failure_reasons": ["stderr_nonempty"], "stderr": "unclassified diagnostic\n"}
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            out = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "stderr_nonempty"):
                builder.write_runtime_receipt(out, "proof.json", record)
            self.assertEqual(json.loads((out / "proof.json").read_text()), record)

    def test_exit_zero_diagnostic_is_preserved_and_fails(self):
        result = subprocess.CompletedProcess([], 0, "", "unclassified SDK diagnostic\n")
        receipt = builder.sdk_runtime_receipt(result, "linux-x64", False, {})
        self.assertEqual(receipt["stderr"], result.stderr)
        self.assertIn("stderr_nonempty", receipt["failure_reasons"])
        self.assertFalse(receipt["runtime_passed"])
        self.assertEqual(receipt["outcome"], "failed")

    def test_optional_unavailable_is_not_native_success(self):
        result = subprocess.CompletedProcess([], 0, "native-device-unavailable (-2)\n", "")
        receipt = builder.sdk_runtime_receipt(result, "linux-x64", False, {})
        self.assertFalse(receipt["device_opened"])
        self.assertFalse(receipt["runtime_passed"])
        self.assertEqual(receipt["outcome"], "unavailable")

    def test_required_device_is_enforced_independently(self):
        result = subprocess.CompletedProcess([], 0, "native-device-unavailable (-2)\n", "")
        receipt = builder.sdk_runtime_receipt(result, "linux-x64", True, {})
        self.assertIn("required_device_protocol_incomplete", receipt["failure_reasons"])
        self.assertEqual(receipt["outcome"], "failed")

    def test_unknown_output_is_not_a_success(self):
        result = subprocess.CompletedProcess([], 0, "unexpected output\n", "")
        receipt = builder.sdk_runtime_receipt(result, "linux-x64", False, {})
        self.assertIn("unexpected_stdout", receipt["failure_reasons"])
        self.assertFalse(receipt["runtime_passed"])


if __name__ == "__main__":
    unittest.main()
