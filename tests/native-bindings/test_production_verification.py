"""Production verification must never overlay peer sources."""
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools/native_bindings"))
from verify import select_stdlib
from verify_rpmalloc_heaps import run_check


class ProductionLibraryTests(unittest.TestCase):
    def test_missing_check_executable_is_retained_as_a_failure(self):
        with TemporaryDirectory() as directory:
            row = run_check([Path(directory) / "missing-program"], {})
        self.assertIsNone(row["exit"])
        self.assertIn("Could not execute check", row["stderr"])
        self.assertIn("missing-program", row["command"][0])

    def test_explicit_production_tree_bypasses_staging_and_preserves_sources(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / "production"
            (production / "prelude").mkdir(parents=True)
            source = production / "prelude/module.jai"
            source.write_text("canonical :: 17;\n")
            with patch("verify.stage", side_effect=AssertionError("production verification staged peers")):
                selected = select_stdlib(root / "work", production=production)
            self.assertEqual(selected, production.resolve())
            self.assertEqual(source.read_text(), "canonical :: 17;\n")
            self.assertFalse((root / "work").exists())

    def test_production_and_overlay_cannot_be_combined(self):
        with self.assertRaises(ValueError):
            select_stdlib(Path("unused"), foundation=Path("peer"), production=Path("production"))


if __name__ == "__main__":
    unittest.main()
