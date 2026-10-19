import unittest
from rust_item_spacing import space_items


class ItemSpacingTests(unittest.TestCase):
    def test_use_run_stays_together_after_a_multi_line_inner_attribute(self):
        src = (
            "#![expect(\n"
            "    clippy::too_many_arguments,\n"
            '    reason = "mirrors the C API"\n'
            ")]\n"
            "\n"
            "use a::B;\n"
            "use c::D;\n"
        )
        self.assertEqual(space_items(src), src)

    def test_use_run_after_single_line_inner_attribute(self):
        src = "#![allow(unsafe_code)]\n\nuse a::B;\nuse c::D;\n"
        self.assertEqual(space_items(src), src)

    def test_multi_line_items_are_separated(self):
        src = "fn a() {\n}\nfn b() {\n}\n"
        self.assertEqual(space_items(src), "fn a() {\n}\n\nfn b() {\n}\n")


if __name__ == "__main__":
    unittest.main()
