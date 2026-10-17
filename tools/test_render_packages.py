import re
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import render_packages

SUMS = "\n".join([
    f"{'a' * 64}  jai-linux-x64.tar.gz",
    f"{'b' * 64}  jai-macos-arm64.tar.gz",
    f"{'1' * 64}  jai-linux-arm64.tar.gz",
    f"{'2' * 64}  jai-macos-x64.tar.gz",
    f"{'e' * 64}  jai-vscode-1.2.3.vsix",
    "",
])
# 0.4.0 and earlier named their archives jaic-<platform>, and had no Intel macOS or arm64 Linux build.
OLD_SUMS = "\n".join(line for line in SUMS.splitlines() if "macos-x64" not in line and "linux-arm64" not in line)
OLD_SUMS = OLD_SUMS.replace("  jai-", "  jaic-").replace(" *jai-", " *jaic-").replace("jaic-vscode", "jai-vscode")


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
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jai-macos-arm64.tar.gz"',
                      formula)
        self.assertIn(f'sha256 "{"b" * 64}"', formula)
        self.assertIn(f'sha256 "{"a" * 64}"', formula)
        self.assertIn("class Jacket < Formula", formula)
        # Every platform has its own build: no architecture requirement.
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jai-macos-x64.tar.gz"', formula)
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v1.2.3/jai-linux-arm64.tar.gz"', formula)
        self.assertIn(f'sha256 "{"1" * 64}"', formula)
        self.assertIn(f'sha256 "{"2" * 64}"', formula)
        self.assertNotIn("depends_on arch: :", formula)
        self.assertNotIn("do\n\n", formula)

    def test_releases_without_intel_macos_or_arm64_linux_require_the_built_architecture(self):
        jai_named = "\n".join(line for line in SUMS.splitlines() if "macos-x64" not in line and "linux-arm64" not in line)
        for version, sums, prefix in (("0.3.0", OLD_SUMS, "jaic"), ("0.4.0", OLD_SUMS, "jaic"), ("0.4.1", jai_named, "jai")):
            formula = self.render(sums, version=version)[1]["homebrew/jacket.rb"]
            self.assertIn("depends_on arch: :arm64\n\n    on_arm do", formula)
            self.assertIn("depends_on arch: :x86_64\n\n    on_arm do", formula)
            self.assertNotIn("macos-x64", formula)
            self.assertNotIn("linux-arm64", formula)
            self.assertEqual(formula.count(f"/{prefix}-macos-arm64.tar.gz"), 2)
        # A later release must have them.
        sums = "\n".join(line for line in SUMS.splitlines() if "macos-x64" not in line)
        with self.assertRaisesRegex(ValueError, "jai-macos-x64.tar.gz"):
            self.render(sums, version="0.4.2")

    def test_missing_asset_is_an_error(self):
        sums = "\n".join(line for line in SUMS.splitlines() if "linux-x64" not in line)
        with self.assertRaisesRegex(ValueError, "jai-linux-x64.tar.gz"):
            self.render(sums)

    def test_releases_up_to_0_4_0_keep_their_jaic_archive_names(self):
        _, files = self.render(OLD_SUMS, version="0.4.0")
        self.assertIn('url "https://github.com/matteopolak/jacket/releases/download/v0.4.0/jaic-linux-x64.tar.gz"',
                      files["homebrew/jacket.rb"])
        with self.assertRaisesRegex(ValueError, "jaic-linux-x64.tar.gz"):
            self.render("\n".join(line for line in OLD_SUMS.splitlines() if "linux-x64" not in line),
                        version="0.3.0")

    def test_the_prefix_follows_the_archives_present(self):
        # A dry run of release.yml builds jai-* archives while Cargo.toml still says 0.4.0.
        self.assertIn("/v0.4.0/jai-macos-arm64.tar.gz", self.render(SUMS, version="0.4.0")[1]["homebrew/jacket.rb"])
        self.assertEqual(render_packages.expected_prefix("0.4.0"), "jaic")
        self.assertEqual(render_packages.expected_prefix("0.3.9"), "jaic")
        self.assertEqual(render_packages.expected_prefix("0.4.1"), "jai")
        self.assertEqual(render_packages.expected_prefix("0.4.1-rc.1"), "jai")
        self.assertEqual(render_packages.expected_prefix("1.0.0"), "jai")

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
            names = [f"jai-{asset}" for asset in render_packages.PLATFORM_ASSETS.values()]
            for name in names:
                (Path(tmp) / name).write_bytes(b"")
            sums = render_packages.hash_archives(tmp)
        empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        self.assertEqual(sums, {name: empty for name in names})


if __name__ == "__main__":
    unittest.main()
