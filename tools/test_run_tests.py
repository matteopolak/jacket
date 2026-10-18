import unittest

import run_tests


class Discovery(unittest.TestCase):
    def test_every_tools_folder_with_tests_is_discovered(self):
        names = {folder.relative_to(run_tests.TOOLS).as_posix() for folder in run_tests.folders()}
        expected = {p.parent.relative_to(run_tests.TOOLS).as_posix() for p in run_tests.TOOLS.rglob("test_*.py")}
        self.assertTrue(expected - {"."} <= names)
        self.assertTrue({"native_bindings", "numerics", "ci"} <= names)

    def test_a_pattern_narrows_the_folders(self):
        names = {folder.name for folder in run_tests.folders("test_bump_version.py")}
        self.assertEqual(names, {"tools"})

    def test_hand_run_scripts_do_not_match_the_test_pattern(self):
        self.assertFalse(list(run_tests.TOOLS.rglob("test_sdl3.py")))
        self.assertFalse(list(run_tests.TOOLS.rglob("test_webgpu_native.py")))


if __name__ == "__main__":
    unittest.main()
