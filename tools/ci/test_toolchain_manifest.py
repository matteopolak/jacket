import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import toolchain_manifest as tm
from toolkit.repo import ROOT

FIXTURES = ROOT / "editors" / "fixtures"


class ToolchainManifest(unittest.TestCase):
    def test_sums_become_the_shared_fixture(self):
        sums = tm.from_sums((FIXTURES / "SHA256SUMS").read_text())
        self.assertEqual(
            tm.manifest("v0.4.0", sums),
            json.loads((FIXTURES / "toolchain-manifest.json").read_text()),
        )

    def test_non_archives_are_left_out(self):
        sums = tm.from_sums((FIXTURES / "SHA256SUMS").read_text())
        self.assertFalse(any("vsix" in name or name == "install.sh" for name in sums))

    def test_archives_folder_gives_the_same_hashes_as_sums(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "jacket-linux-x64.tar.gz").write_bytes(b"abc")
            (Path(tmp) / "jacket-vscode-1.vsix").write_bytes(b"zzz")
            got = tm.from_archives(Path(tmp))
        self.assertEqual(list(got), ["jacket-linux-x64.tar.gz"])
        self.assertEqual(got["jacket-linux-x64.tar.gz"], "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")

    def test_rejects_bad_version_and_empty(self):
        with self.assertRaises(ValueError):
            tm.manifest("main", {"jacket-linux-x64.tar.gz": "0" * 64})
        with self.assertRaises(ValueError):
            tm.manifest("1.2.3", {})

    def test_main_writes_the_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "m.json"
            self.assertEqual(tm.main(["--version", "0.4.0", "--sums", str(FIXTURES / "SHA256SUMS"), "--out", str(out)]), 0)
            self.assertEqual(out.read_text(), (FIXTURES / "toolchain-manifest.json").read_text())


if __name__ == "__main__":
    unittest.main()
