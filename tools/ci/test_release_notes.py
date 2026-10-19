import unittest

import release_notes as rn

LOG = """# Changelog

## [Unreleased]

## [0.2.0] - 2026-01-02

### Added

- b

## [0.1.0] - 2026-01-01

- a
"""


class ReleaseNotes(unittest.TestCase):
    def test_takes_the_section_up_to_the_next_heading(self):
        self.assertEqual(rn.section(LOG, "0.2.0"), "### Added\n\n- b\n")

    def test_tag_prefix_and_last_section(self):
        self.assertEqual(rn.section(LOG, "v0.1.0"), "- a\n")

    def test_missing_or_empty_section_is_empty(self):
        self.assertEqual(rn.section(LOG, "0.3.0"), "")
        self.assertEqual(rn.section(LOG, "Unreleased"), "")

    def test_does_not_match_a_version_prefix(self):
        self.assertEqual(rn.section("## [0.20.0]\n- x\n", "0.2.0"), "")

    def test_the_real_changelog_has_the_current_top_section(self):
        import re
        text = rn.ROOT.joinpath("CHANGELOG.md").read_text(encoding="utf-8")
        version = re.search(r"^## \[(\d+\.\d+\.\d+)\]", text, re.M)[1]
        self.assertTrue(rn.section(text, version))


if __name__ == "__main__":
    unittest.main()
