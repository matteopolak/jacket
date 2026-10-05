import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from fetch_upstreams import DEPENDENCIES, LIBRARIES, REPOSITORIES, safe_path
from verify_upstreams import verify

class CorpusTests(unittest.TestCase):
    def test_paths_cannot_escape_root(self):
        for path in ['../bad.jai', '/bad.jai', 'a/../../bad.jai', '.']:
            with self.subTest(path=path), self.assertRaises(ValueError): safe_path(path)
        self.assertEqual(str(safe_path('examples/hello.jai')), 'examples/hello.jai')

    def fixture(self, root):
        data = b'main :: () {}'
        projects = []
        for repo in REPOSITORIES + DEPENDENCIES + LIBRARIES:
            destination = root / 'corpus/upstream' / repo.replace('/', '--')
            destination.mkdir(parents=True)
            (destination / 'main.jai').write_bytes(data)
            projects.append({'repository':repo, 'revision':'1'*40, 'latest_jai_change':'2026-01-01T00:00:00+00:00',
                'selection':'recent-source', 'files':[{'path':'main.jai','bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}]})
        (root / 'corpus/upstreams.json').write_text(json.dumps({'format':1,'minimum_source_date':'2025-10-01T00:00:00+00:00','projects':projects}))
        return root / 'corpus/upstream' / REPOSITORIES[0].replace('/', '--')

    def test_changed_inputs_are_rejected_and_unlisted_reported(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); project = self.fixture(root)
            self.assertEqual(verify(root), (len(REPOSITORIES + DEPENDENCIES + LIBRARIES),) * 2 + ([],))
            (project / 'main.jai').write_bytes(b'modified')
            with self.assertRaises(ValueError): verify(root)
        with TemporaryDirectory() as directory:
            root = Path(directory); project = self.fixture(root)
            (project / 'extra.jai').write_text('unlisted')
            self.assertEqual(verify(root)[2], [project.name + '/extra.jai'])
            (project / 'main.jai').unlink()
            with self.assertRaises(FileNotFoundError): verify(root)

    def test_stale_selection_is_rejected(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); self.fixture(root)
            path = root / 'corpus/upstreams.json'; manifest = json.loads(path.read_text())
            manifest['projects'][0]['latest_jai_change'] = '2020-01-01T00:00:00+00:00'
            path.write_text(json.dumps(manifest))
            with self.assertRaises(ValueError): verify(root)

if __name__ == '__main__': unittest.main()
