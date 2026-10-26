import io
import os
import sys
import tarfile
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

import install_pinned
from toolkit.hashing import sha256_bytes
from toolkit.repo import ROOT


def script_archive(entries):
    """A gzip tar of executable shell scripts: name -> body."""
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w:gz") as tar:
        for name, body in entries.items():
            data = body.encode()
            info = tarfile.TarInfo(name)
            info.size = len(data)
            info.mode = 0o755
            tar.addfile(info, io.BytesIO(data))
    return buffer.getvalue()


class Rust(unittest.TestCase):
    def channel(self):
        return install_pinned.rust_channel(ROOT)

    def install(self, components, targets):
        calls = []
        with mock.patch.object(
            install_pinned.subprocess,
            "run",
            side_effect=lambda command, **kwargs: calls.append(command),
        ):
            install_pinned.install_rust(ROOT, components, targets)
        return calls

    def test_the_channel_comes_from_rust_toolchain_toml(self):
        for line in (ROOT / "rust-toolchain.toml").read_text().splitlines():
            if line.startswith("channel"):
                self.assertEqual(self.channel(), line.split('"')[1])

    def test_components_and_targets_reach_rustup(self):
        channel = self.channel()
        self.assertEqual(
            self.install(["rustfmt"], ["wasm32-unknown-unknown"]),
            [
                [
                    "rustup",
                    "toolchain",
                    "install",
                    channel,
                    "--profile",
                    "minimal",
                    "--component",
                    "rustfmt",
                ],
                [
                    "rustup",
                    "target",
                    "add",
                    "wasm32-unknown-unknown",
                    "--toolchain",
                    channel,
                ],
            ],
        )

    def test_the_plain_install_names_only_the_channel(self):
        self.assertEqual(
            self.install([], []),
            [
                [
                    "rustup",
                    "toolchain",
                    "install",
                    self.channel(),
                    "--profile",
                    "minimal",
                ]
            ],
        )


@unittest.skipIf(sys.platform == "win32", "stand-in tools are shell scripts")
class Prebuilt(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dest = Path(self.tmp.name) / "bin"
        self.archive = script_archive(
            {"cargo-nextest": '#!/bin/sh\necho "cargo-nextest $*"\n'}
        )
        self.manifest = dict(install_pinned.pins.load(), nextest="1.2.3")
        for _, key in install_pinned.PREBUILT["nextest"].hosts.values():
            self.manifest[key] = sha256_bytes(self.archive)

    def install(self, manifest=None):
        urls = []

        def fake_fetch(url, **kwargs):
            urls.append(url)
            return self.archive

        with (
            mock.patch.object(install_pinned, "fetch", fake_fetch),
            mock.patch.object(install_pinned, "host_key", return_value="linux-arm64"),
            redirect_stdout(io.StringIO()),
        ):
            install_pinned.install_prebuilt(
                "nextest", manifest or self.manifest, self.dest
            )
        return urls

    def test_a_verified_archive_is_unpacked_and_run(self):
        self.assertEqual(self.install(), ["https://get.nexte.st/1.2.3/linux-arm"])
        self.assertTrue(os.access(self.dest / "cargo-nextest", os.X_OK))

    def test_a_checksum_mismatch_installs_nothing(self):
        bad = dict(self.manifest, nextest_sha256_linux_arm64="0" * 64)
        with self.assertRaises(SystemExit) as caught:
            self.install(bad)
        self.assertIn(
            "does not match nextest_sha256_linux_arm64", str(caught.exception)
        )
        self.assertFalse((self.dest / "cargo-nextest").exists())

    def test_a_platform_without_a_pin_names_the_cargo_install_fallback(self):
        with mock.patch.object(install_pinned, "host_key", return_value="macos"):
            with self.assertRaises(SystemExit) as caught:
                install_pinned.install_prebuilt(
                    "cargo-mutants", self.manifest, self.dest
                )
        self.assertIn("cargo install --locked cargo-mutants@", str(caught.exception))

    def test_every_pin_the_table_names_is_in_the_manifest(self):
        manifest = install_pinned.pins.load()
        for tool in install_pinned.PREBUILT.values():
            self.assertIn(tool.version_pin, manifest)
            for _, key in tool.hosts.values():
                self.assertIn(key, manifest)


@unittest.skipIf(sys.platform == "win32", "symbolic links need privileges there")
class Extract(unittest.TestCase):
    def test_strip_drops_the_leading_directory_and_keeps_links(self):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:xz") as tar:
            for name, body in (("top/bin/tool", b"x"), ("top/lib/libx.so.1", b"y")):
                info = tarfile.TarInfo(name)
                info.size = len(body)
                tar.addfile(info, io.BytesIO(body))
            link = tarfile.TarInfo("top/lib/libx.so")
            link.type = tarfile.SYMTYPE
            link.linkname = "libx.so.1"
            tar.addfile(link)
            hard = tarfile.TarInfo("top/bin/alias")
            hard.type = tarfile.LNKTYPE
            hard.linkname = "top/bin/tool"
            tar.addfile(hard)
        buffer.seek(0)
        with (
            tempfile.TemporaryDirectory() as tmp,
            tarfile.open(fileobj=buffer, mode="r:xz") as tar,
        ):
            install_pinned.extract(tar, Path(tmp), strip=1)
            self.assertEqual((Path(tmp) / "bin/tool").read_bytes(), b"x")
            self.assertEqual((Path(tmp) / "bin/alias").read_bytes(), b"x")
            self.assertEqual(os.readlink(Path(tmp) / "lib/libx.so"), "libx.so.1")


@unittest.skipIf(sys.platform == "win32", "the stand-in llvm-config is a shell script")
class LlvmRelease(unittest.TestCase):
    def test_unpacks_into_runner_temp_and_exports_the_prefix(self):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:xz") as tar:
            body = b'#!/bin/sh\necho "23.1.2 $*"\n'
            info = tarfile.TarInfo("LLVM-23.1.2-Linux-X64/bin/llvm-config")
            info.size = len(body)
            info.mode = 0o755
            tar.addfile(info, io.BytesIO(body))
        with tempfile.TemporaryDirectory() as tmp:
            temp = Path(tmp)
            env_file = temp / "github_env"
            downloads = []

            def fake_download(url, path, **kwargs):
                downloads.append(url)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(buffer.getvalue())

            environment = {"RUNNER_TEMP": str(temp), "GITHUB_ENV": str(env_file)}
            with (
                mock.patch.dict(os.environ, environment),
                mock.patch.object(install_pinned, "download", fake_download),
                redirect_stdout(io.StringIO()),
            ):
                for _ in range(2):
                    install_pinned.install_llvm_release(
                        {"llvm": "23.1.2"}, "LLVM-23.1.2-Linux-X64.tar.xz"
                    )
            self.assertEqual(
                downloads,
                [
                    "https://github.com/llvm/llvm-project/releases/download/llvmorg-23.1.2/LLVM-23.1.2-Linux-X64.tar.xz"
                ],
                "the second run reuses the downloaded archive",
            )
            self.assertTrue((temp / "llvm/bin/llvm-config").is_file())
            self.assertIn(
                f"LLVM_SYS_231_PREFIX={(temp / 'llvm').as_posix()}\n",
                env_file.read_text(),
            )


if __name__ == "__main__":
    unittest.main()
