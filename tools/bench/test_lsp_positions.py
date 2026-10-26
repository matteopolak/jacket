

import unittest

import benchgen
from bench.lsp_positions import (delete_lines_edit, insert_edit, mask_line, pick_positions, replace_edit, typing_script,
                           utf16_col)

SOURCE = '''#import "Basic";

Vec :: struct { x: int; y: int; }

helper :: (v: Vec) -> int {
    // comment with ignored_word
    total := v.x + 1;
    s := "string.with.dots";
    return total * v.y;
}

second :: (n: int) -> int {
    out := n + helper(.{1, 2});
    log_it(out);
    return out;
}
'''


class PositionTests(unittest.TestCase):
    def test_masking_keeps_length_and_hides_comments_and_strings(self):
        line = 'a := "b.c"; // d.e'
        masked, _ = mask_line(line)
        self.assertEqual(len(masked), len(line))
        self.assertEqual(masked.rstrip(), 'a := "   ";')
        self.assertEqual(mask_line('x /* y', 0)[1], 1)
        self.assertEqual(mask_line('z */ w', 1), ('     w', 0))

    def test_utf16_columns_count_surrogate_pairs(self):
        self.assertEqual(utf16_col('a\U0001F600b', 2), 3)
        self.assertEqual(utf16_col('aéb', 2), 2)

    def test_picks_are_deterministic_and_inside_bodies(self):
        first = pick_positions(SOURCE)
        self.assertEqual(first, pick_positions(SOURCE))
        lines = SOURCE.split('\n')
        hover = first['hover']
        self.assertEqual(lines[hover['line']][hover['character']:][:len(hover['name'])], hover['name'])
        self.assertNotEqual(hover['name'], 'ignored_word')
        self.assertIn(first['declaration']['name'], ('helper', 'second'))
        member = first['member']
        self.assertEqual(lines[member['line']][member['character'] - 1], '.')
        self.assertNotIn('string', member['base'])

    def test_call_pick_is_just_after_the_parenthesis(self):
        pick = pick_positions(SOURCE)
        call = pick['call']
        line = SOURCE.split('\n')[call['line']]
        self.assertEqual(line[call['character'] - 1], '(')
        self.assertEqual(line[call['character'] - 1 - len(call['name']):call['character'] - 1], call['name'])

    def test_fraction_moves_the_pick_and_no_bodies_gives_none(self):
        early, late = pick_positions(SOURCE, 0.0), pick_positions(SOURCE, 0.99)
        self.assertLess(early['hover']['line'], late['hover']['line'])
        self.assertIsNone(pick_positions('x :: 1;\n'))

    def test_bodies_with_the_brace_on_the_next_line(self):
        pick = pick_positions('walk :: (node: *Node)\n{\n    visit_node(node.left);\n}\n')
        self.assertEqual(pick['declaration'], {'line': 0, 'character': 0, 'name': 'walk'})
        self.assertEqual(pick['hover']['line'], 2)

    def test_generated_programs_have_positions(self):
        pick = pick_positions(benchgen.generate(2000, 1)['main.jai'])
        self.assertIsNotNone(pick)
        self.assertIsNotNone(pick['member'])
        self.assertIsNotNone(pick['call'])

    def test_edits(self):
        self.assertEqual(insert_edit(3, 4, 'x')['range']['end'], {'line': 3, 'character': 4})
        self.assertEqual(delete_lines_edit(2)['range']['end'], {'line': 3, 'character': 0})
        replaced = replace_edit(1, 2, 3, 4, 'q')
        self.assertEqual((replaced['range']['start'], replaced['range']['end'], replaced['text']),
                         ({'line': 1, 'character': 2}, {'line': 3, 'character': 4}, 'q'))
        self.assertEqual(typing_script('  ', 'ab'), [(2, 'a'), (3, 'b')])


if __name__ == '__main__':
    unittest.main()
