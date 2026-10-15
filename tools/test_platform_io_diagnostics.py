"""Protocol-parser tests only; these synthetic bytes are not native execution evidence."""
import subprocess
import unittest

from check_platform_io import fault_failures


class FaultDiagnosticTests(unittest.TestCase):
    def result(self, text=b"", status=0xc0000005, stdout=b""):
        return subprocess.CompletedProcess(["original-fixture.exe"], status, stdout, text)

    def test_status_and_output_fail_independently(self):
        failed = fault_failures(self.result(b"unexpected", 0, b"unexpected"), False)
        self.assertEqual(failed, ["exception_status", "unexpected_stdout", "disabled_handler_stderr"])
        self.assertEqual(fault_failures(self.result(), False), [])

    def test_single_frame_remains_failure(self):
        text = b"Windows exception 0x00000000c0000005\nnative stack:\n  0x0000000000000001\n"
        self.assertEqual(fault_failures(self.result(text), True), ["frame_count=1"])

    def test_named_frames_are_required_when_requested(self):
        text = (b"Windows exception 0x00000000c0000005\nnative stack:\n"
                b"  0x0000000000000001 platform_io_fault_leaf + 0x0000000000000001\n"
                b"  0x0000000000000002 platform_io_fault_middle + 0x0000000000000001\n")
        self.assertEqual(fault_failures(self.result(text), True,
                         ("platform_io_fault_leaf", "platform_io_fault_middle")), [])
        self.assertIn("missing_symbol=absent", fault_failures(self.result(text), True, ("absent",)))

    def test_failure_notice_cannot_be_hidden_by_frames(self):
        text = (b"Windows exception 0x00000000c0000005\nnative stack:\n"
                b"  0x0000000000000001\n  0x0000000000000002\nnative unwind failed\n")
        self.assertIn("unwind failed", fault_failures(self.result(text), True))
        self.assertIn("stack truncated", fault_failures(self.result(text.replace(
            b"native unwind failed", b"native stack truncated at 64 frames")), True))


if __name__ == "__main__":
    unittest.main()
