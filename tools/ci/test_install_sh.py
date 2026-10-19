"""install.sh against stub toolchains in archives laid out like a release's, served through JAI_BASE_URL=file://.

Runs on macOS and Linux, with or without a published release. The release-dependent checks (a real
download, a pipe install, the installed tools) stay in the `install-sh` job of ci.yml.
"""

import json
import os
import platform
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT
from toolkit.hashing import sha256_bytes

VECTORS = json.loads((ROOT / "tests" / "installers" / "vectors.json").read_text())

PLATFORMS = {
    ("Darwin", "arm64"): "macos-arm64",
    ("Darwin", "x86_64"): "macos-x64",
    ("Linux", "x86_64"): "linux-x64",
    ("Linux", "aarch64"): "linux-arm64",
    ("Linux", "arm64"): "linux-arm64",
}
PLATFORM = PLATFORMS.get((platform.system(), platform.machine()))
TOOLS = ("jaic", "jailsp", "jailint", "jaifmt")


@unittest.skipIf(PLATFORM is None, "install.sh has no build for this platform")
class InstallSh(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(lambda: subprocess.run(["rm", "-rf", str(self.tmp)]))
        self.release = self.tmp / "release"
        self.share = self.tmp / "share"
        self.bin = self.tmp / "bin"
        self.build("9.0.1")
        self.build("9.0.2")

    def build(self, version):
        stage = self.tmp / f"stage-{version}" / f"jacket-{PLATFORM}"
        (stage / "stdlib" / "prelude").mkdir(parents=True)
        (stage / "stdlib" / "prelude" / "module.jai").touch()
        for tool in TOOLS:
            script = stage / tool
            script.write_text(f'#!/bin/sh\necho "{tool} {version}"\n')
            script.chmod(0o755)
        directory = self.release / f"v{version}"
        directory.mkdir(parents=True)
        archive = directory / f"jacket-{PLATFORM}.tar.gz"
        with tarfile.open(archive, "w:gz") as tar:
            tar.add(stage, arcname=f"jacket-{PLATFORM}")
        digest = sha256_bytes(archive.read_bytes())
        (directory / "SHA256SUMS").write_text(f"{digest}  {archive.name}\n")

    def install(self, version=None, bin_dir=None, expect_ok=True):
        env = {
            k: v for k, v in os.environ.items() if not k.startswith(("JAI_", "JAIC_"))
        }
        env.update(
            JAI_BASE_URL=f"file://{self.release}",
            JAI_INSTALL_DIR=str(self.share),
            JAI_BIN_DIR=str(bin_dir or self.bin),
        )
        if version:
            env["JAI_VERSION"] = version
        result = subprocess.run(
            ["sh", str(ROOT / "install.sh")], env=env, capture_output=True, text=True
        )
        if expect_ok:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def run_tool(self, tool):
        return subprocess.run(
            [str(self.bin / tool)], capture_output=True, text=True
        ).stdout.strip()

    def test_install_then_upgrade_replaces_the_old_version(self):
        self.install("9.0.1")
        self.assertEqual(self.run_tool("jaic"), "jaic 9.0.1")
        self.install("9.0.2")
        self.assertEqual(self.run_tool("jaifmt"), "jaifmt 9.0.2")
        self.assertTrue((self.share / "9.0.2").is_dir())
        self.assertFalse((self.share / "9.0.1").exists())
        self.assertEqual(
            os.readlink(self.bin / "jaic"), str(self.share / "9.0.2" / "jaic")
        )

    def test_a_damaged_download_is_refused(self):
        archive = self.release / "v9.0.2" / f"jacket-{PLATFORM}.tar.gz"
        with archive.open("ab") as handle:
            handle.write(b"damaged\n")
        result = self.install("9.0.2", expect_ok=False)
        self.assertIn("checksum mismatch", result.stderr)
        self.assertFalse((self.share / "9.0.2").exists())

    def test_a_command_that_is_not_our_symlink_is_never_replaced(self):
        occupied = self.tmp / "occupied"
        occupied.mkdir()
        (occupied / "jaic").touch()
        result = self.install("9.0.1", bin_dir=occupied, expect_ok=False)
        self.assertIn("is not a symlink", result.stderr)

    def test_an_override_needs_an_explicit_version(self):
        result = self.install(expect_ok=False)
        self.assertIn("JAI_BASE_URL needs JAI_VERSION", result.stderr)

    def test_a_missing_version_says_it_could_not_download(self):
        result = self.install("0.0.0-missing", expect_ok=False)
        self.assertIn("could not download", result.stderr)


class SharedVectors(unittest.TestCase):
    """tests/installers/vectors.json, which install.ps1, the VS Code extension and the Neovim plugin also read."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(lambda: subprocess.run(["rm", "-rf", str(self.tmp)]))

    def install(self, env_extra=None, path_prefix=None):
        env = {
            k: v for k, v in os.environ.items() if not k.startswith(("JAI_", "JAIC_"))
        }
        env.update(
            JAI_BASE_URL=f"file://{self.tmp / 'release'}",
            JAI_VERSION="1.0.0",
            JAI_INSTALL_DIR=str(self.tmp / "share"),
            JAI_BIN_DIR=str(self.tmp / "bin"),
        )
        if path_prefix:
            env["PATH"] = f"{path_prefix}{os.pathsep}{env['PATH']}"
        env.update(env_extra or {})
        return subprocess.run(
            ["sh", str(ROOT / "install.sh")], env=env, capture_output=True, text=True
        )

    def release(self, sums_text):
        directory = self.tmp / "release" / "v1.0.0"
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "SHA256SUMS").write_text(sums_text)

    def test_the_platform_picks_the_archive(self):
        self.release("")  # no entries: the error names the archive install.sh chose
        for vector in VECTORS["platforms"]:
            sysname, machine = vector["uname"]
            if sysname == "Windows_NT":
                continue
            with self.subTest(sysname=sysname, machine=machine):
                stub = self.tmp / f"uname-{sysname}-{machine}"
                stub.mkdir()
                script = stub / "uname"
                script.write_text(
                    f'#!/bin/sh\ncase "$1" in -s) echo {sysname};; -m) echo {machine};; *) echo {sysname};; esac\n'
                )
                script.chmod(0o755)
                result = self.install(path_prefix=str(stub))
                self.assertNotEqual(result.returncode, 0)
                if vector["asset"]:
                    self.assertIn(f"has no {vector['asset']}", result.stderr)
                else:
                    self.assertIn("no prebuilt Jai toolchain", result.stderr)

    def test_sha256sums_line_formats(self):
        if PLATFORM is None:
            self.skipTest("install.sh has no build for this platform")
        asset = f"jacket-{PLATFORM}.tar.gz"
        directory = self.tmp / "release" / "v1.0.0"
        stage = self.tmp / "stage" / f"jacket-{PLATFORM}"
        stage.mkdir(parents=True)
        for tool in TOOLS:
            (stage / tool).write_text("#!/bin/sh\n")
            (stage / tool).chmod(0o755)
        directory.mkdir(parents=True)
        with tarfile.open(directory / asset, "w:gz") as tar:
            tar.add(stage, arcname=f"jacket-{PLATFORM}")
        digest = sha256_bytes((directory / asset).read_bytes())
        for case in VECTORS["sha256sums_lines"]:
            with self.subTest(case["name"]):
                line = case["line"].format(
                    hash=digest, HASH=digest.upper(), asset=asset
                )
                self.release(line + "\n")
                result = self.install()
                if case["accepted"]:
                    self.assertEqual(result.returncode, 0, result.stderr)
                else:
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(f"has no {asset}", result.stderr)
                subprocess.run(
                    ["rm", "-rf", str(self.tmp / "share"), str(self.tmp / "bin")]
                )


if __name__ == "__main__":
    unittest.main()
