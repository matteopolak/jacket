import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "tools/ci/install_rust_toolchain.sh"


@unittest.skipIf(sys.platform == "win32", "needs bash")
class InstallRustToolchain(unittest.TestCase):
    def run_script(self, *args):
        with tempfile.TemporaryDirectory() as tmp:
            rustup = Path(tmp) / "rustup"
            rustup.write_text('#!/bin/sh\necho "rustup $*"\n')
            rustup.chmod(rustup.stat().st_mode | stat.S_IEXEC)
            env = dict(os.environ, PATH=tmp + os.pathsep + os.environ["PATH"])
            result = subprocess.run(["bash", str(SCRIPT), *args], cwd=ROOT, env=env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.splitlines()

    def channel(self):
        for line in (ROOT / "rust-toolchain.toml").read_text().splitlines():
            if line.startswith("channel"):
                return line.split('"')[1]

    def test_installs_the_pinned_channel(self):
        self.assertEqual(self.run_script(), [f"rustup toolchain install {self.channel()} --profile minimal"])

    def test_components_and_targets(self):
        channel = self.channel()
        self.assertEqual(
            self.run_script("--component", "rustfmt", "--target", "wasm32-unknown-unknown"),
            [
                f"rustup toolchain install {channel} --profile minimal --component rustfmt",
                f"rustup target add wasm32-unknown-unknown --toolchain {channel}",
            ],
        )


if __name__ == "__main__":
    unittest.main()
