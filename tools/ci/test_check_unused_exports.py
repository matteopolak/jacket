import unittest

from ci import check_unused_exports as check


def message(*parts):
    """A protobuf message from `(field, value)` pairs: ints as varints, bytes length-delimited."""
    out = b""
    for number, value in parts:
        if isinstance(value, int):
            out += _varint(number << 3) + _varint(value)
        else:
            out += _varint(number << 3 | 2) + _varint(len(value)) + value
    return out


def _varint(value):
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        out += bytes([byte | (0x80 if value else 0)])
        if not value:
            return out


def occurrence(line, symbol, roles):
    return message((1, bytes([line, 0, 3])), (2, symbol.encode()), (3, roles))


SYMBOL = "rust-analyzer cargo jaic 0.1.0 util/{}()."


def index(*occurrences, path="crates/jaic/src/util.rs"):
    document = message((1, path.encode()), *[(2, o) for o in occurrences])
    return message((2, document))


class ReadIndex(unittest.TestCase):
    def test_a_definition_and_a_reference_are_told_apart(self):
        data = index(
            occurrence(4, SYMBOL.format("used"), 1),
            occurrence(9, SYMBOL.format("used"), 0),
            occurrence(6, SYMBOL.format("lonely"), 1),
        )
        definitions, referenced = check.read_index(data)

        self.assertEqual(
            sorted(definitions), [SYMBOL.format("lonely"), SYMBOL.format("used")]
        )
        self.assertEqual(referenced, {SYMBOL.format("used")})


class Unused(unittest.TestCase):
    def found(self, text, line, *, referenced=()):
        symbol = SYMBOL.format("f")
        return check.unused(
            {symbol: ("crates/jaic/src/util.rs", line)},
            set(referenced) and {symbol},
            lambda path: text,
        )

    def test_an_unreferenced_pub_fn_is_reported(self):
        self.assertEqual(
            self.found("pub fn f() {}", 0), ["crates/jaic/src/util.rs:1: jaic/util/f"]
        )

    def test_a_referenced_one_is_not(self):
        self.assertEqual(self.found("pub fn f() {}", 0, referenced=[1]), [])

    def test_a_name_written_elsewhere_is_kept_because_the_index_misses_macro_uses(self):
        files = {"a.rs": "pub fn f() {}", "b.rs": "m!(f());"}
        found = check.unused(
            {SYMBOL.format("f"): ("crates/jaic/src/util.rs", 0)},
            set(),
            lambda path: "pub fn f() {}",
            files,
        )

        self.assertEqual(found, [])

    def test_a_name_written_only_at_its_definition_is_reported(self):
        files = {"a.rs": "pub fn f() {}", "b.rs": "fn fold() {}"}
        found = check.unused(
            {SYMBOL.format("f"): ("crates/jaic/src/util.rs", 0)},
            set(),
            lambda path: "pub fn f() {}",
            files,
        )

        self.assertEqual(found, ["crates/jaic/src/util.rs:1: jaic/util/f"])

    def test_a_crate_private_one_is_not(self):
        self.assertEqual(self.found("pub(crate) fn f() {}", 0), [])

    def test_code_other_programs_call_by_name_is_not(self):
        self.assertEqual(
            self.found('#[unsafe(no_mangle)]\npub extern "C" fn f() {}', 1), []
        )

    def test_a_symbol_of_a_member_is_not_an_item(self):
        self.assertIsNone(
            check.symbol_name("rust-analyzer cargo jaic 0.1.0 util/Type#method().")
        )


if __name__ == "__main__":
    unittest.main()
