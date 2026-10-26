

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from generators.jai_codegen import DEFAULT_FORMATTER, emit_generated


@unittest.skipUnless(
    DEFAULT_FORMATTER.is_file(),
    f"{DEFAULT_FORMATTER} is missing: run python3 tools/run_tests.py (it builds jaifmt first) or `just fmt --check --lang jai`",
)
class GeneratedFormatting(unittest.TestCase):
    def test_check_detects_drift_without_writing_and_generation_is_idempotent(self):
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "generated.jai"
            source = "Item::struct{value:s64;}\n"
            path.write_text(source)
            with self.assertRaisesRegex(SystemExit, "stale generated source"):
                emit_generated(path, source, check=True)
            self.assertEqual(path.read_text(), source)
            emit_generated(path, source)
            formatted = path.read_text()
            self.assertNotEqual(formatted, source)
            emit_generated(path, source, check=True)
            emit_generated(path, source)
            self.assertEqual(path.read_text(), formatted)


if __name__ == "__main__":
    unittest.main()
