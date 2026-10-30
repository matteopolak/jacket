import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bench import workloads


class CopyInTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name).resolve()
        self.addCleanup(self.temp.cleanup)
        self.corpus = self.base / 'corpus' / 'upstream'
        (self.corpus / 'lib').mkdir(parents=True)
        (self.corpus / 'lib' / 'lib.jai').write_text('lib')
        (self.corpus / 'app' / 'modules').mkdir(parents=True)
        (self.corpus / 'app' / 'main.jai').write_text('main')

    def test_a_link_to_a_sibling_is_copied_as_its_contents(self):
        os.symlink('../../lib', self.corpus / 'app' / 'modules' / 'lib')
        copy = workloads.copy_in(self.corpus / 'app', self.base / 'scratch')
        self.assertEqual((copy / 'modules' / 'lib' / 'lib.jai').read_text(), 'lib')

    def test_an_absolute_link_from_another_machine_is_found_in_this_corpus(self):
        os.symlink('C:/Users/someone/projects/jai/corpus/upstream/lib', self.corpus / 'app' / 'modules' / 'lib')
        with mock.patch.object(workloads, 'CORPUS', self.corpus):
            copy = workloads.copy_in(self.corpus / 'app', self.base / 'scratch')
        self.assertEqual((copy / 'modules' / 'lib' / 'lib.jai').read_text(), 'lib')

    def test_a_link_that_leads_nowhere_says_so(self):
        os.symlink('/gone/elsewhere/lib', self.corpus / 'app' / 'modules' / 'lib')
        with mock.patch.object(workloads, 'CORPUS', self.corpus), self.assertRaisesRegex(FileNotFoundError, 'fetch_upstreams'):
            workloads.copy_in(self.corpus / 'app', self.base / 'scratch')


if __name__ == '__main__':
    unittest.main()
