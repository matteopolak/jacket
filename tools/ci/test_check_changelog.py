import unittest

import check_changelog as cc

GOOD = """# Changelog

## [Unreleased]

Jacket builds for new targets. Audio no longer crackles.

### Highlights

- New targets: WebAssembly and ARMv7 Linux ([targets](docs/compiler/targets.md)).

### Breaking changes

| Was | Now |
| --- | --- |
| `size()` | `size(layout)` |

### Fixes

- Sound_Player no longer crackles on Windows. (#84, @someone)

## [0.1.0] - 2026-01-01

An old release whose notes are far too long for the rules, which only apply to notes not yet released. One two three.
"""


class Notes(unittest.TestCase):
    def test_a_short_entry_passes(self):
        self.assertEqual(cc.problems(GOOD), [])

    def test_no_unreleased_section_passes(self):
        self.assertEqual(cc.problems("# Changelog\n\n## [0.1.0] - 2026-01-01\n\nOld.\n"), [])

    def test_a_long_bullet_fails(self):
        long = "- " + " ".join(["word"] * 21) + "\n"
        text = GOOD.replace("- Sound_Player no longer crackles on Windows. (#84, @someone)\n", long)
        self.assertEqual(cc.problems(text), ["Fixes: a bullet has 21 words, the limit is 20: - word word word word word ..."])

    def test_links_and_attribution_do_not_count(self):
        bullet = "- " + " ".join(["word"] * 19) + " [x](docs/a/very/long/path.md). (#1, #2, @a, @b)\n"
        text = GOOD.replace("- Sound_Player no longer crackles on Windows. (#84, @someone)\n", bullet)
        self.assertEqual(cc.problems(text), [])

    def test_images_do_not_count(self):
        table = '<table>\n<tr><td><img src="https://example.com/a.svg" alt="one two three four five six"></td></tr>\n</table>\n'
        image = "![seven eight nine ten eleven twelve](https://example.com/b.svg)\n"
        text = GOOD.replace("Audio no longer crackles.\n", "Audio no longer crackles.\n\n" + table + "\n" + image)
        self.assertEqual(cc.problems(text), [])

    def test_summary_limits(self):
        text = GOOD.replace("Jacket builds for new targets. Audio no longer crackles.", "One. Two. Three.")
        self.assertEqual(cc.problems(text), ["the summary has 3 sentences, the limit is 2"])
        self.assertEqual(cc.problems(GOOD.replace("Jacket builds for new targets. Audio no longer crackles.\n\n", "")),
                         ["the summary is missing: one or two sentences before the first ### heading"])

    def test_headings_and_highlight_count(self):
        text = GOOD.replace("### Fixes", "### Fixed")
        self.assertEqual(cc.problems(text), ["unknown section `### Fixed`: use Highlights, Breaking changes or Fixes, in that order"])
        many = "".join(f"- Highlight {n}.\n" for n in range(7))
        text = GOOD.replace("- New targets: WebAssembly and ARMv7 Linux ([targets](docs/compiler/targets.md)).\n", many)
        self.assertEqual(cc.problems(text), ["Highlights has 7 bullets, the limit is 6"])

    def test_a_long_table_cell_fails(self):
        text = GOOD.replace("| `size()` |", "| " + " ".join(["w"] * 16) + " |")
        self.assertEqual(cc.problems(text), ["Breaking changes: a table cell has 16 words, the limit is 15: w w w w w w ..."])


if __name__ == "__main__":
    unittest.main()
