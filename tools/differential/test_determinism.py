"""Compiler-owned host data is normalized without hiding artifact structure."""
import unittest

from differential.determinism import TIMED_OUT, Report, strip_global_data


def data_global(symbol="compiler.global_data.g9", payload=r"\01\02\03\04", size=4, align=4):
    return f'@{symbol} = internal global <{{ [{size} x i8] }}> <{{ [{size} x i8] c"{payload}" }}>, align {align}\n'.encode()


class HostDataNormalizationTests(unittest.TestCase):
    def test_llvm_backslash_bytes_do_not_break_payload_matching(self):
        first = data_global(payload=r"0\\\CA\F2", size=4)
        second = data_global(payload=r"0\\\CAk", size=4)
        self.assertEqual(strip_global_data(first), strip_global_data(second))
        self.assertIn(b'c"<host-data>"', strip_global_data(first))

    def test_emitted_global_names_normalize_only_the_host_payload(self):
        first = data_global()
        second = data_global(payload=r"\05\06\07\08")
        self.assertEqual(strip_global_data(first), strip_global_data(second))
        self.assertIn(b"[4 x i8]", strip_global_data(first))
        self.assertIn(b"align 4", strip_global_data(first))

    def test_other_globals_and_similarly_named_symbols_are_checked(self):
        for symbol in ["user.g9", "compiler.global_data_extra.g9", "compiler.global_data.g9.other"]:
            first = data_global(symbol=symbol)
            second = data_global(symbol=symbol, payload=r"\05\06\07\08")
            self.assertNotEqual(strip_global_data(first), strip_global_data(second))
            self.assertEqual(strip_global_data(first), first)

    def test_layout_alignment_and_identity_are_not_normalized(self):
        reference = strip_global_data(data_global())
        for changed in [data_global(size=8), data_global(align=8), data_global(symbol="compiler.global_data.g10")]:
            self.assertNotEqual(reference, strip_global_data(changed))


class TimeoutTests(unittest.TestCase):
    OK = (0, b"ok", b"")

    def test_a_timeout_is_a_failure_reported_as_one(self):
        report = Report()
        report.compare("check a.jai", [("one", self.OK), ("two", TIMED_OUT), ("three", self.OK)])
        self.assertEqual(len(report.failures), 1)
        self.assertIn("timeout: `two`", report.failures[0])
        self.assertNotIn("differs", report.failures[0])
        self.assertEqual(report.timeouts, 1)

    def test_every_run_timing_out_is_too_slow_to_compare(self):
        report = Report()
        report.compare("check a.jai", [("one", TIMED_OUT), ("two", TIMED_OUT)])
        self.assertIn("too slow to compare", report.failures[0])

    def test_a_real_result_that_looks_like_the_old_marker_is_a_difference(self):
        report = Report()
        report.compare("check a.jai", [("one", self.OK), ("two", (-1, b"", b"timeout"))])
        self.assertIn("differs", report.failures[0])
        self.assertEqual(report.timeouts, 0)


if __name__ == "__main__":
    unittest.main()
