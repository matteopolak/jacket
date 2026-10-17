"""Source/parser contracts; native package success is tested by the real SDK test."""

import ast
from pathlib import Path
import os
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from android_crypto_linux import (
    elf_dynamic,
    license_sources,
    public_header_paths,
    runtime_paths,
)


class LinuxClosure(unittest.TestCase):
    def test_real_soname_and_system_needed(self):
        self.assertEqual(
            elf_dynamic(
                "(SONAME) Library soname: [libcrypto.so.3]\n"
                "(NEEDED) Shared library: [libc.so.6]"
            ),
            ("libcrypto.so.3", ["libc.so.6"]),
        )

    def test_missing_ambiguous_or_nonbasename_soname_rejected(self):
        for text in (
            "",
            "(SONAME) [/tmp/libcrypto.so.3]",
            "(SONAME) [..]",
            "(SONAME) [libcrypto.so.3]\n(SONAME) [libcrypto.so.3]",
        ):
            with self.subTest(text=text), self.assertRaises(ValueError):
                elf_dynamic(text)

    def test_third_party_needed_rejected(self):
        with self.assertRaisesRegex(ValueError, "third-party"):
            elf_dynamic("(SONAME) [libcrypto.so.3]\n(NEEDED) [libunexpected.so.1]")

    def test_unresolved_or_unreported_runtime_rejected(self):
        for report in ("libc.so.6 => not found", ""):
            with self.subTest(report=report), self.assertRaises(ValueError):
                runtime_paths(report, ["libc.so.6"])

    def test_multiarch_header_dependencies_and_spaces(self):
        self.assertEqual(
            public_header_paths(
                "OPENSSL_HEADERS: probe.c /usr/include/openssl/crypto.h \\\n"
                " /usr/include/x86_64-linux-gnu/openssl/configuration.h "
                "/sdk\\ space/include/openssl/opensslv.h /usr/include/stdio.h"
            ),
            [
                Path("/sdk space/include/openssl/opensslv.h"),
                Path("/usr/include/openssl/crypto.h"),
                Path("/usr/include/x86_64-linux-gnu/openssl/configuration.h"),
            ],
        )

    def test_malformed_header_rule_rejected(self):
        with self.assertRaises(ValueError):
            public_header_paths("not a dependency rule")

    def test_every_package_requires_its_own_notice(self):
        target = (
            Path(os.environ["CARGO_TARGET_DIR"]) / "linux-openssl-package/unit-scratch"
        )
        target.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=target) as temporary:
            root = Path(temporary)
            archive = root / "libcrypto.a"
            archive.touch()
            sdk = SimpleNamespace(prefix=root)

            def metadata(command):
                if "--search" in command:
                    return "libssl-dev:amd64: " + str(archive) + "\n"
                return "/usr/share/doc/libssl-dev/absent-notice\n"

            with (
                patch(
                    "android_crypto_linux.shutil.which",
                    return_value="/usr/bin/dpkg-query",
                ),
                patch("android_crypto_linux.run", side_effect=metadata),
            ):
                with self.assertRaisesRegex(
                    ValueError, "no installed copyright notice"
                ):
                    license_sources(sdk, [archive])

    def test_unattributed_input_rejected(self):
        with (
            patch(
                "android_crypto_linux.shutil.which", return_value="/usr/bin/dpkg-query"
            ),
            patch("android_crypto_linux.run", return_value=""),
        ):
            with self.assertRaisesRegex(ValueError, "no installed package owner"):
                license_sources(
                    SimpleNamespace(prefix=Path("/absent-sdk")), [Path("/absent.a")]
                )

    def test_builder_dispatch_precedes_darwin_sdk_inputs(self):
        builder = Path(__file__).with_name("build_android_crypto.py")
        tree = ast.parse(builder.read_text())
        build = next(
            node
            for node in tree.body
            if isinstance(node, ast.FunctionDef) and node.name == "build"
        )
        self.assertIsInstance(build.body[1], ast.If)
        self.assertIn("Linux", ast.unparse(build.body[1].test))
        self.assertIn("build_linux", ast.unparse(build.body[1]))


if __name__ == "__main__":
    unittest.main()
