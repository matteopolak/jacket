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


class SetupToolTests(unittest.TestCase):
    def forbear(self):
        project = workloads.CorpusProject('gabrielmfern--forbear', ('forbear-build',))
        return project

    def test_a_workload_whose_setup_tools_are_missing_is_unavailable_with_the_reason(self):
        with tempfile.TemporaryDirectory() as folder:
            project = self.forbear()
            with mock.patch.object(workloads, 'CORPUS', Path(folder)), mock.patch('shutil.which', return_value=None):
                (Path(folder) / project.project).mkdir()
                reason = project.available()
        self.assertIn('setup needs', reason)
        for tool in ('sh', 'clang', 'ar'):
            self.assertIn(tool, reason)

    def test_materialize_survives_a_setup_program_that_cannot_start(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            (base / 'upstream' / 'gabrielmfern--forbear').mkdir(parents=True)
            project = self.forbear()
            with mock.patch.object(workloads, 'CORPUS', base / 'upstream'), \
                    mock.patch('subprocess.run', side_effect=FileNotFoundError('sh')):
                self.assertTrue(project.materialize(base / 'scratch').is_dir())

    def test_tools_that_are_installed_leave_it_available(self):
        with tempfile.TemporaryDirectory() as folder:
            project = self.forbear()
            with mock.patch.object(workloads, 'CORPUS', Path(folder)), mock.patch('shutil.which', return_value='/bin/x'):
                (Path(folder) / project.project).mkdir()
                self.assertEqual(project.available(), '')


if __name__ == '__main__':
    unittest.main()
