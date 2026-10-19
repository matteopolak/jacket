import tempfile
import unittest
from pathlib import Path

import bump_version as bv

MANIFEST = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "0.1.0"\nrust-version = "1.95"\n'
LOCK = ('[[package]]\nname = "a"\nversion = "0.1.0"\ndependencies = [\n "serde",\n]\n\n'
        '[[package]]\nname = "serde"\nversion = "0.1.0"\nsource = "registry+https://example.test/index"\n\n'
        '[[package]]\nname = "fuzz"\nversion = "0.0.0"\n')
LUA = "---doc\nreturn '0.1.0'\n"
JAIFMT = '// doc\nJAIFMT_VERSION :: "0.1.0";\n'
CHANGELOG = "# Changelog\n\n## [0.1.0] - 2026-01-01\n\nOld.\n"


class Bump(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        files = {"Cargo.toml": MANIFEST, "Cargo.lock": LOCK, "fuzz/Cargo.lock": LOCK,
                 bv.VERSION_LUA: LUA, bv.VERSION_JAIFMT: JAIFMT, "CHANGELOG.md": CHANGELOG}
        for rel, text in files.items():
            (self.root / rel).parent.mkdir(parents=True, exist_ok=True)
            (self.root / rel).write_text(text)

    def test_every_copy_moves_and_registry_packages_stay(self):
        self.assertEqual(bv.bump(self.root, "0.2.0", "2026-02-02"), "0.1.0")
        self.assertIn('[workspace.package]\nversion = "0.2.0"\nrust-version = "1.95"', bv.read(self.root, "Cargo.toml"))
        for lock in bv.LOCKS:
            text = bv.read(self.root, lock)
            self.assertIn('name = "a"\nversion = "0.2.0"', text)
            self.assertIn('name = "serde"\nversion = "0.1.0"', text)
            self.assertIn('name = "fuzz"\nversion = "0.0.0"', text)
        self.assertIn("return '0.2.0'", bv.read(self.root, bv.VERSION_LUA))
        self.assertIn('JAIFMT_VERSION :: "0.2.0";', bv.read(self.root, bv.VERSION_JAIFMT))
        changelog = bv.read(self.root, "CHANGELOG.md")
        self.assertTrue(changelog.startswith("# Changelog\n\n## [0.2.0] - 2026-02-02\n\n## [0.1.0]"))
        self.assertEqual(bv.disagreements(self.root), [])

    def test_existing_changelog_section_is_kept(self):
        (self.root / "CHANGELOG.md").write_text("# Changelog\n\n## [0.2.0] - 2026-01-05\n\nWritten.\n")
        bv.bump(self.root, "0.2.0", "2026-02-02")
        changelog = bv.read(self.root, "CHANGELOG.md")
        self.assertEqual(changelog.count("## [0.2.0]"), 1)
        self.assertIn("2026-01-05", changelog)

    def test_bad_versions_are_refused(self):
        for bad in ("1.2", "v1.2.3", "1.2.3-rc1", "0.1.0"):
            with self.assertRaises(SystemExit):
                bv.bump(self.root, bad)

    def test_check_flags_a_stale_plugin_version_or_lock(self):
        (self.root / bv.VERSION_LUA).write_text("return '0.0.9'\n")
        (self.root / "fuzz/Cargo.lock").write_text(LOCK.replace('name = "a"\nversion = "0.1.0"', 'name = "a"\nversion = "0.0.9"'))
        problems = bv.disagreements(self.root)
        (self.root / bv.VERSION_JAIFMT).write_text('JAIFMT_VERSION :: "0.0.9";\n')
        problems = bv.disagreements(self.root)
        self.assertEqual(len(problems), 3)
        self.assertIn("version.lua", problems[0])
        self.assertIn("version.jai", problems[1])
        self.assertIn("fuzz/Cargo.lock", problems[2])


class Repository(unittest.TestCase):
    def test_version_copies_agree(self):
        self.assertEqual(bv.disagreements(bv.ROOT), [])


if __name__ == "__main__":
    unittest.main()
