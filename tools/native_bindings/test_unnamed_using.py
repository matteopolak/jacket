"""Retain reviewed unnamed using edges without inventing a named/as field."""

import unittest

from generate import generate_facade
from spec import parse_spec


class UnnamedUsing(unittest.TestCase):
    def test_member_inventory_preserves_first_unnamed_base(self):
        page = (
            "## Public declaration inventory\n- `Child :: struct`\n"
            "## Public type members\n#### `Child` (struct)\n"
            "- `using Parent;`\n- `value: u64;`\n"
        )
        parsed = parse_spec(page)
        self.assertEqual(
            parsed.shapes[0].code,
            "Child :: struct {\n    using Parent;\n    value: u64;\n}",
        )
        generated, _ = generate_facade("Metal", parsed)
        self.assertIn("using Parent;", generated)
        self.assertLess(
            generated.index("using Parent;"), generated.index("value: u64;")
        )
        self.assertNotIn("#as", generated)
        self.assertNotIn("parent:", generated)


if __name__ == "__main__":
    unittest.main()
