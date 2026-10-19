import unittest

import check_diagnostics_covered as covered


class Extraction(unittest.TestCase):
    def test_plain_and_formatted_messages(self):
        src = 'return err(span, "bad thing"); Diagnostic::error(s, format!("no `{}` in {}", a, b))'
        self.assertEqual(covered.templates(src), ["bad thing", "no `{}` in {}"])

    def test_the_message_is_found_after_other_arguments(self):
        self.assertEqual(covered.templates('self.err(start, "invalid float literal")'), ["invalid float literal"])

    def test_calls_without_a_literal_message_are_skipped(self):
        self.assertEqual(covered.templates("err(span, msg); x.map_err(|e| e)"), [])

    def test_escaped_braces_stay_literal_and_continuations_join(self):
        self.assertEqual(covered.templates('err(s, format!("a {{}} b \\\n      c {x}"))'), ["a {} b c {}"])

    def test_commas_and_parentheses_inside_strings_do_not_split_arguments(self):
        self.assertEqual(covered.templates('err(f(a, b), "one, (two")'), ["one, (two"])

    def test_test_modules_are_cut_off(self):
        production, tests = covered.strip_test_modules("fn a() {}\n#[cfg(test)]\nmod tests {\n}\n")
        self.assertEqual(production, "fn a() {}\n")
        self.assertIn("mod tests", tests)


class Matching(unittest.TestCase):
    def test_placeholders_match_text_on_one_line(self):
        self.assertTrue(covered.pattern("no member `{}` in {}").search("error: no member `x` in Foo"))
        self.assertIsNone(covered.pattern("no member `{}` in {}").search("no member `x`\nin Foo"))

    def test_templates_without_enough_text_never_match(self):
        self.assertIsNone(covered.pattern("{}: {}"))

    def test_untested_lists_what_no_text_expects(self):
        self.assertEqual(covered.untested({"bad thing {}", "other thing"}, "error: bad thing 3\n"), {"other thing"})


if __name__ == "__main__":
    unittest.main()
