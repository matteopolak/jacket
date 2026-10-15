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

    def cache(self):
        configuration = {"source": "pinned", "sdk": "20348", "crt": "MT"}
        entry = self.path / mesa.build_cache_key(configuration)
        tree = entry / "llvm"
        tree.mkdir(parents=True)
        (tree / "library.lib").write_bytes(b"native library integrity fixture")
        mesa.seal_llvm_cache(entry, configuration)
        return entry, configuration

    def test_completed_cache_requires_exact_configuration(self):
        entry, configuration = self.cache()
        self.assertTrue(mesa.verify_llvm_cache(entry, configuration))
        with self.assertRaisesRegex(ValueError, "configuration"):
            mesa.verify_llvm_cache(entry, dict(configuration, crt="MD"))
        self.assertNotEqual(mesa.build_cache_key(configuration),
                            mesa.build_cache_key(dict(configuration, sdk="changed")))

    def test_cache_rejects_modified_missing_and_extra_files(self):
        entry, configuration = self.cache()
        library = entry / "llvm/library.lib"
        original = library.read_bytes()
        library.write_bytes(b"corruption")
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)
        library.unlink()
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)
        library.write_bytes(original)
        (entry / "llvm/unrecorded.dll").write_bytes(b"unexpected")
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)

    def test_partial_cache_does_not_count_as_complete(self):
        entry = self.path / "partial"
        (entry / "llvm").mkdir(parents=True)
        (entry / "llvm/unfinished.lib").write_bytes(b"partial")
        self.assertFalse(mesa.verify_llvm_cache(entry, {"source": "pinned"}))

    def test_native_cache_key_tracks_sdk_tools_and_flags(self):
        # Integrity/configuration fixtures only: no native binary is executed or
        # accepted as a driver by these tests.
        sdk = self.path / "public-sdk"
        vc = self.path / "public-toolchain"
        for directory in [sdk / "include", sdk / "lib", vc / "bin"]:
            directory.mkdir(parents=True)
        header = sdk / "include/native.h"
        header.write_bytes(b"native header fixture")
        library = sdk / "lib/native.lib"
        library.write_bytes(b"native SDK library fixture")
        tool = vc / "bin/tool.exe"
        tool.write_bytes(b"tool identity fixture")
        env = {"WINDOWSSDKDIR": str(sdk), "VCTOOLSINSTALLDIR": str(vc),
               "INCLUDE": str(sdk / "include"), "LIB": str(sdk / "lib"),
               "WINDOWSSDKVERSION": "10.0.20348.0", "VCTOOLSVERSION": "14.39"}
        with patch.object(mesa.shutil, "which", return_value=str(tool)):
            baseline = mesa.llvm_cache_configuration(env)
            key = mesa.build_cache_key(baseline)
            self.assertEqual(baseline["toolchain"]["VCToolsVersion"], "14.39")
            self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(dict(env, CL="/Od"))))
            self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(dict(env, WINDOWSSDKVERSION="changed"))))
            for path in [header, library, tool]:
                original = path.read_bytes()
                path.write_bytes(original + b"changed")
                self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(env)))
                path.write_bytes(original)
            (vc / "bin/frontend.dll").write_bytes(b"changed compiler backend")
            self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(env)))


if __name__ == "__main__":
    unittest.main()
