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


def problems(source, name="tools/x/script.py"):
    return style.build_path_problems(source, name)


class BuildPaths(unittest.TestCase):
    def test_reads_of_the_variable_are_flagged(self):
        for source in (
            'import os\nx = os.environ.get("CARGO_TARGET_DIR")',
            'import os\nx = os.environ.get("CARGO_TARGET_DIR", "t")',
            'import os\nx = os.getenv("CARGO_TARGET_DIR")',
            'import os\nx = os.environ["CARGO_TARGET_DIR"]',
            'import os\nx = "CARGO_TARGET_DIR" in os.environ',
            'x = env.get("CARGO_TARGET_DIR")',
        ):
            with self.subTest(source=source):
                self.assertEqual(len(problems(source)), 1)

    def test_setting_the_variable_is_fine(self):
        for source in (
            'env["CARGO_TARGET_DIR"] = "t"',
            'env = dict(os.environ, CARGO_TARGET_DIR="t")',
            'patch.dict(os.environ, {"CARGO_TARGET_DIR": "t"})',
        ):
            with self.subTest(source=source):
                self.assertEqual(problems(source), [])

    def test_hand_built_target_paths_are_flagged(self):
        for source in (
            'x = ROOT / "target"',
            'x = ROOT / "target/debug/jaic"',
            'x = ROOT / "target" / "debug"',
            'x = ROOT / f"target/{name}"',
            'x = toolkit.ROOT / "target"',
        ):
            with self.subTest(source=source):
                self.assertEqual(len(problems(source)), 1)

    def test_other_paths_are_fine(self):
        for source in (
            'x = ROOT / "stdlib"',
            'x = ROOT / "targets.json"',
            'x = root / "target"',
            'x = BENCH / "target"',
            'x = cargo_target_dir() / "debug"',
        ):
            with self.subTest(source=source):
                self.assertEqual(problems(source), [])

    def test_allow_listed_files_are_skipped(self):
        self.assertEqual(problems('x = ROOT / "target"', "tools/repo_output.py"), [])

    def test_allow_list_names_real_files(self):
        for name in style.BUILD_PATH_ALLOWED:
            self.assertTrue((style.ROOT / name).is_file(), name)


class Banners(unittest.TestCase):
    def test_rule_only_comments_are_banners(self):
        self.assertEqual(
            len(style.check_banners("x.rs", "// ------\n// Layout\n  // ======\n")), 2
        )
        self.assertEqual(len(style.check_banners("x.py", "# ----\n")), 1)

    def test_plain_comments_and_markdown_pass(self):
        self.assertEqual(
            style.check_banners("x.rs", "// Layout\n/// ---\n//! ---\n// a - b\n"), []
        )


if __name__ == "__main__":
    unittest.main()
