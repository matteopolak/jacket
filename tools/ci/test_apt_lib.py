"""CI package sources are explicit, and failed required updates stay fatal."""

import os
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
            # CI's setup-unix sets it for the job; these tests choose their own.
            env={k: v for k, v in os.environ.items() if k != "JAI_APT_ARCHIVES"},
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
        # A failed attempt's own output is shown on standard error.
        self.assertEqual(result.stderr.count("timeout 180 apt-get update"), 5)
        self.assertEqual(result.stdout.count("dpkg --configure -a"), 5)
        self.assertIn("attempt 5 of 5", result.stderr)

    def test_successful_command_prints_nothing(self):
        result = self.run_bash("""source "$1"
sudo() { printf 'Unpacking %s\\n' "$*"; }
apt_get install --yes llvm""")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout + result.stderr, "")

    def test_prefetch_downloads_each_planned_package_and_never_fails(self):
        result = self.run_bash("""source "$1"
sudo() {
  if [[ "$1" == apt-get ]]; then
    printf "'http://m/a_1.deb' a_1.deb 10 SHA256:x\\n'http://m/b_2.deb' b_2.deb 20 SHA256:y\\n"
  else
    shift 2
    [[ "$1" != curl ]] || { shift; printf 'curl %s\\n' "$*"; return 22; }
  fi
}
export -f sudo
apt_prefetch pkg""")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("/var/cache/apt/archives/a_1.deb http://m/a_1.deb", result.stdout)
        self.assertIn("/var/cache/apt/archives/b_2.deb http://m/b_2.deb", result.stdout)

    def test_prefetch_writes_into_the_cached_archive_directory(self):
        result = self.run_bash("""source "$1"
sudo() {
  if [[ "$1" == apt-get ]]; then
    printf "'http://m/a_1.deb' a_1.deb 10 SHA256:x\\n"
  else
    shift 2
    [[ "$1" != curl ]] || { shift; printf 'curl %s\\n' "$*"; }
  fi
}
export -f sudo
JAI_APT_ARCHIVES=/cache/apt apt_prefetch pkg""")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("/cache/apt/a_1.deb http://m/a_1.deb", result.stdout)

    def test_prefetch_tolerates_a_failed_plan(self):
        result = self.run_bash("""source "$1"
sudo() { return 100; }
apt_prefetch pkg""")
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
