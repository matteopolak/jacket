import tempfile
import unittest
from pathlib import Path

import module_swap
import phases
from cmd_compare import Observed, compare_outputs


class ModuleFolder(unittest.TestCase):
    def test_links_the_named_modules_and_the_shared_folder(self):
        with tempfile.TemporaryDirectory() as tmp:
            source, dest = Path(tmp, 'other'), Path(tmp, 'swap')
            for name in ('Basic', 'Sort', 'internal/sort'):
                (source / name).mkdir(parents=True)
            (source / 'Single.jai').write_text('')
            linked = module_swap.module_folder(source, dest, ['Sort', 'Single'])
            self.assertEqual(sorted(linked), ['Single.jai', 'Sort', 'internal'])
            self.assertEqual((dest / 'Sort').resolve(), (source / 'Sort').resolve())
            self.assertFalse((dest / 'Basic').exists())  # not named: the compiler's own is used

    def test_a_missing_module_is_an_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(SystemExit):
                module_swap.module_folder(Path(tmp), Path(tmp, 'swap'), ['Sort'])


class Settings(unittest.TestCase):
    def test_module_dirs_go_first_in_the_search_path(self):
        source = phases.program(Path('/w/main.jai'), Path('/w/out'), writes_executable=True, optimized=False,
                                run_main=False, import_dirs=('/w/extra',), module_dirs=('/w/swap',))
        self.assertIn('MODULE_DIRS :: string.["/w/swap"];', source)
        self.assertIn('IMPORT_DIRS :: string.["/w/extra"];', source)


class Quoting(unittest.TestCase):
    def test_unquoted_reasons_name_no_output(self):
        a, b = Observed(0, ['same']), Observed(0, ['other line', 'more'])
        self.assertIn('other line', compare_outputs(a, b, True))
        self.assertEqual(compare_outputs(a, b, True, quote=False), 'compiler output differs: 1 lines vs 2 lines')


if __name__ == '__main__':
    unittest.main()
