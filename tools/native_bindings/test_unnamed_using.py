"""Retain reviewed unnamed using edges without inventing a named/as field."""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import unittest

from generate import generate_facade
from native_bindings.spec import parse_spec


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
