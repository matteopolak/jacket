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
    f"{'c' * 64} *jai-windows-arm64.zip",
    f"{'D' * 64}  jai-windows-x64.zip",
    f"{'e' * 64}  jai-vscode-1.2.3.vsix",
    "",
])
# 0.4.0 and earlier named their archives jaic-<platform>.
OLD_SUMS = SUMS.replace("  jai-", "  jaic-").replace(" *jai-", " *jaic-").replace("jaic-vscode", "jai-vscode")


class RenderTests(unittest.TestCase):
    def render(self, sums=SUMS, version="1.2.3"):
        values = render_packages.substitutions(version, render_packages.parse_sums(sums), None, "2026-10-07")
        out = Path(tempfile.mkdtemp())
        paths = render_packages.render_all(values, out)
        return out, {path.relative_to(out).as_posix(): path.read_text() for path in paths}

    def test_formula_takes_the_version_urls_and_lowercase_checksums(self):
        _, files = self.render()
        formula = files["homebrew/jai.rb"]
        self.assertIn("for release v1.2.3.", formula)
        self.assertIn('url "https://github.com/matteopolak/jai/releases/download/v1.2.3/jai-macos-arm64.tar.gz"',
                      formula)
        self.assertIn(f'sha256 "{"b" * 64}"', formula)
        self.assertIn(f'sha256 "{"a" * 64}"', formula)
        self.assertIn("class Jai < Formula", formula)

    def test_winget_manifests_live_under_the_identifier_path_with_uppercase_checksums(self):
        _, files = self.render()
        base = "winget/manifests/m/matteopolak/jai/1.2.3/"
        self.assertEqual(sorted(files), ["homebrew/jai.rb",
                                         base + "matteopolak.jai.installer.yaml",
                                         base + "matteopolak.jai.locale.en-US.yaml",
                                         base + "matteopolak.jai.yaml"])
        installer = files[base + "matteopolak.jai.installer.yaml"]
        self.assertIn(f"InstallerSha256: {'D' * 64}", installer)
        self.assertIn(f"InstallerSha256: {'C' * 64}", installer)
        self.assertIn("ReleaseDate: 2026-10-07", installer)
        self.assertIn("InstallerUrl: https://github.com/matteopolak/jai/releases/download/v1.2.3/jai-windows-x64.zip",
                      installer)
        self.assertIn("RelativeFilePath: jai-windows-arm64\\jaifmt.exe", installer)
        for text in files.values():
            self.assertIsNone(re.search(r"@[A-Z0-9_]+@", text))
            if "PackageIdentifier" in text:
                self.assertIn("PackageIdentifier: matteopolak.jai\nPackageVersion: 1.2.3\n", text)

    def test_winget_manifests_share_one_schema_version(self):
        _, files = self.render()
        versions = {re.search(r"^ManifestVersion: (\S+)$", text, re.M).group(1)
                    for name, text in files.items() if name.startswith("winget/")}
        self.assertEqual(len(versions), 1)
        (version,) = versions
        for name, text in files.items():
            if name.startswith("winget/"):
                self.assertIn(f".{version}.schema.json", text)

    def test_missing_asset_is_an_error(self):
        sums = "\n".join(line for line in SUMS.splitlines() if "windows-arm64" not in line)
        with self.assertRaisesRegex(ValueError, "jai-windows-arm64.zip"):
            self.render(sums)

    def test_releases_up_to_0_4_0_keep_their_jaic_archive_names(self):
        _, files = self.render(OLD_SUMS, version="0.4.0")
        self.assertIn('url "https://github.com/matteopolak/jai/releases/download/v0.4.0/jaic-linux-x64.tar.gz"',
                      files["homebrew/jai.rb"])
        installer = files["winget/manifests/m/matteopolak/jai/0.4.0/matteopolak.jai.installer.yaml"]
        self.assertIn("RelativeFilePath: jaic-windows-x64\\jaic.exe", installer)
        self.assertIn("/v0.4.0/jaic-windows-arm64.zip", installer)
        self.assertNotIn("/jai-windows", installer)
        with self.assertRaisesRegex(ValueError, "jaic-windows-arm64.zip"):
            self.render("\n".join(line for line in OLD_SUMS.splitlines() if "windows-arm64" not in line),
                        version="0.3.0")

    def test_the_prefix_follows_the_archives_present(self):
        # A dry run of release.yml builds jai-* archives while Cargo.toml still says 0.4.0.
        self.assertIn("/v0.4.0/jai-macos-arm64.tar.gz", self.render(SUMS, version="0.4.0")[1]["homebrew/jai.rb"])
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
            render_packages.main(["--version", "v1.2.3", "--sums", str(sums), "--out", tmp,
                                  "--release-date", "2026-01-02"])
            self.assertIn("/v1.2.3/", (Path(tmp) / "homebrew" / "jai.rb").read_text())

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
