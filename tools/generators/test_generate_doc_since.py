import tempfile
import unittest
from pathlib import Path

from generators import generate_doc_since as since


class DocSinceTests(unittest.TestCase):
    def test_render_and_read_round_trip(self):
        seen = {("String", "split"): "-", ("String", "join"): "0.2.0", ("Basic", "log"): "0.2.0"}
        text = since.render(["v0.1.0", "v0.2.0"], seen)
        with tempfile.TemporaryDirectory() as scratch:
            path = Path(scratch) / "since.tsv"
            path.write_text(text, encoding="utf-8")
            covered, back = since.read(path)
        self.assertEqual(covered, ["v0.1.0", "v0.2.0"])
        self.assertEqual(back, seen)

    def test_a_missing_file_covers_nothing(self):
        self.assertEqual(since.read(Path("/nonexistent/since.tsv")), ([], {}))

    def test_the_checked_in_file_is_well_formed(self):
        covered, seen = since.read(since.TARGET)
        self.assertTrue(covered and seen)
        self.assertEqual(covered, sorted(covered, key=lambda t: tuple(map(int, t[1:].split(".")))))


if __name__ == "__main__":
    unittest.main()
