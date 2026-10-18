import re
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import render_packages

SUMS = "\n".join([
    f"{'a' * 64}  jacket-linux-x64.tar.gz",
    f"{'b' * 64}  jacket-macos-arm64.tar.gz",
    f"{'1' * 64}  jacket-linux-arm64.tar.gz",
    f"{'2' * 64}  jacket-macos-x64.tar.gz",
    f"{'e' * 64}  jacket-vscode-1.2.3.vsix",
    "",
])

class RenderTests(unittest.TestCase):
    def render(self, sums=SUMS, version="1.2.3"):
        values = render_packages.substitutions(version, render_packages.parse_sums(sums), None)
        out = Path(tempfile.mkdtemp())
        paths = render_packages.render_all(values, out)
        return out, {path.relative_to(out).as_posix(): path.read_text() for path in paths}

    def test_formula_takes_the_version_urls_and_lowercase_checksums(self):
        _, files = self.render()
        formula = files["homebrew/jacket.rb"]
        self.assertIn("for release v1.2.3.", formula)
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jacket-macos-arm64.tar.gz"',
                      formula)
        self.assertIn(f'sha256 "{"b" * 64}"', formula)
        self.assertIn(f'sha256 "{"a" * 64}"', formula)
        self.assertIn("class Jacket < Formula", formula)
        # Every platform has its own build: no architecture requirement.
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jacket-macos-x64.tar.gz"', formula)
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jacket-linux-arm64.tar.gz"', formula)
        self.assertIn(f'sha256 "{"1" * 64}"', formula)
        self.assertIn(f'sha256 "{"2" * 64}"', formula)
        self.assertNotIn("depends_on arch: :", formula)
        self.assertNotIn("do\n\n", formula)

    def test_missing_asset_is_an_error(self):
        for name in ("linux-x64", "macos-x64", "linux-arm64"):
            sums = "\n".join(line for line in SUMS.splitlines() if name not in line)
            with self.assertRaisesRegex(ValueError, f"jacket-{name}.tar.gz"):
                self.render(sums)

    def test_malformed_input_is_an_error(self):
        with self.assertRaisesRegex(ValueError, "line 1"):
            render_packages.parse_sums("not a checksum line\n")
        with self.assertRaisesRegex(ValueError, "x.y.z"):
            self.render(version="v1.2")

    def test_main_strips_the_tag_prefix(self):
        with tempfile.TemporaryDirectory() as tmp:
            sums = Path(tmp) / "SHA256SUMS"
            sums.write_text(SUMS)
            render_packages.main(["--version", "v1.2.3", "--sums", str(sums), "--out", tmp])
            self.assertIn("/v1.2.3/", (Path(tmp) / "homebrew" / "jacket.rb").read_text())

    def test_archives_are_hashed_like_sha256sum(self):
        with tempfile.TemporaryDirectory() as tmp:
            names = [asset for asset in render_packages.PLATFORM_ASSETS.values()]
            for name in names:
                (Path(tmp) / name).write_bytes(b"")
            sums = render_packages.hash_archives(tmp)
        empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        self.assertEqual(sums, {name: empty for name in names})


if __name__ == "__main__":
    unittest.main()
