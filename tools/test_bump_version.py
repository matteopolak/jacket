import tempfile
import json
import unittest
from pathlib import Path

import bump_version as bv
import release
from generators import generate_version_files as gen

MANIFEST = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "0.1.0"\nrust-version = "1.95"\n'
LOCK = ('[[package]]\nname = "a"\nversion = "0.1.0"\ndependencies = [\n "serde",\n]\n\n'
        '[[package]]\nname = "serde"\nversion = "0.1.0"\nsource = "registry+https://example.test/index"\n\n'
        '[[package]]\nname = "fuzz"\nversion = "0.0.0"\n')
CHANGELOG = "# Changelog\n\n## [0.1.0] - 2026-01-01\n\nOld.\n"


class Bump(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        files = {"Cargo.toml": MANIFEST, "Cargo.lock": LOCK, "fuzz/Cargo.lock": LOCK, "CHANGELOG.md": CHANGELOG}
        for rel in gen.OUTPUTS:
            files[rel] = ""
        for rel, text in files.items():
            (self.root / rel).parent.mkdir(parents=True, exist_ok=True)
            (self.root / rel).write_text(text)
        gen.write(self.root)

    def test_every_copy_moves_and_registry_packages_stay(self):
        self.assertEqual(bv.bump(self.root, "0.2.0", "2026-02-02"), "0.1.0")
        self.assertIn('[workspace.package]\nversion = "0.2.0"\nrust-version = "1.95"', bv.read(self.root, "Cargo.toml"))
        for lock in bv.LOCKS:
            text = bv.read(self.root, lock)
            self.assertIn('name = "a"\nversion = "0.2.0"', text)
            self.assertIn('name = "serde"\nversion = "0.1.0"', text)
            self.assertIn('name = "fuzz"\nversion = "0.0.0"', text)
        self.assertIn("return '0.2.0'", bv.read(self.root, "editors/nvim/lua/jai/version.lua"))
        self.assertIn('JAIFMT_VERSION :: "0.2.0";', bv.read(self.root, "jaifmt/version.jai"))
        changelog = bv.read(self.root, "CHANGELOG.md")
        self.assertTrue(changelog.startswith("# Changelog\n\n## [0.2.0] - 2026-02-02\n\n## [0.1.0]"))
        self.assertEqual(bv.disagreements(self.root), [])

    def docs_builder(self):
        builder = self.root / "website/docs"
        builder.mkdir(parents=True)
        package = {"name": "owned-builder", "version": "0.1.0", "dependencies": {"owned": "0.1.0"}}
        lock = {"name": package["name"], "version": "0.1.0", "lockfileVersion": 3,
                "packages": {"": package.copy(), "node_modules/owned": {"version": "0.1.0", "integrity": "unchanged"}}}
        for name, data in (("package.json", package), ("package-lock.json", lock)):
            (builder / name).write_text(json.dumps(data, indent=2) + "\n")
        return builder, package, lock

    def test_docs_builder_identity_moves_with_jacket_without_changing_dependency_pins(self):
        builder, package, lock = self.docs_builder()
        bv.bump(self.root, "0.2.0", "2026-02-02")
        updated = json.loads((builder / "package.json").read_text())
        locked = json.loads((builder / "package-lock.json").read_text())
        self.assertEqual(updated["version"], "0.2.0")
        self.assertEqual(locked["version"], "0.2.0")
        self.assertEqual(locked["packages"][""]["version"], "0.2.0")
        self.assertEqual(updated["dependencies"], package["dependencies"])
        self.assertEqual(locked["packages"]["node_modules/owned"], lock["packages"]["node_modules/owned"])
        self.assertEqual(locked["packages"][""]["dependencies"], package["dependencies"])
        self.assertEqual(bv.disagreements(self.root), [])

    def test_check_reports_each_stale_docs_builder_identity(self):
        builder, _, _ = self.docs_builder()
        for filename, path in (("package.json", ("version",)), ("package-lock.json", ("version",)),
                               ("package-lock.json", ("packages", "", "version"))):
            source = builder / filename
            data = json.loads(source.read_text())
            current = data
            for part in path[:-1]:
                current = current[part]
            current[path[-1]] = "stale"
            source.write_text(json.dumps(data))
        problems = bv.disagreements(self.root)
        self.assertEqual(len(problems), 3)
        self.assertTrue(all("website/docs/" in problem for problem in problems))

    def test_malformed_builder_metadata_does_not_leave_a_partial_bump(self):
        builder, _, _ = self.docs_builder()
        (builder / "package-lock.json").write_text("{")
        before = (self.root / "Cargo.toml").read_text()
        with self.assertRaisesRegex(SystemExit, "package-lock.json"):
            bv.bump(self.root, "0.2.0", "2026-02-02")
        self.assertEqual((self.root / "Cargo.toml").read_text(), before)

    def test_existing_changelog_section_is_kept(self):
        (self.root / "CHANGELOG.md").write_text("# Changelog\n\n## [0.2.0] - 2026-01-05\n\nWritten.\n")
        bv.bump(self.root, "0.2.0", "2026-02-02")
        changelog = bv.read(self.root, "CHANGELOG.md")
        self.assertEqual(changelog.count("## [0.2.0]"), 1)
        self.assertIn("2026-01-05", changelog)

    def test_unreleased_heading_becomes_the_new_version(self):
        (self.root / "CHANGELOG.md").write_text(
            "# Changelog\n\n## [Unreleased]\n\n### New\n\n- A thing.\n\n## [0.1.0] - 2026-01-01\n\nOld.\n")
        bv.bump(self.root, "0.2.0", "2026-02-02")
        self.assertEqual(
            bv.read(self.root, "CHANGELOG.md"),
            "# Changelog\n\n## [0.2.0] - 2026-02-02\n\n### New\n\n- A thing.\n\n## [0.1.0] - 2026-01-01\n\nOld.\n")

    def test_unreleased_forms_preserve_notes_and_extract_only_new_release(self):
        for heading in ("## Unreleased", "## [Unreleased]", "## Unreleased   "):
            with self.subTest(heading=heading):
                before = f"# Changelog\n\n{heading}\n\n### Added\n\n- New.\n\n## [0.1.0]\nOld.\n"
                after = bv.bump_changelog(before, "0.2.0", "2026-02-02")
                self.assertEqual(after, before.replace(heading, "## [0.2.0] - 2026-02-02"))
                self.assertEqual(release.changelog_section(after, "0.2.0"), "### Added\n\n- New.\n")

    def test_prewritten_version_section_keeps_its_heading_suffix(self):
        text = "## [0.2.0] - pending\nWritten.\n## Unreleased\nLater.\n"
        self.assertEqual(bv.bump_changelog(text, "0.2.0", "today"), text)

    def test_unreleased_preserves_final_newline_presence(self):
        for ending in ("", "\n", "\r\n"):
            before = "## Unreleased\n- New." + ending
            self.assertEqual(bv.bump_changelog(before, "0.2.0", "today"),
                             "## [0.2.0] - today\n- New." + ending)

    def test_fenced_headings_do_not_steal_release(self):
        for fence in ("```", "~~~~"):
            before = f"# Changelog\n\n{fence}markdown\n## [0.2.0]\n## Unreleased\n{fence}\n\n## Unreleased\nReal.\n"
            self.assertEqual(bv.bump_changelog(before, "0.2.0", "today"),
                             before.replace("## Unreleased\nReal.", "## [0.2.0] - today\nReal."))

    def test_inline_backticks_do_not_hide_the_real_unreleased_heading(self):
        before = "# Changelog\n\n```example```\n\n## Unreleased\nNew.\n\n## [0.1.0]\nOld.\n"
        after = bv.bump_changelog(before, "0.2.0", "today")
        self.assertEqual(after, before.replace("## Unreleased", "## [0.2.0] - today"))
        self.assertEqual(release.changelog_section(after, "0.2.0"), "New.\n")

    def test_tilde_info_may_contain_backticks(self):
        before = "~~~example`\n## [0.2.0]\n## Unreleased\n~~~\n\n## Unreleased\nNew.\n"
        self.assertEqual(bv.bump_changelog(before, "0.2.0", "today"),
                         before.replace("## Unreleased\nNew.", "## [0.2.0] - today\nNew."))

    def test_unreleased_lookalikes_are_not_renamed(self):
        for heading in ("### Unreleased", "## Unreleasedness", "## [Unreleased", "## Unreleased]", "## [Unreleased] suffix", "prose Unreleased"):
            before = heading + "\n\n## [0.1.0]\nOld.\n"
            self.assertEqual(bv.bump_changelog(before, "0.2.0", "today"),
                             heading + "\n\n## [0.2.0] - today\n\n## [0.1.0]\nOld.\n")

    def test_bad_versions_are_refused(self):
        for bad in ("1.2", "v1.2.3", "1.2.3-rc1", "0.1.0"):
            with self.assertRaises(SystemExit):
                bv.bump(self.root, bad)

    def test_bump_edits_only_the_manifest_locks_changelog_and_generated_copies(self):
        before = {p: p.read_text() for p in self.root.rglob("*") if p.is_file()}
        bv.bump(self.root, "0.2.0", "2026-02-02")
        changed = {str(p.relative_to(self.root)) for p, text in before.items() if p.read_text() != text}
        self.assertEqual(changed, {"Cargo.toml", "Cargo.lock", "fuzz/Cargo.lock", "CHANGELOG.md", *gen.OUTPUTS})

    def test_check_flags_a_stale_generated_copy_or_lock(self):
        (self.root / "editors/nvim/lua/jai/version.lua").write_text("return '0.0.9'\n")
        (self.root / "fuzz/Cargo.lock").write_text(LOCK.replace('name = "a"\nversion = "0.1.0"', 'name = "a"\nversion = "0.0.9"'))
        problems = bv.disagreements(self.root)
        self.assertEqual(len(problems), 2)
        self.assertIn("version.lua", problems[0])
        self.assertIn("fuzz/Cargo.lock", problems[1])


class Generator(unittest.TestCase):
    def test_checked_in_copies_match_cargo_toml(self):
        self.assertEqual(gen.stale(bv.ROOT), [])

    def test_output_is_what_the_readers_parse(self):
        text = gen.render("1.2.3")
        self.assertRegex(text[gen.OUTPUTS[0]], r'(?m)^JAIFMT_VERSION :: "1\.2\.3";$')
        self.assertRegex(text[gen.OUTPUTS[1]], r"(?m)^return '1\.2\.3'$")


class Repository(unittest.TestCase):
    def test_version_copies_agree(self):
        self.assertEqual(bv.disagreements(bv.ROOT), [])


if __name__ == "__main__":
    unittest.main()
