import importlib
import os
import tempfile
import unittest


import build_portable as bp
from repo_output import output_dir


class Fixtures(unittest.TestCase):
    # Builds into a private output directory: other tests and tools write target/portable-tests
    # at the same time, and a shared directory made the determinism check flaky.
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.saved = os.environ.get("JACKET_OUTPUT_DIR")
        os.environ["JACKET_OUTPUT_DIR"] = cls.tmp.name
        importlib.reload(bp)

    @classmethod
    def tearDownClass(cls):
        if cls.saved is None:
            os.environ.pop("JACKET_OUTPUT_DIR", None)
        else:
            os.environ["JACKET_OUTPUT_DIR"] = cls.saved
        importlib.reload(bp)
        cls.tmp.cleanup()

    def build(self, command, folder, names):
        OUT = output_dir("portable-tests")
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
        for command in ("zip", "archive", "mpeg", "machine", "data", "meow", "clipboard", "runtime", "watcher"):
            self.assertIn(command, bp.__doc__)


if __name__ == "__main__":
    unittest.main()
