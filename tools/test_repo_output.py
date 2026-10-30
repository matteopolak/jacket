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


if __name__ == '__main__':
    unittest.main()
