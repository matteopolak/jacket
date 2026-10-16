"""Immutable captures may use local outputs without capturing themselves."""

import tempfile
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

import shared_snapshot


class SharedSnapshotTests(unittest.TestCase):
    def setUp(self):
        output = (
            Path(__file__).resolve().parents[2]
            / "artifacts/shared-integration/snapshot-tests"
        )
        output.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=output)
        self.root = Path(self.temporary.name)
        (self.root / "stdlib").mkdir()
        (self.root / "stdlib/module.jai").write_text("value :: 17;\n")
        self.patch = patch.object(shared_snapshot, "ROOT", self.root)
        self.patch.start()

    def tearDown(self):
        self.patch.stop()
        self.temporary.cleanup()

    def test_local_output_captures_only_inputs_and_detects_mutation(self):
        snapshot, receipt = shared_snapshot.capture(
            self.root / "artifacts/ordinary-output"
        )
        self.assertEqual(set(receipt["files"]), {"stdlib/module.jai"})
        self.assertEqual((snapshot / "stdlib/module.jai").read_text(), "value :: 17;\n")
        self.assertTrue(shared_snapshot.unchanged(snapshot, receipt))
        copied = snapshot / "stdlib/module.jai"
        copied.chmod(0o644)
        copied.write_text("value :: 18;\n")
        self.assertFalse(shared_snapshot.unchanged(snapshot, receipt))
        self.assertEqual(
            (self.root / "stdlib/module.jai").read_text(), "value :: 17;\n"
        )

    def test_outputs_inside_selected_inputs_reject_before_capture(self):
        for output, extras in [(self.root, ()), (self.root / "stdlib/output", ())]:
            with self.subTest(output=output), self.assertRaises(ValueError):
                shared_snapshot.capture(output, extras)
        tools = self.root / "tools"
        tools.mkdir()
        (tools / "source.py").write_text("value = 17\n")
        with self.assertRaises(ValueError):
            shared_snapshot.capture(tools / "output", ("tools",))

    def test_a_selected_ancestor_prevents_recursive_capture(self):
        with self.assertRaises(ValueError):
            shared_snapshot.capture(self.root / "artifacts/output", (".",))

    def test_executable_helpers_keep_execution_without_write_permissions(self):
        tools = self.root / "tools"
        tools.mkdir()
        helper = tools / "consumer.py"
        helper.write_text('print("capture helper ok")\n')
        helper.chmod(0o755)
        snapshot, receipt = shared_snapshot.capture(
            self.root / "artifacts/output", ("tools",)
        )
        copied = snapshot / "tools/consumer.py"
        expected = 0o444 | (helper.stat().st_mode & 0o111)
        self.assertEqual(copied.stat().st_mode & 0o777, expected)
        self.assertEqual(
            subprocess.check_output([sys.executable, str(copied)], text=True),
            "capture helper ok\n",
        )
        self.assertTrue(shared_snapshot.unchanged(snapshot, receipt))


if __name__ == "__main__":
    unittest.main()
