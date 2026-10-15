import hashlib
from pathlib import Path
import tempfile
import unittest

from stage_application_corpus import exclusion, stage


class ApplicationStaging(unittest.TestCase):
    def setUp(self):
        owned = Path(__file__).resolve().parents[1] / "artifacts/test-tmp"
        owned.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=owned)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.destination = self.root / "destination"
        self.folder = self.source / "focus-editor--focus"
        self.folder.mkdir(parents=True)

    def manifest(self, path, data=b"main :: () {}\n"):
        return {"projects": [{"repository": "focus-editor/focus", "revision": "original-pin",
                              "files": [{"path": path, "bytes": len(data),
                                         "sha256": hashlib.sha256(data).hexdigest()}]}]}

    def test_exact_application_bytes_and_hash_preserved(self):
        data = b"// original\nmain :: () {}\n"
        (self.folder / "main.jai").write_bytes(data)
        report = stage(self.manifest("main.jai", data), self.source, self.destination)
        self.assertEqual(report["staged"], 1)
        self.assertEqual((self.destination / "focus-editor--focus/main.jai").read_bytes(), data)

    def test_excluded_missing_vendor_path_never_accessed(self):
        # There is deliberately no modules directory or source file to open.
        report = stage(self.manifest("modules/Input/module.jai"), self.source, self.destination)
        self.assertEqual(report["excluded"], 1)
        self.assertFalse(self.destination.exists())

    def test_manifest_hash_mismatch_rejected(self):
        (self.folder / "main.jai").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "manifest mismatch"):
            stage(self.manifest("main.jai"), self.source, self.destination)

    def test_symlink_application_file_rejected(self):
        original = self.root / "other.jai"
        original.write_bytes(b"main :: () {}\n")
        (self.folder / "main.jai").symlink_to(original)
        with self.assertRaisesRegex(ValueError, "symlink source file"):
            stage(self.manifest("main.jai"), self.source, self.destination)

    def test_symlink_parent_rejected(self):
        original = self.root / "other"
        original.mkdir()
        (original / "main.jai").write_bytes(b"main :: () {}\n")
        (self.folder / "src").symlink_to(original, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink source path"):
            stage(self.manifest("src/main.jai"), self.source, self.destination)

    def test_traversal_absolute_and_case_variants(self):
        for path in ("../outside.jai", "/outside.jai", "src\\outside.jai"):
            with self.assertRaises(ValueError):
                exclusion(path)
        for path in ("MODULES/Basic/main.jai", "stdlib/Basic.jai", "src/module.jai"):
            self.assertIsNotNone(exclusion(path))


if __name__ == "__main__":
    unittest.main()
