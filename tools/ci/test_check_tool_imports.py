import unittest

import check_tool_imports as check


def problems(source, name="tools/x/script.py"):
    return check.build_path_problems(source, name)


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
        for name in check.BUILD_PATH_ALLOWED:
            self.assertTrue((check.ROOT / name).is_file(), name)


if __name__ == "__main__":
    unittest.main()
