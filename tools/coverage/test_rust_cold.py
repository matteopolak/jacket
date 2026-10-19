import unittest

import rust_cold

LCOV = """SF:/w/crates/jaic/src/sema/a.rs
FN:3,_ZN4sema1a3fooE
FN:20,_ZN4sema1a3barE
FNDA:5,_ZN4sema1a3fooE
FNDA:0,_ZN4sema1a3barE
DA:3,5
DA:4,0
DA:20,0
DA:21,0
end_of_record
SF:/w/crates/jaic/src/interp/b.rs
DA:1,1
DA:2,1
end_of_record
SF:/w/crates/other/src/c.rs
DA:1,0
end_of_record
"""
ROOTS = ["crates/jaic/src"]


class Cold(unittest.TestCase):
    def test_areas_are_sorted_coldest_first(self):
        rows = rust_cold.area_table(rust_cold.parse(LCOV), ROOTS)
        self.assertEqual(rows, [("crates/jaic/src/sema", 1, 4), ("crates/jaic/src/interp", 2, 2)])

    def test_files_outside_the_roots_are_ignored(self):
        rows = rust_cold.cold_files(rust_cold.parse(LCOV), ROOTS, 10)
        self.assertEqual([r[2] for r in rows], ["/w/crates/jaic/src/sema/a.rs", "/w/crates/jaic/src/interp/b.rs"])

    def test_unrun_functions_are_listed_with_their_length(self):
        rows = rust_cold.cold_functions(rust_cold.parse(LCOV), ROOTS, 10)
        self.assertEqual([(r[0], r[2]) for r in rows], [(2, 20)])


if __name__ == "__main__":
    unittest.main()
