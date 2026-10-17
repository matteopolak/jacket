import unittest

import ci_changes


class Matching(unittest.TestCase):
    def test_globs(self):
        self.assertTrue(ci_changes.matches(["stdlib/Extensions/SDL3/**"], ["stdlib/Extensions/SDL3/a/b.jai"]))
        self.assertFalse(ci_changes.matches(["tools/sdl3*.py"], ["tools/native/sdl3/x.py"]))
        self.assertTrue(ci_changes.matches(["nix/**"], [".github/workflows/ci.yml"]))
        self.assertFalse(ci_changes.matches(["nix/**"], ["stdlib/Basic/module.jai"]))


if __name__ == "__main__":
    unittest.main()
