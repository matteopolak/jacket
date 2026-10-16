"""Portable installed OpenSSL discovery; fixtures and probes stay on assigned T7."""

import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from openssl_sdk import SDKError, discover_sdk, probe_sdk


class OpenSSLDiscovery(unittest.TestCase):
    def setUp(self):
        scratch = Path(os.environ["CARGO_TARGET_DIR"]) / "openssl-sdk-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "SDK with spaces"

    def sdk(self, libdir="lib", library="libcrypto.so"):
        header = self.root / "include/openssl/opensslv.h"
        header.parent.mkdir(parents=True)
        header.write_text("discovery-only fixture; never compiled")
        library_path = self.root / libdir / library
        library_path.parent.mkdir(parents=True)
        library_path.write_text("discovery-only fixture; never linked")
        return library_path

    def test_explicit_sdk_preserves_argument_boundaries(self):
        library = self.sdk()
        sdk = discover_sdk(self.root, environ={})
        self.assertEqual(sdk.prefix, self.root.resolve())
        self.assertEqual(sdk.library, library)
        self.assertIn("-I" + str(self.root / "include"), sdk.cflags)
        self.assertIn(str(library), sdk.ldflags)

    def test_missing_explicit_sdk_is_fatal_without_fallback(self):
        with patch("openssl_sdk._pkg_config") as metadata:
            with self.assertRaisesRegex(SDKError, "opensslv.h"):
                discover_sdk(self.root, environ={})
            metadata.assert_not_called()

    def test_environment_root_is_authoritative(self):
        self.sdk()
        sdk = discover_sdk(environ={"OPENSSL_ROOT_DIR": str(self.root)})
        self.assertEqual(sdk.prefix, self.root.resolve())
        with self.assertRaises(SDKError):
            discover_sdk(environ={"OPENSSL_DIR": str(self.root / "absent")})

    def test_explicit_argument_wins_over_environment(self):
        self.sdk()
        sdk = discover_sdk(self.root, environ={"OPENSSL_ROOT_DIR": "/missing"})
        self.assertEqual(sdk.prefix, self.root.resolve())

    def test_missing_development_library_is_fatal(self):
        header = self.root / "include/openssl/opensslv.h"
        header.parent.mkdir(parents=True)
        header.write_text("fixture")
        with self.assertRaisesRegex(SDKError, "libcrypto"):
            discover_sdk(self.root, environ={})

    def test_pkg_config_supports_linux_multiarch_and_filtered_flags(self):
        library = self.sdk("lib/x86_64-linux-gnu")
        values = {
            "--variable=prefix": str(self.root),
            "--variable=includedir": str(self.root / "include"),
            "--variable=libdir": str(library.parent),
            "--cflags": "",
            "--libs": "-lcrypto -pthread",
        }
        with patch("openssl_sdk._pkg_config", side_effect=lambda option, env: values[option]):
            sdk = discover_sdk(environ={})
        self.assertEqual(sdk.source, "pkg-config:libcrypto")
        self.assertEqual(sdk.library, library)
        self.assertIn(str(library), sdk.ldflags)
        self.assertIn("-pthread", sdk.ldflags)
        self.assertNotIn("-lcrypto", sdk.ldflags)

    def test_standard_prefix_fallback_is_not_homebrew_specific(self):
        library = self.sdk("lib64")
        with patch("openssl_sdk._pkg_config", return_value=None), patch("openssl_sdk._brew_prefix", return_value=None), patch("openssl_sdk._system_prefixes", return_value=[self.root]):
            sdk = discover_sdk(environ={})
        self.assertEqual(sdk.library, library)

    def test_windows_import_library_layout(self):
        library = self.sdk("lib/VC/x64/MD", "libcrypto.lib")
        with patch("openssl_sdk.platform.machine", return_value="AMD64"), patch("openssl_sdk.platform.system", return_value="Windows"):
            sdk = discover_sdk(self.root, environ={})
        self.assertEqual(sdk.library, library)

    def test_pkg_config_library_directory_is_authoritative(self):
        self.sdk()
        values = {
            "--variable=prefix": str(self.root),
            "--variable=includedir": str(self.root / "include"),
            "--variable=libdir": str(self.root / "missing-metadata-library"),
            "--cflags": "",
            "--libs": "-lcrypto",
        }
        with patch("openssl_sdk._pkg_config", side_effect=lambda option, env: values[option]):
            with self.assertRaises(SDKError):
                discover_sdk(environ={})

    def test_missing_all_sdk_candidates_is_an_error_not_a_skip(self):
        with patch("openssl_sdk._pkg_config", return_value=None), patch("openssl_sdk._brew_prefix", return_value=None), patch("openssl_sdk._system_prefixes", return_value=[]):
            with self.assertRaisesRegex(SDKError, "OpenSSL development SDK"):
                discover_sdk(environ={})

    def test_real_installed_sdk_compiles_links_and_runs(self):
        sdk = discover_sdk()
        receipt = probe_sdk(sdk, Path(self.temporary.name) / "real-probe")
        self.assertEqual(receipt["header_major"], receipt["runtime_major"])
        self.assertGreaterEqual(receipt["header_major"], 3)


if __name__ == "__main__":
    unittest.main()
