import unittest

import sizes

SOURCE = '''// comment :: () { not code }
Vec :: struct { x: int; }
Kind :: enum { a; b; }

first :: (v: Vec) -> int {
    s := "a { fake } :: () {";
    return v.x;
}

second :: () {
}
'''


class SizeTests(unittest.TestCase):
    def test_counts_code_lines_procs_and_types(self):
        facts = sizes.facts_of_text(SOURCE)
        self.assertEqual((facts.procs, facts.structs), (2, 2))
        self.assertEqual(facts.lines, 8)  # blank lines and the comment-only line are not code
        self.assertEqual(facts.files, 1)

    def test_per_kloc(self):
        self.assertEqual(sizes.SourceFacts(lines=2000).per_kloc(1.0), 0.5)
        self.assertIsNone(sizes.SourceFacts().per_kloc(1.0))


if __name__ == '__main__':
    unittest.main()
