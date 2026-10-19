import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import build_portable as bp
from toolkit.repo import ROOT

OUT = ROOT / "target" / "portable-tests"


class Fixtures(unittest.TestCase):
    def build(self, command, folder, names):
        self.assertEqual(bp.main([command]), 0)
        for name in names:
            self.assertTrue((OUT / folder / name).is_file(), f"{command} did not write {name}")
        return {p.name: p.read_bytes() for p in sorted((OUT / folder).iterdir())}

    def test_zip_fixtures_are_deterministic(self):
        names = ["valid.zip", "empty.zip", "truncated.zip", "zip64.zip"]
        self.assertEqual(self.build("zip", "zip", names), self.build("zip", "zip", names))

    def test_archive_fixtures(self):
        built = self.build("archive", "archive-fixtures", ["classic.a", "gnu.a", "coff.a", "bsd-big.a", "bad-size.a"])
        self.assertTrue(built["classic.a"].startswith(b"!<arch>\n"))

    def test_mpeg_fixtures(self):
        built = self.build("mpeg", "mpeg", ["reordered.m1v", "ac.m1v", "stereo.mp2", "joint-crc.mp2"])
        self.assertEqual(len(built["stereo.mp2"]), 2 * 417)

    def test_every_subcommand_is_documented_in_the_module_help(self):
        for command in ("zip", "archive", "mpeg", "machine", "meow", "clipboard", "runtime", "watcher"):
            self.assertIn(command, bp.__doc__)


if __name__ == "__main__":
    unittest.main()
