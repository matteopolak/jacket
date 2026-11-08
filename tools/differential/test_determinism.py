"""Compiler-owned host data is normalized without hiding artifact structure."""
import unittest

from differential.determinism import strip_global_data


def data_global(symbol="compiler.global_data.g9", payload=r"\01\02\03\04", size=4, align=4):
    return f'@{symbol} = internal global <{{ [{size} x i8] }}> <{{ [{size} x i8] c"{payload}" }}>, align {align}\n'.encode()


class HostDataNormalizationTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
