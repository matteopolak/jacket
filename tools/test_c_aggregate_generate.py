"""The c-aggregate-abi generator writes jaifmt-formatted files that match the committed fixtures."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from toolkit.repo import ROOT, jaifmt_path

NATIVE = ROOT / "tests/compiler/native"
GENERATOR = NATIVE / "c-aggregate-abi/generate.py"


@unittest.skipUnless(jaifmt_path().exists(), "needs the repository's own jaifmt")
class AggregateGenerator(unittest.TestCase):
    def test_output_is_formatted_and_committed(self):
        with tempfile.TemporaryDirectory() as scratch:
            subprocess.run(
                [sys.executable, str(GENERATOR), "--out", scratch], check=True
            )
            out = Path(scratch)
            names = sorted(
                p.relative_to(out) for p in out.rglob("*") if p.is_file()
            )
            self.assertIn(Path("c-aggregate-abi/shapes_0.jai"), names)
            self.assertIn(Path("c-variadic-aggregates/variadics.jai"), names)
            for name in names:
                self.assertEqual(
                    (out / name).read_text(),
                    (NATIVE / name).read_text(),
                    f"{name} differs from the committed file",
                )


if __name__ == "__main__":
    unittest.main()
