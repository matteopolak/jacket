import unittest

import ci_changes


class Matching(unittest.TestCase):
    def test_globs(self):
        self.assertTrue(ci_changes.matches(["stdlib/Extensions/SDL3/**"], ["stdlib/Extensions/SDL3/a/b.jai"]))
        self.assertFalse(ci_changes.matches(["tools/sdl3*.py"], ["tools/native/sdl3/x.py"]))
        self.assertTrue(ci_changes.matches(["nix/**"], [".github/workflows/ci.yml"]))
        self.assertFalse(ci_changes.matches(["nix/**"], ["stdlib/Basic/module.jai"]))

    def test_docs_only_changes_skip_code(self):
        self.assertFalse(ci_changes.code_changed(["docs/tools/cli.md", "README.md", "docs/images/a.png"]))
        self.assertTrue(ci_changes.code_changed(["docs/tools/cli.md", "tools/ci/ci_changes.py"]))
        # Docs that a generator or test reads, and Markdown fixtures, count as code.
        self.assertTrue(ci_changes.code_changed(["docs/specs/stdlib/modules/socket.md"]))
        self.assertTrue(ci_changes.code_changed(["docs/tools/jailint.md"]))
        self.assertTrue(ci_changes.code_changed(["stdlib/Basic/tests/fixtures/notes.md"]))


if __name__ == "__main__":
    unittest.main()
