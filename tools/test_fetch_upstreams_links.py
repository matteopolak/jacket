import errno
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import fetch_upstreams


class LinkFolderTests(unittest.TestCase):
    def test_a_relative_symlink_is_made_where_allowed(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder).resolve()
            (base / 'target').mkdir()
            (base / 'sub').mkdir()
            fetch_upstreams.link_folder(base / 'sub' / 'link', base / 'target')
            self.assertFalse(os.path.isabs(os.readlink(base / 'sub' / 'link')))

    def test_without_the_symlink_privilege_a_copy_is_made_not_an_absolute_junction(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder).resolve()
            (base / 'target').mkdir()
            (base / 'target' / 'a.jai').write_text('a')
            error = OSError(errno.EPERM, 'privilege')
            error.winerror = fetch_upstreams.ERROR_PRIVILEGE_NOT_HELD
            with mock.patch.object(Path, 'symlink_to', side_effect=error):
                fetch_upstreams.link_folder(base / 'link', base / 'target')
                fetch_upstreams.link_folder(base / 'link', base / 'target')  # again over the copy
            self.assertFalse((base / 'link').is_symlink())
            self.assertEqual((base / 'link' / 'a.jai').read_text(), 'a')


if __name__ == '__main__':
    unittest.main()
