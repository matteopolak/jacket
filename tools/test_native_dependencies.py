"""Hermetic provenance/target tests; no native compiler or library is executed."""
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import native_dependencies as native


class NativeDependencies(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def test_real_directives_preserve_options_but_ignore_strings_and_nested_comments(self):
        source = '''// #library "fake"
        /* outer /* #library "nested" */ */
        example := "#system_library \\"fake\\"";
        one :: #library,no_dll,link_always "mac/VkMemAlloc";
        two :: #library,system "c++";
        three :: #system_library "Foundation";
        '''
        rows = native.declarations(source)
        self.assertEqual([row.name for row in rows], ['mac/VkMemAlloc', 'c++', 'Foundation'])
        self.assertEqual(rows[0].options, ('no_dll', 'link_always'))
        self.assertEqual(rows[0].line, 4)
        self.assertEqual([row.kind for row in rows], [native.DependencyKind.LOCAL,
                                                     native.DependencyKind.SYSTEM, native.DependencyKind.SYSTEM])

    def test_corpus_manifest_rejects_source_changes_and_escaping_symlinks(self):
        source = self.write('corpus/upstream/example--repo/module.jai', 'lib :: #library "local";\n')
        manifest = {'format': 1, 'projects': [{'repository': 'example/repo', 'revision': 'a' * 40,
                    'files': [{'path': 'module.jai', 'sha256': native.sha256(source)}]}]}
        self.write('corpus/upstreams.json', json.dumps(manifest))
        self.assertEqual(native.corpus_evidence(self.root)[0]['declarations'][0]['name'], 'local')
        source.write_text('lib :: #library "changed";\n')
        with self.assertRaisesRegex(ValueError, 'pinned source'):
            native.corpus_evidence(self.root)
        source.unlink()
        source.symlink_to(self.write('outside.jai', 'lib :: #library "local";\n'))
        with self.assertRaisesRegex(ValueError, 'pinned source'):
            native.corpus_evidence(self.root)

    def test_protected_source_paths_and_symlinks_are_rejected_before_bytes_are_read(self):
        for base in ('reference', 'vendor', 'corpus/upstream'):
            protected = self.write(f'{base}/not-a-native-library.a', 'self-written sentinel')
            link = self.root / f'alias-{base.replace("/", "-")}'
            link.symlink_to(protected)
            with patch.object(native, 'sha256', side_effect=AssertionError('must not read bytes')):
                for candidate in (protected, link):
                    with self.assertRaisesRegex(ValueError, 'protected input'):
                        native.fingerprint(candidate, self.root)

    def receipt_fixture(self):
        source = self.write('source/vk_mem_alloc.h', '// self-written source fixture\n')
        compiler = self.write('tools/compiler', 'self-written inert fixture\n')
        archiver = self.write('tools/archiver', 'self-written inert fixture\n')
        artifact = self.write('artifacts/native-dependencies/build/fixture.a', 'inert fixture bytes\n')
        recipe = native.RebuildRecipe('example/source', 'b' * 40, 'fixture.h', native.sha256(source), 'https://example.invalid')
        receipt = {'format': 1, 'kind': 'source-rebuild-evidence', 'recipe': asdict(recipe),
                   'target': 'arm64-apple-darwin', 'compiler': asdict(native.fingerprint(compiler, self.root)),
                   'archiver': asdict(native.fingerprint(archiver, self.root)),
                   'inputs': [asdict(native.fingerprint(source, self.root))],
                   'artifact': asdict(native.fingerprint(artifact, self.root)),
                   'flags': list(native.VMA_FLAGS), 'link_authority': False,
                   'wrapper_sha256': hashlib.sha256(native.VMA_WRAPPER.encode()).hexdigest()}
        path = self.write('receipt.json', json.dumps(receipt))
        return recipe, receipt, path, source, artifact

    def test_receipt_is_bound_to_target_flags_source_tools_and_artifact(self):
        recipe, receipt, path, source, artifact = self.receipt_fixture()
        with patch.object(native, 'VMA', recipe):
            self.assertEqual(native.verify_receipt(path, 'arm64-apple-darwin', self.root).path, str(artifact.resolve()))
            with self.assertRaisesRegex(ValueError, 'target mismatch'):
                native.verify_receipt(path, 'x86_64-unknown-linux-gnu', self.root)
            for name in ('compiler', 'archiver', 'artifact'):
                candidate = Path(receipt[name]['path'])
                previous = candidate.read_text()
                candidate.write_text('changed inert bytes')
                with self.assertRaisesRegex(ValueError, 'fingerprint mismatch'):
                    native.verify_receipt(path, 'arm64-apple-darwin', self.root)
                candidate.write_text(previous)
            source.write_text('changed source')
            with self.assertRaisesRegex(ValueError, 'fingerprint mismatch'):
                native.verify_receipt(path, 'arm64-apple-darwin', self.root)

    def test_receipt_never_grants_link_authority(self):
        recipe, receipt, path, _, _ = self.receipt_fixture()
        receipt['link_authority'] = True
        path.write_text(json.dumps(receipt))
        with patch.object(native, 'VMA', recipe):
            with self.assertRaisesRegex(ValueError, 'configuration'):
                native.verify_receipt(path, 'arm64-apple-darwin', self.root)

    def test_receipt_flags_and_protected_artifact_cannot_be_substituted(self):
        recipe, receipt, path, _, artifact = self.receipt_fixture()
        with patch.object(native, 'VMA', recipe):
            receipt['flags'].append('-include-unreviewed')
            path.write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'configuration'):
                native.verify_receipt(path, 'arm64-apple-darwin', self.root)
            receipt['flags'] = list(native.VMA_FLAGS)
            receipt['artifact']['path'] = str(protected.resolve())
            path.write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'protected input'):
                native.verify_receipt(path, 'arm64-apple-darwin', self.root)

    def test_rebuild_rejects_unreviewed_source_before_subprocess(self):
        source = self.write('source/vk_mem_alloc.h', '// self-written unreviewed header')
        compiler = native.Fingerprint('/usr/bin/clang++', 'c' * 64)
        with patch.object(native, 'installed_tool', return_value=compiler), patch.object(native.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'reviewed official source pin'):
                native.build_vma(source, self.root / 'include', Path('/usr/bin/clang++'),
                                 Path('/usr/bin/ar'), self.root / 'artifacts/native-dependencies/build',
                                 'arm64-apple-darwin', self.root)
            run.assert_not_called()

    def test_build_environment_scrubs_injected_search_paths(self):
        with patch.dict(native.os.environ, {'CPATH': str(self.root / 'reference'),
                         'CPLUS_INCLUDE_PATH': 'unsafe', 'DYLD_INSERT_LIBRARIES': 'unsafe',
                         'SDKROOT': 'unsafe', 'DEVELOPER_DIR': 'unsafe', 'PATH': 'unsafe'}):
            environment = native.clean_environment()
        self.assertEqual(environment['PATH'], '/usr/bin:/bin:/opt/homebrew/bin')
        for name in ('CPATH', 'CPLUS_INCLUDE_PATH', 'DYLD_INSERT_LIBRARIES', 'SDKROOT', 'DEVELOPER_DIR'):
            self.assertNotIn(name, environment)


if __name__ == '__main__':
    unittest.main()
