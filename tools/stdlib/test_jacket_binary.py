"""`jacket_binary` has one source (beside the compiler) and one override (`--jacket`)."""

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from stdlib.jacket_tests import jacket_binary
from toolkit.host import exe_name


class JacketBinary(unittest.TestCase):
    def test_the_jacket_beside_the_compiler(self):
        with tempfile.TemporaryDirectory() as tmp:
            beside = Path(tmp) / exe_name("jacket")
            beside.write_text("")
            self.assertEqual(jacket_binary(Path(tmp) / exe_name("jaic")).resolve(), beside.resolve())

    def test_the_path_is_never_searched(self):
        with tempfile.TemporaryDirectory() as tmp, tempfile.TemporaryDirectory() as elsewhere:
            installed = Path(elsewhere) / exe_name("jacket")
            installed.write_text("")
            installed.chmod(0o755)
            with patch.dict(os.environ, {"PATH": elsewhere}):
                with self.assertRaisesRegex(FileNotFoundError, "--jacket"):
                    jacket_binary(Path(tmp) / exe_name("jaic"))

    def test_an_explicit_jacket_wins_and_must_exist(self):
        with tempfile.TemporaryDirectory() as tmp:
            chosen = Path(tmp) / "mine"
            with self.assertRaisesRegex(FileNotFoundError, "mine"):
                jacket_binary(Path(tmp) / "jaic", chosen)
            chosen.write_text("")
            self.assertEqual(jacket_binary(Path(tmp) / "jaic", chosen), chosen)


if __name__ == "__main__":
    unittest.main()
