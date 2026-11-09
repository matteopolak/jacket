import tempfile
import unittest
from pathlib import Path
from unittest import mock

import repo_output


class MainCheckoutTests(unittest.TestCase):
    def test_a_checkout_without_version_control_is_its_own_main(self):
        with tempfile.TemporaryDirectory() as folder:
            with mock.patch.object(repo_output, 'ROOT', Path(folder)), \
                    mock.patch('subprocess.run', side_effect=FileNotFoundError('git')):
                self.assertEqual(repo_output.main_checkout(), Path(folder))

    def test_a_linked_worktree_resolves_to_the_main_checkout(self):
        with tempfile.TemporaryDirectory() as folder:
            main = Path(folder).resolve() / 'main'
            admin = main / '.git' / 'worktrees' / 'w'
            admin.mkdir(parents=True)
            (admin / 'commondir').write_text('../..\n')
            tree = Path(folder).resolve() / 'w'
            tree.mkdir()
            (tree / '.git').write_text(f'gitdir: {admin}\n')
            with mock.patch.object(repo_output, 'ROOT', tree):
                self.assertEqual(repo_output.main_checkout(), main)


class OverrideTests(unittest.TestCase):
    def test_a_relative_output_directory_is_below_the_checkout_not_the_cwd(self):
        with tempfile.TemporaryDirectory() as folder, \
                mock.patch.dict('os.environ', {'JACKET_OUTPUT_DIR': 'out/x'}), \
                mock.patch.object(repo_output, 'ROOT', Path(folder)):
            self.assertEqual(repo_output.output_dir('a'), Path(folder).resolve() / 'out' / 'x' / 'a')
            self.assertEqual(repo_output.shared_output_dir(), Path(folder).resolve() / 'out' / 'x')

    def test_an_absolute_output_directory_is_used_as_given(self):
        with tempfile.TemporaryDirectory() as folder, \
                mock.patch.dict('os.environ', {'JACKET_OUTPUT_DIR': folder}):
            self.assertEqual(repo_output.output_dir('a'), Path(folder).resolve() / 'a')


if __name__ == '__main__':
    unittest.main()
