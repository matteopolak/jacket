"""Host-neutral SDL Python preparation contracts; no native compilation."""

import os
import hashlib
import tarfile
import json
from unittest.mock import patch
import sdl_test_support as support
import prepare_sdl_python_tests as prepare
from pathlib import Path
import unittest


class SdlPythonSetup(unittest.TestCase):
    def test_owned_tests_do_not_force_repository_artifact_scratch(self):
        root = Path(os.environ.get("SDL_SETUP_TEST_ROOT", support.repository_root()))
        for name in (
            "test_sdl3_coverage.py",
            "test_sdl3_gen.py",
            "test_sdl3_native.py",
        ):
            with self.subTest(name=name):
                self.assertNotIn(
                    "artifacts/sdl3/tmp", (root / "tools" / name).read_text()
                )

    def test_scratch_parent_is_created_only_under_explicit_root(self):
        with support.temporary_directory() as outer:
            root = Path(outer) / "new parent" / "nested"
            with patch.dict(os.environ, {"SDL_TEST_TMPDIR": str(root)}):
                with support.temporary_directory() as child:
                    self.assertEqual(Path(child).parent, root)
                    self.assertTrue(Path(child).is_dir())
            self.assertFalse(Path(child).exists())

    def test_unconfigured_or_relative_scratch_has_no_internal_fallback(self):
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(RuntimeError, "SDL_TEST_TMPDIR is required"):
                support.temporary_directory()
        with patch.dict(os.environ, {"SDL_TEST_TMPDIR": "relative"}):
            with self.assertRaises(ValueError):
                support.temporary_directory()

    def test_canonical_helpers_need_no_handoff_environment_keys(self):
        root = support.repository_root()
        with patch.dict(os.environ, {"SDL_PYTHON_TEST_ROOT": str(root)}, clear=True):
            self.assertEqual(
                support.source_path(
                    "SDL_FIXTURE_TEST_HELPER", "tools/native_bindings/fixture_paths.py"
                ),
                root / "tools/native_bindings/fixture_paths.py",
            )
            self.assertEqual(
                support.source_path(
                    "SDL_SDK_TEST_HELPER", "tools/native_bindings/sdl_sdk_headers.py"
                ),
                root / "tools/native_bindings/sdl_sdk_headers.py",
            )

    def test_explicit_missing_helper_override_is_not_silently_replaced(self):
        with support.temporary_directory() as temporary:
            with patch.dict(
                os.environ,
                {"SDL_FIXTURE_TEST_HELPER": str(Path(temporary) / "missing.py")},
            ):
                with self.assertRaises(FileNotFoundError):
                    support.source_path(
                        "SDL_FIXTURE_TEST_HELPER",
                        "tools/native_bindings/fixture_paths.py",
                    )

    def test_offline_missing_and_corrupt_cache_fail_without_download(self):
        with support.temporary_directory() as temporary:
            root = Path(temporary)
            pin = json.loads(
                (
                    support.repository_root()
                    / "tools/native_bindings/dependencies.json"
                ).read_text()
            )["sdl2-runtime-oracle"]["archive"]
            with patch.object(
                prepare.urllib.request,
                "urlopen",
                side_effect=AssertionError("offline network access"),
            ):
                with self.assertRaises(FileNotFoundError):
                    prepare.prepare(root / "scratch", root / "cache", offline=True)
                archive = (
                    root
                    / "cache"
                    / ("sdl2-runtime-oracle-" + pin["sha256"] + ".tar.gz")
                )
                archive.write_bytes(b"corrupt test cache; never a native artifact")
                with self.assertRaisesRegex(ValueError, "cache checksum mismatch"):
                    prepare.prepare(root / "scratch", root / "cache", offline=True)

    def test_metadata_cleanup_rejects_actual_unknown_fixture_files(self):
        with support.temporary_directory() as temporary:
            root = Path(temporary)
            (root / "subject.h").write_text("real independent fixture")
            sidecar = root / "._subject.h"
            sidecar.write_bytes(
                b"not AppleDouble; never suppress a native header failure"
            )
            with self.assertRaisesRegex(ValueError, "Unrecognized"):
                support.remove_fixture_metadata(root)
            self.assertTrue(sidecar.exists())
            sidecar.write_bytes(bytes.fromhex("0005160700020000"))
            support.remove_fixture_metadata(root)
            self.assertFalse(sidecar.exists())
            self.assertEqual(
                (root / "subject.h").read_text(), "real independent fixture"
            )

    def test_prepared_inputs_match_pinned_archive_member_bytes(self):
        # Main prerequisite has already verified/staged these public C inputs.
        source = support.source_path(
            "SDL_SDK_TEST_SOURCE", support.scratch_root() / "sdl-sdk-source"
        )
        receipt = json.loads(
            (support.scratch_root() / "sdl-sdk-test-inputs.json").read_text()
        )
        pin = json.loads(
            (
                support.repository_root() / "tools/native_bindings/dependencies.json"
            ).read_text()
        )["sdl2-runtime-oracle"]["archive"]
        self.assertEqual(receipt["sha256"], pin["sha256"])
        self.assertEqual(receipt["root"], pin["root"])
        archive = Path(receipt["archive"])
        self.assertEqual(
            hashlib.sha256(archive.read_bytes()).hexdigest(), pin["sha256"]
        )
        with tarfile.open(archive) as stream:
            for member in prepare.MEMBERS:
                self.assertEqual(
                    (source / member).read_bytes(),
                    stream.extractfile(pin["root"] + "/" + member).read(),
                )
        self.assertIn(
            b"#cmakedefine SDL2COMPAT_DISABLE_X11",
            (source / prepare.MEMBERS[1]).read_bytes(),
        )
        self.assertIn(
            b"project(sdl2_compat VERSION 2.32.70",
            (source / "CMakeLists.txt").read_bytes(),
        )
        self.assertTrue((source / "LICENSE.txt").read_bytes())


if __name__ == "__main__":
    unittest.main()
