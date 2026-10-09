import re
import unittest

import benchgen
import jaistats


class BenchGenTests(unittest.TestCase):
    def test_same_seed_gives_the_same_program_and_other_seeds_differ(self):
        a = benchgen.generate(3000, 7)['main.jai']
        self.assertEqual(a, benchgen.generate(3000, 7)['main.jai'])
        self.assertNotEqual(a, benchgen.generate(3000, 8)['main.jai'])

    def test_size_is_close_to_the_request(self):
        for lines in (2000, 8000):
            count = benchgen.generate(lines, 1)['main.jai'].count('\n')
            self.assertTrue(0.9 * lines < count < 1.15 * lines, (lines, count))

    def test_only_shared_modules_are_imported_and_main_prints_a_checksum(self):
        text = benchgen.generate(2000, 3)['main.jai']
        imports = set(re.findall(r'#import "(\w+)"', text))
        self.assertEqual(imports, {'Basic', 'Math', 'String', 'Hash_Table'})
        self.assertIn('main :: () {', text)
        self.assertIn('print("checksum %\\n", acc);', text)

    def test_every_generated_procedure_is_called(self):
        text = benchgen.generate(3000, 5)['main.jai']
        defined = re.findall(r'(?m)^((?:update|compute|process|build|scan|merge|reduce|fold|check|find|apply|collect|'
                             r'resolve|emit|visit|parse|step|mix|pack|walk)_\w+) :: \(', text)
        self.assertTrue(defined)
        for name in defined:
            self.assertGreaterEqual(len(re.findall(r"\b" + name + r"\(", text)), 1, name)

    def test_split_output_loads_every_part(self):
        files = benchgen.generate(4000, 2, files=4)
        self.assertEqual(sorted(files), ['main.jai', 'part_1.jai', 'part_2.jai', 'part_3.jai'])
        for name in files:
            if name != 'main.jai':
                self.assertIn(f'#load "{name}";', files['main.jai'])

    def test_trace_prints_after_each_driver_call(self):
        self.assertIn('print("', benchgen.generate(1500, 1, trace=True)['main.jai'].split('drive_')[1])
        self.assertNotIn('print("update', benchgen.generate(1500, 1)['main.jai'])


class StatsTests(unittest.TestCase):
    def test_blank_hides_comments_and_strings(self):
        text = 'a :: 1; // if for\n/* while /* nested */ still */ b := "if \\" for";\n'
        out = jaistats.blank(text)
        self.assertNotRegex(out, r'\b(if|for|while)\b')
        self.assertEqual(out.count('\n'), text.count('\n'))

    def test_quantiles_and_histogram(self):
        self.assertEqual(jaistats.quantiles([1, 2, 3, 4])['p50'], 3)
        self.assertEqual(jaistats.histogram([1, 1, 5, 9], [1, 5]), [0.5, 0.5])


if __name__ == '__main__':
    unittest.main()
