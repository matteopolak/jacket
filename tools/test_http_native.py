"""HTTP setup/dispatch policy tests; mocked SDK builds are not native evidence."""

import json
import hashlib
import io
import os
from pathlib import Path
import subprocess
import tempfile
import tarfile
import unittest
from unittest.mock import patch

import http_native
import test_http_conformance_coverage as coverage_tests


class HttpNativeSetupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        environment = patch.dict(
            os.environ, {"CARGO_TARGET_DIR": str(self.root)}, clear=False
        )
        environment.start()
        self.addCleanup(environment.stop)
        os.environ.pop("JAIC_NATIVE_LIBS", None)
        self.platform = patch.object(
            http_native.native, "host_dir", return_value="linux-x64"
        )
        self.platform.start()
        self.addCleanup(self.platform.stop)

    def populate(self, directory):
        directory.mkdir(parents=True, exist_ok=True)
        for name in http_native.required_files("linux-x64"):
            file = directory / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(b"original unit-test artifact placeholder")

    def test_profile_reuses_full_pinned_mbedtls_contract(self):
        manifest = json.loads(http_native.native.MANIFEST.read_text())
        profile = http_native.native_profile()
        self.assertEqual(set(profile["libraries"]), {"mbedtls"})
        self.assertEqual(
            profile["libraries"]["mbedtls"], manifest["libraries"]["mbedtls"]
        )
        self.assertEqual(profile["sources"]["mbedtls"], manifest["sources"]["mbedtls"])
        self.assertEqual(
            set(http_native.required_files("linux-x64")),
            {"libmbedtls.a", "libmbedtls.so", "licenses/mbedtls/LICENSE"},
        )

    def test_builds_real_builder_profile_before_exposing_provider(self):
        destination = self.root / "native"
        with (
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ) as fetch,
            patch.object(
                http_native.native,
                "build",
                side_effect=lambda *args: self.populate(destination),
            ) as build,
            patch.object(http_native.native, "copy_licenses") as licenses,
        ):
            result = http_native.ensure_native_libraries(destination)
        self.assertEqual(result, destination.resolve())
        self.assertEqual(os.environ["JAIC_NATIVE_LIBS"], str(destination.resolve()))
        self.assertEqual(build.call_args.args[0], "mbedtls")
        self.assertEqual(build.call_args.args[-1], "linux-x64")
        fetch.assert_called_once()
        licenses.assert_called_once()

    def test_complete_explicit_provider_is_preserved(self):
        provider = self.root / "provided"
        self.populate(provider)
        os.environ["JAIC_NATIVE_LIBS"] = str(provider)
        with patch.object(http_native.native, "fetch") as fetch:
            self.assertEqual(http_native.ensure_native_libraries(), provider.resolve())
        fetch.assert_not_called()
        self.assertEqual(os.environ["JAIC_NATIVE_LIBS"], str(provider.resolve()))

    def test_complete_provider_precedes_partial_directory(self):
        partial = self.root / "partial"
        partial.mkdir()
        (partial / "libmbedtls.a").write_bytes(b"original unit test input")
        complete = self.root / "complete"
        self.populate(complete)
        os.environ["JAIC_NATIVE_LIBS"] = os.pathsep.join(map(str, [partial, complete]))
        with patch.object(http_native.native, "fetch") as fetch:
            self.assertEqual(http_native.ensure_native_libraries(), complete.resolve())
        fetch.assert_not_called()
        self.assertEqual(
            os.environ["JAIC_NATIVE_LIBS"].split(os.pathsep),
            [str(complete.resolve()), str(partial.resolve())],
        )

    def test_archive_only_provider_does_not_claim_interpreter_support(self):
        partial = self.root / "partial"
        partial.mkdir()
        (partial / "libmbedtls.a").write_bytes(b"original unit test input")
        os.environ["JAIC_NATIVE_LIBS"] = str(partial)
        destination = self.root / "native"
        with (
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ),
            patch.object(
                http_native.native,
                "build",
                side_effect=lambda *args: self.populate(destination),
            ),
            patch.object(http_native.native, "copy_licenses"),
        ):
            http_native.ensure_native_libraries(destination)
        self.assertEqual(
            os.environ["JAIC_NATIVE_LIBS"].split(os.pathsep),
            [str(destination.resolve()), str(partial.resolve())],
        )

    def test_build_failure_remains_fatal_and_does_not_publish_provider(self):
        with (
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ),
            patch.object(
                http_native.native,
                "build",
                side_effect=subprocess.CalledProcessError(1, ["cc"]),
            ),
            patch.object(http_native.native, "copy_licenses"),
        ):
            with self.assertRaises(subprocess.CalledProcessError):
                http_native.ensure_native_libraries(self.root / "native")
        self.assertNotIn("JAIC_NATIVE_LIBS", os.environ)

    def test_conformance_sets_parent_provider_before_every_child(self):
        import http_conformance

        provider = self.root / "provided"
        observed = []

        def setup():
            os.environ["JAIC_NATIVE_LIBS"] = str(provider)

        def child(*args):
            observed.append(os.environ.get("JAIC_NATIVE_LIBS"))
            return True

        with (
            patch.object(
                http_conformance, "ensure_native_libraries", side_effect=setup
            ),
            patch.object(
                http_conformance, "find_jaic", return_value=Path("own-compiler")
            ),
            patch.object(http_conformance, "check_corpus", return_value=True),
            patch.object(http_conformance, "run_harness", side_effect=child) as harness,
            patch.object(http_conformance, "check_server", side_effect=child),
            patch.object(http_conformance, "llhttp_summary", side_effect=child),
            patch.object(http_conformance, "external_suites", return_value=True),
            patch("builtins.print"),
            patch(
                "sys.argv",
                [
                    "http_conformance.py",
                    "--modes",
                    "interp,native,wasm-interp,wasm-native",
                ],
            ),
        ):
            with self.assertRaises(SystemExit) as finished:
                http_conformance.main()
        self.assertEqual(finished.exception.code, 0)
        self.assertEqual(observed, [str(provider)] * 3)
        self.assertEqual(
            harness.call_args.args[1], "interp,native,wasm-interp,wasm-native"
        )
        self.assertEqual(
            harness.call_args.args[2], http_conformance.HTTP_CONFORMANCE_CASE_IDS
        )

    def test_success_status_without_shared_artifact_remains_failure(self):
        with (
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ),
            patch.object(http_native.native, "build"),
            patch.object(http_native.native, "copy_licenses"),
        ):
            with self.assertRaisesRegex(RuntimeError, "missing"):
                http_native.ensure_native_libraries(self.root / "native")
        self.assertNotIn("JAIC_NATIVE_LIBS", os.environ)

    def test_changed_owned_artifact_is_rebuilt(self):
        destination = self.root / "native"
        with (
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ),
            patch.object(
                http_native.native,
                "build",
                side_effect=lambda *args: self.populate(destination),
            ) as build,
            patch.object(http_native.native, "copy_licenses"),
        ):
            http_native.ensure_native_libraries(destination)
            (destination / "libmbedtls.so").write_bytes(b"changed artifact")
            http_native.ensure_native_libraries(destination)
        self.assertEqual(build.call_count, 2)

    def test_changed_profile_is_rebuilt(self):
        destination = self.root / "native"
        profile = http_native.native_profile()
        with (
            patch.object(http_native, "native_profile", return_value=profile),
            patch.object(
                http_native.native, "fetch", return_value=self.root / "source"
            ),
            patch.object(
                http_native.native,
                "build",
                side_effect=lambda *args: self.populate(destination),
            ) as build,
            patch.object(http_native.native, "copy_licenses"),
        ):
            http_native.ensure_native_libraries(destination)
            profile["libraries"]["mbedtls"]["defines"].append(
                "ORIGINAL_TEST_CONFIGURATION=1"
            )
            http_native.ensure_native_libraries(destination)
        self.assertEqual(build.call_count, 2)


class NativeArchiveTransportTests(unittest.TestCase):
    def test_cache_cleanup_failure_cannot_mark_nested_old_source_current(self):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:gz") as archive:
            entry = tarfile.TarInfo("sdk/new.h")
            entry.size = 1
            archive.addfile(entry, io.BytesIO(b"x"))
        data = stream.getvalue()
        source = {
            "revision": "sdk-test",
            "archive": {
                "url": "https://sdk.invalid/source.tar.gz",
                "root": "sdk",
                "sha256": hashlib.sha256(data).hexdigest(),
            },
        }
        with tempfile.TemporaryDirectory() as scratch:
            cache = Path(scratch) / "cache"
            base = cache / source["revision"]
            base.mkdir(parents=True)
            (base / "old.h").write_bytes(b"original old source")
            original = http_native.native.shutil.rmtree

            def refuse_base(path, ignore_errors=False, **kwargs):
                if Path(path) == base:
                    if not ignore_errors:
                        raise PermissionError("original denied-cleanup regression")
                    return
                return original(path, ignore_errors=ignore_errors, **kwargs)

            with (
                patch.object(
                    http_native.native.urllib.request,
                    "urlopen",
                    return_value=io.BytesIO(data),
                ),
                patch.object(
                    http_native.native.shutil, "rmtree", side_effect=refuse_base
                ),
            ):
                with self.assertRaises(PermissionError):
                    http_native.native.fetch(source, cache)
            self.assertEqual((base / "old.h").read_bytes(), b"original old source")
            self.assertFalse((base / ".sha256").exists())

    def test_verified_archive_root_is_preserved_as_a_directory(self):
        stream = io.BytesIO()
        files = {
            "sdk/.attributes": b"original hidden source file\n",
            "sdk/include/api.h": b"original SDK declaration fixture\n",
        }
        with tarfile.open(fileobj=stream, mode="w:gz") as archive:
            for name, data in files.items():
                entry = tarfile.TarInfo(name)
                entry.size = len(data)
                archive.addfile(entry, io.BytesIO(data))
        data = stream.getvalue()
        source = {
            "revision": "independent-test-sdk",
            "archive": {
                "url": "https://sdk.invalid/source.tar.gz",
                "root": "sdk",
                "sha256": hashlib.sha256(data).hexdigest(),
            },
        }
        with tempfile.TemporaryDirectory() as scratch:
            with (
                patch.object(
                    http_native.native.urllib.request,
                    "urlopen",
                    return_value=io.BytesIO(data),
                ),
                patch.object(
                    http_native.native.shutil,
                    "move",
                    wraps=http_native.native.shutil.move,
                ) as move,
            ):
                result = http_native.native.fetch(source, Path(scratch) / "cache")
            self.assertEqual(
                (result / ".attributes").read_bytes(), files["sdk/.attributes"]
            )
            self.assertEqual(
                (result / "include/api.h").read_bytes(), files["sdk/include/api.h"]
            )
            self.assertEqual(
                (result / ".sha256").read_text(), source["archive"]["sha256"]
            )
            # Directory-level relocation avoids splitting metadata companions
            # from their files while the extraction directory is enumerated.
            self.assertEqual(move.call_count, 1)
            self.assertEqual(Path(move.call_args.args[0]).name, "sdk")


def load_tests(loader, tests, pattern):
    # The existing normal-CI setup entry point also enforces case coverage.
    tests.addTests(loader.loadTestsFromModule(coverage_tests))
    return tests


if __name__ == "__main__":
    unittest.main()
