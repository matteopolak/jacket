"""Public alias generation preserves existing owners and canonical targets."""

import unittest

from generate_appkit import render_manual_aliases


class PublicAliases(unittest.TestCase):
    spec = "The `Child` aggregate also publicly aliases `Parent.scale`."

    def test_alias_preserves_body_and_ignores_braces_in_literals(self):
        source = 'Child :: struct {\n    text :: "}";\n    run :: () { /* } */ }\n}\n'
        expected = source[:-2] + "    scale :: Parent.scale;\n}\n"
        self.assertEqual(render_manual_aliases(self.spec, source), expected)

    def test_exact_alias_is_idempotent(self):
        source = "Child :: struct {\n    scale :: Parent.scale;\n}\n"
        self.assertEqual(render_manual_aliases(self.spec, source), source)

    def test_conflicting_alias_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Conflicting public alias"):
            render_manual_aliases(
                self.spec, "Child :: struct {\n    scale :: Other.scale;\n}\n"
            )

    def test_missing_owner_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Missing public alias owner"):
            render_manual_aliases(self.spec, "Other :: struct {}\n")

    def test_multiple_owners_remain_separate(self):
        spec = (
            self.spec + " The `Other` aggregate also publicly aliases `Parent.width`."
        )
        source = "Child :: struct {\n}\nOther :: struct {\n}\n"
        expected = "Child :: struct {\n    scale :: Parent.scale;\n}\nOther :: struct {\n    width :: Parent.width;\n}\n"
        self.assertEqual(render_manual_aliases(spec, source), expected)


if __name__ == "__main__":
    unittest.main()
