"""Strict source inventory diagnostics; no SDK or font substitution."""

from pathlib import Path
import tempfile
import unittest

from verify_font_selector_inputs import source_inventory


class Inventory(unittest.TestCase):
    def test_invalid_source_is_reported_without_lossy_decode(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "module.jai").write_text("value :: 17;")
            (root / "invalid.jai").write_bytes(b"\xff")
            report = source_inventory(root)
            self.assertEqual(report["jai_files"], 2 + len(report["sidecars"]))
            failures = report["strict_utf8_failures"]
            ordinary = [row for row in failures if row["path"] == "invalid.jai"]
            self.assertEqual(len(ordinary), 1)
            self.assertEqual(ordinary[0]["header"], "ff")
            # Real T7 transport may create binary metadata for each normal file.
            # Every extra decoding failure must still be exposed and identified.
            for row in failures:
                if row["path"] != "invalid.jai":
                    self.assertTrue(row["path"].startswith("._"))
                    self.assertTrue(row["header"].startswith("0005160700020000"))

    def test_sidecar_prefix_does_not_hide_valid_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "._legitimate.jai").write_text("value :: 17;")
            report = source_inventory(root)
            self.assertEqual(report["jai_files"], 1)
            self.assertEqual(len(report["sidecars"]), 1)
            self.assertEqual(report["strict_utf8_failures"], [])

    def test_unfiltered_inventory_reports_binary_sidecar_cause(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "nested"
            nested.mkdir()
            data = bytes.fromhex("0005160700020000") + bytes(16) + bytes.fromhex("0001")
            data += bytes.fromhex("000000020000002600000001ff")
            (nested / "._module.jai").write_bytes(data)
            report = source_inventory(root)
            self.assertEqual(report["sidecars"][0]["path"], "nested/._module.jai")
            self.assertEqual(len(report["strict_utf8_failures"]), 1)

    def test_non_source_binary_is_outside_jai_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "binary.dat").write_bytes(b"\xff")
            self.assertEqual(source_inventory(root)["jai_files"], 0)


if __name__ == "__main__":
    unittest.main()
