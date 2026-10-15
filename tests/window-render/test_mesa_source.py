"""Original source-cache integrity regressions; no driver execution is simulated."""
import hashlib
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import window_render_mesa_windows as mesa


class SourceIntegrity(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name)

    def archive(self, member):
        archive = self.path / "fixture.archive"
        with tarfile.open(archive, "w") as output:
            entry = tarfile.TarInfo(member)
            entry.size = 4
            output.addfile(entry, io.BytesIO(b"data"))
        return hashlib.sha256(archive.read_bytes()).hexdigest()

    def test_digest_failure_does_not_extract(self):
        self.archive("source/file.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", "0" * 64, "source")):
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                mesa.source_tree("fixture", self.path)
        self.assertFalse((self.path / "source").exists())

    def test_verified_source_rejects_traversal(self):
        digest = self.archive("../escaped.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", digest, "source")):
            with self.assertRaises(tarfile.OutsideDestinationError):
                mesa.source_tree("fixture", self.path)
        self.assertFalse((self.path.parent / "escaped.c").exists())

    def test_modified_source_is_not_reused(self):
        digest = self.archive("source/file.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", digest, "source")):
            tree = mesa.source_tree("fixture", self.path)
            (tree / "file.c").write_bytes(b"modified")
            with self.assertRaisesRegex(ValueError, "stale/modified"):
                mesa.source_tree("fixture", self.path)


if __name__ == "__main__":
    unittest.main()
