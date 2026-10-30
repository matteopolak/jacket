import unittest

import check_diagnostics_covered as covered

TABLE = '''
catalog! {
    "types" => {
        TypeMismatch = "type-mismatch", Error,
            "a value of the wrong type";
        FormatMissingArgument = "format-missing-argument", Warning,
            "a format string with more `%` than arguments",
            explain = "text";
    }
}
'''


class Table(unittest.TestCase):
    def test_codes_are_read_in_order_with_their_variants(self):
        self.assertEqual(
            covered.catalog_codes(TABLE),
            [
                ("TypeMismatch", "type-mismatch"),
                ("FormatMissingArgument", "format-missing-argument"),
            ],
        )

    def test_test_modules_are_cut_off(self):
        production, tests = covered.strip_test_modules(
            "fn a() {}\n#[cfg(test)]\nmod tests {\n}\n"
        )
        self.assertEqual(production, "fn a() {}\n")
        self.assertIn("mod tests", tests)


class Matching(unittest.TestCase):
    codes = covered.catalog_codes(TABLE)

    def test_a_rendered_header_names_the_code(self):
        self.assertEqual(
            covered.untested(self.codes, "t.jai:3:4: error[type-mismatch]: bad\n"),
            {"format-missing-argument"},
        )

    def test_a_json_field_names_the_code(self):
        self.assertEqual(
            covered.untested(self.codes, '"code": "format-missing-argument"'),
            {"type-mismatch"},
        )

    def test_a_rust_test_names_the_variant(self):
        self.assertEqual(
            covered.untested(self.codes, "assert_eq!(d.code, Code::TypeMismatch);"),
            {"format-missing-argument"},
        )

    def test_a_longer_variant_does_not_count(self):
        self.assertEqual(
            covered.untested(self.codes, "Code::TypeMismatchExtra"),
            {"type-mismatch", "format-missing-argument"},
        )

    def test_the_message_alone_is_not_a_test(self):
        self.assertEqual(
            covered.untested(self.codes, "error: a value of the wrong type"),
            {"type-mismatch", "format-missing-argument"},
        )


if __name__ == "__main__":
    unittest.main()
