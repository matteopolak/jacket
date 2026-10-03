import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from inventory_project_requirements import scan


class ProjectRequirementsTests(unittest.TestCase):
    def fixture(self, root, text):
        (root/'corpus').mkdir()
        (root/'reference').mkdir()
        base=root/'corpus/upstream/example--project'
        base.mkdir(parents=True)
        path=base/'main.jai'
        path.write_text(text)
        (root/'corpus/reference-inputs.json').write_text(json.dumps({'format':1,'files':[]}))
        (root/'corpus/upstreams.json').write_text(json.dumps({'format':1,'projects':[{
            'repository':'example/project','revision':'pinned','files':[{'path':'main.jai','sha256':hashlib.sha256(text.encode()).hexdigest()}]}]}))
        return path

    def test_only_uncommented_metadata_and_explicit_unverified_requirements(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            self.fixture(root,'// #import "Fake"; compiler_fake();\n#module_parameters(N:=3);\nX :: #import,file "real.jai";\nlib :: #library,system "trusted-name";\ncompiler_create_workspace("small");\n')
            report=scan(root)
            self.assertEqual(report['inventory'],1)
            self.assertEqual(set(report['import_targets']),{'real.jai'})
            self.assertEqual(set(report['library_targets']),{'trusted-name'})
            self.assertEqual(set(report['compiler_identifiers']),{'compiler_create_workspace'})
            row=report['requirements']['compiler-workspaces-and-message-lifecycle']
            self.assertEqual(row['files'],1)
            self.assertEqual(row['samples'][0]['line'],5)
            self.assertEqual(row['acceptance'],'unverified full requirement')
            self.assertNotIn('compiler_fake',json.dumps(report))

    def test_optional_simd_evidence_is_pinned_and_never_native_acceptance(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            path=self.fixture(root,'main :: () {}')
            (root/'artifacts').mkdir()
            evidence={'scope':'static source only','supported_instruction_plan':['movups.x'],
                      'staged_target_contract':'x86_64','remaining_source_contracts':['bsf'],
                      'sources':[{'path':path.relative_to(root).as_posix(),
                                  'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}]}
            manifest=root/'artifacts/simd-source-inventory.json'
            manifest.write_text(json.dumps(evidence))
            actual=scan(root)['actual_simd_evidence']
            self.assertEqual(actual['owner'],'llvm_types_high')
            self.assertIn('not source-to-native acceptance',actual['acceptance'])
            evidence['sources'][0]['sha256']='unverified'
            manifest.write_text(json.dumps(evidence))
            with self.assertRaises(ValueError): scan(root)

    def test_source_drift_is_a_blocker(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            self.fixture(root,'main :: () {}').write_text('modified')
            with self.assertRaises(ValueError): scan(root)

if __name__ == '__main__':
    unittest.main()
