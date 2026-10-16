"""Reviewed field fences supplement a same-target member inventory."""

import unittest

from spec import parse_spec


class FieldFragments(unittest.TestCase):
    def page(
        self, field="using #as base: Parent;", member="run :: (self: *Child) -> () {"
    ):
        return (
            "## Public declaration inventory\n- `Child :: struct`\n"
            "## Public type members\n```jai\nChild :: struct {\n"
            + field
            + "\n}\n```\n#### `Child` (struct)\n- `"
            + member
            + "`\n"
        )

    def test_first_field_and_methods_preserved_once(self):
        shapes = parse_spec(self.page()).shapes
        self.assertEqual(len(shapes), 1)
        self.assertLess(
            shapes[0].code.index("using #as base"), shapes[0].code.index("run ::")
        )
        self.assertEqual(shapes[0].code.count("base: Parent"), 1)

    def test_duplicate_field_is_not_duplicated(self):
        shapes = parse_spec(self.page(member="using #as base: Parent")).shapes
        self.assertEqual(len(shapes), 1)
        self.assertEqual(shapes[0].code.count("base: Parent"), 1)

    def test_conflicting_field_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Conflicting reviewed field"):
            parse_spec(self.page(member="base: Other"))

    def test_distinct_target_fragments_are_not_merged(self):
        text = self.page().replace(
            "#### `Child` (struct)", "#### `Child` [OS == .LINUX] (struct)"
        )
        self.assertEqual(len(parse_spec(text).shapes), 2)

    def test_anonymous_union_is_not_flattened(self):
        shapes = parse_spec(self.page(field="using union { a: u32; b: u64; }")).shapes
        self.assertEqual(len(shapes), 2)


if __name__ == "__main__":
    unittest.main()
