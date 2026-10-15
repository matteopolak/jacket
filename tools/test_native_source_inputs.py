"""Pinned native release transport tests; these do not claim native ABI execution."""
import hashlib
import importlib.util
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

NATIVE_TOOLS = Path(__file__).resolve().parent / "native_bindings"
sys.path.insert(0, str(NATIVE_TOOLS))
SPEC = importlib.util.spec_from_file_location("native_source_builder", NATIVE_TOOLS / "build_dependencies.py")
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)


class NativeReleaseInputs(unittest.TestCase):
    def test_callback_scoped_build_keeps_shared_allocator_dependency(self):
        self.assertEqual(builder.library_closure({'rpmalloc_callback_probe'}),
                         {'rpmalloc_callback_probe', 'binding_rpmalloc_sdk'})
        self.assertEqual(builder.library_closure({'binding_callback_context'}),
                         {'binding_callback_context'})
        self.assertEqual(builder.library_closure({'binding_rpmalloc_callbacks_nostats'}),
                         {'binding_rpmalloc_callbacks_nostats', 'binding_rpmalloc_sdk_nostats'})

    def fixture(self):
        header = b"/* original public C header fixture */\n#define NATIVE_WORD_BITS 64\n"
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode="w:xz") as stream:
            info = tarfile.TarInfo("native-1/include/native.h")
            info.size = len(header)
            stream.addfile(info, io.BytesIO(header))
            transitive = b'/* independently authored transitive header fixture */\n'
            info = tarfile.TarInfo("native-1/include/detail.h")
            info.size = len(transitive)
            stream.addfile(info, io.BytesIO(transitive))
        data = archive.getvalue()
        source = {"repository": "unavailable-mirror/native", "revision": "native-1",
                  "files": {"include/native.h": hashlib.sha256(header).hexdigest()},
                  "archive": {"url": "https://native.invalid/releases/native-1.tar.xz",
                              "sha256": hashlib.sha256(data).hexdigest(), "root": "native-1"}}
        return header, data, source

    def test_release_files_use_verified_archive_without_mirror_fallback(self):
        header, data, source = self.fixture()
        with tempfile.TemporaryDirectory() as temporary:
            vendor = Path(temporary)
            with patch.object(builder.urllib.request, "urlopen", return_value=io.BytesIO(data)) as fetch:
                builder.prepare_source("native", source, vendor)
            fetch.assert_called_once_with(source["archive"]["url"], timeout=60)
            self.assertEqual((vendor / "native/include/native.h").read_bytes(), header)
            with patch.object(builder.urllib.request, "urlopen", side_effect=AssertionError("unexpected network")):
                builder.prepare_source("native", source, vendor)

    def test_archive_corruption_is_rejected_before_extraction(self):
        _, _, source = self.fixture()
        with tempfile.TemporaryDirectory() as temporary:
            vendor = Path(temporary)
            (vendor / "native.tar.gz").write_bytes(b"corrupt archive")
            with self.assertRaisesRegex(RuntimeError, "Archive hash mismatch"):
                builder.prepare_source("native", source, vendor)
            self.assertFalse((vendor / "native-1").exists())

    def test_cached_header_corruption_is_rejected(self):
        _, data, source = self.fixture()
        with tempfile.TemporaryDirectory() as temporary:
            vendor = Path(temporary)
            with patch.object(builder.urllib.request, "urlopen", return_value=io.BytesIO(data)):
                builder.prepare_source("native", source, vendor)

            (vendor / "native/include/native.h").write_bytes(b"corrupt header")
            with self.assertRaisesRegex(RuntimeError, "Native source hash mismatch"):
                builder.prepare_source("native", source, vendor)

    def test_transitive_tree_is_recreated_from_verified_archive(self):
        _, data, source = self.fixture()
        with tempfile.TemporaryDirectory() as temporary:
            vendor = Path(temporary)
            with patch.object(builder.urllib.request, "urlopen", return_value=io.BytesIO(data)):
                builder.prepare_source("native", source, vendor)
            detail = vendor / "native-1/include/detail.h"
            expected = detail.read_bytes()
            detail.write_bytes(b'corrupt transitive header')
            extra = vendor / "native-1/include/injected.h"
            extra.write_bytes(b'not present in verified release')
            with patch.object(builder.urllib.request, "urlopen", side_effect=AssertionError("unexpected network")):
                builder.prepare_source("native", source, vendor)
            self.assertEqual(detail.read_bytes(), expected)
            self.assertFalse(extra.exists())


if __name__ == "__main__":
    unittest.main()
