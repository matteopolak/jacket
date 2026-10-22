"""CI package sources are explicit, and failed required updates stay fatal."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("apt_lib.sh").resolve()


@unittest.skipIf(sys.platform == "win32", "requires bash")
class AptConfiguration(unittest.TestCase):
    def run_bash(self, body, *args):
        return subprocess.run(
            [
                "bash",
                "-euo",
                "pipefail",
                "-c",
                body,
                "test",
                str(SCRIPT),
                *map(str, args),
            ],
            text=True,
            capture_output=True,
        )

    def configure(self, root):
        return self.run_bash('source "$1"; sudo() { "$@"; }; apt_configure "$2"', root)

    def test_vendor_feeds_are_excluded_and_llvm_survives_reconfiguration(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            original = root / "sources.list.d"
            original.mkdir()
            ubuntu = "Types: deb\nURIs: http://archive.ubuntu.com/ubuntu\nSuites: noble\nComponents: main\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n"
            (original / "ubuntu.sources").write_text(ubuntu)
            vendor = original / "microsoft-prod.list"
            vendor.write_text(
                "deb https://packages.microsoft.com/ubuntu/24.04/prod noble main\n"
            )
            result = self.configure(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            parts = root / "jai-ci-sources.list.d"
            self.assertEqual(
                sorted(p.name for p in parts.iterdir()), ["ubuntu.sources"]
            )
            self.assertEqual((parts / "ubuntu.sources").read_text(), ubuntu)
            llvm = parts / "jai-ci-llvm.list"
            llvm.write_text(
                "deb [signed-by=/usr/share/keyrings/jai-ci-llvm.gpg] https://apt.llvm.org/noble/ llvm-toolchain-noble-23 main\n"
            )
            result = self.configure(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(
                vendor.exists(), "the original runner configuration is preserved"
            )
            self.assertEqual(
                sorted(p.name for p in parts.iterdir()),
                ["jai-ci-llvm.list", "ubuntu.sources"],
            )
            config = (root / "apt.conf.d/80jai-ci-network").read_text()
            self.assertIn(f'Dir::Etc::sourceparts "{parts}";', config)
            self.assertIn('Dir::Etc::sourcelist "/dev/null";', config)
            self.assertIn('APT::Update::Error-Mode "any";', config)
            self.assertIn("signed-by=", llvm.read_text())

    def test_legacy_ubuntu_source_list(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = "deb http://archive.ubuntu.com/ubuntu jammy main\n"
            (root / "sources.list").write_text(source)
            result = self.configure(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                (root / "jai-ci-sources.list.d/ubuntu.list").read_text(), source
            )

    def test_missing_base_sources_fail_before_replacing_configuration(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.configure(tmp)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("no Ubuntu base package sources", result.stderr)
            self.assertFalse((Path(tmp) / "apt.conf.d/80jai-ci-network").exists())

    def test_failed_required_update_is_retried_and_remains_a_failure(self):
        result = self.run_bash("""source "$1"
sudo() {
  printf '%s\\n' "$*"
  [[ "$1" != timeout ]]
}
apt_get update""")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout.count("timeout 600 apt-get update"), 3)
        self.assertEqual(result.stdout.count("dpkg --configure -a"), 3)
        self.assertIn("attempt 3 of 3", result.stderr)


if __name__ == "__main__":
    unittest.main()
