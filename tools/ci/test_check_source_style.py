import unittest

import check_source_style as style


class Lengths(unittest.TestCase):
    def check(self, counts, allowed=None):
        return style.check_lengths(counts, allowed or {}, limit=10)

    def test_short_files_pass_and_long_files_fail(self):
        self.assertEqual(self.check({"a.rs": 10}), [])
        self.assertEqual(len(self.check({"a.rs": 11})), 1)

    def test_allowlisted_file_may_not_grow(self):
        self.assertEqual(self.check({"a.rs": 20}, {"a.rs": 20}), [])
        self.assertIn("grew", self.check({"a.rs": 21}, {"a.rs": 20})[0])

    def test_allowlist_entries_must_track_the_file(self):
        self.assertIn("lower", self.check({"a.rs": 15}, {"a.rs": 20})[0])
        self.assertIn("remove", self.check({"a.rs": 9}, {"a.rs": 20})[0])
        self.assertIn("gone", self.check({}, {"a.rs": 20})[0])

    def test_line_count(self):
        self.assertEqual(style.line_count("a\nb\n"), 2)
        self.assertEqual(style.line_count("a\nb"), 2)
        self.assertEqual(style.line_count(""), 0)


class Banners(unittest.TestCase):
    def test_rule_only_comments_are_banners(self):
        self.assertEqual(len(style.check_banners("x.rs", "// ------\n// Layout\n  // ======\n")), 2)
        self.assertEqual(len(style.check_banners("x.py", "# ----\n")), 1)

    def test_plain_comments_and_markdown_pass(self):
        self.assertEqual(style.check_banners("x.rs", "// Layout\n/// ---\n//! ---\n// a - b\n"), [])


if __name__ == "__main__":
    unittest.main()
