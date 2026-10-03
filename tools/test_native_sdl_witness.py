"""Hermetic SDL source/witness protocol validation without SDK execution."""
import json
import hashlib
from dataclasses import asdict
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import native_dependencies as native
import native_sdl_witness as sdl


class SdlWitnessTests(unittest.TestCase):
    def test_complete_bounded_witness_protocol(self):
        self.assertEqual(sdl.check_observed(sdl.EXPECTED, '', 0), sdl.EXPECTED)
        for output in (sdl.EXPECTED.replace('2 30 2', '2 32 70'),
                       sdl.EXPECTED.replace('SDL_Event 56 8', 'SDL_Event 64 8'),
                       sdl.EXPECTED + 'SDL_Event 56 8\n', sdl.EXPECTED[:-2]):
            with self.assertRaisesRegex(ValueError, 'witness failed'):
                sdl.check_observed(output, '', 0)
        with self.assertRaisesRegex(ValueError, 'witness failed'):
            sdl.check_observed(sdl.EXPECTED, 'unreviewed diagnostic', 0)
        with self.assertRaisesRegex(ValueError, 'witness failed'):
            sdl.check_observed(sdl.EXPECTED, '', 20)

    def test_reviewed_configuration_excludes_native_gui_and_external_loaders(self):
        self.assertIn('-DSDL_VIDEO=OFF', sdl.FLAGS)
        self.assertIn('-DSDL_AUDIO=OFF', sdl.FLAGS)
        self.assertIn('-DSDL_LOADSO=OFF', sdl.FLAGS)
        self.assertIn('-DSDL_DLOPEN=OFF', sdl.FLAGS)
        self.assertIn('-DCMAKE_DISABLE_FIND_PACKAGE_Git=TRUE', sdl.FLAGS)

    def test_source_pin_rejects_drift_unlisted_inputs_and_symlinks(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            source = root / 'artifacts/native-dependencies/source/sdl'
            tree = source / 'tree'
            tree.mkdir(parents=True)
            header = tree / 'fixture.h'
            header.write_text('// self-written inert source\n')
            manifest = source / 'source-manifest.json'
            manifest.write_text(json.dumps({'repository': sdl.REPOSITORY, 'revision': sdl.REVISION,
                                'files': [{'path': 'fixture.h', 'sha256': native.sha256(header)}]}))
            with patch.object(sdl, 'SOURCE_MANIFEST_SHA256', native.sha256(manifest)):
                self.assertEqual(len(sdl.source_inputs(source, root)), 1)
                header.write_text('changed')
                with self.assertRaisesRegex(ValueError, 'fingerprint changed'):
                    sdl.source_inputs(source, root)
                header.write_text('// self-written inert source\n')
                (tree / 'unlisted.h').write_text('unlisted')
                with self.assertRaisesRegex(ValueError, 'unlisted'):
                    sdl.source_inputs(source, root)
                (tree / 'unlisted.h').unlink()
                header.unlink()
                external = root / 'external.h'
                external.write_text('// self-written inert source\n')
                header.symlink_to(external)
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    sdl.source_inputs(source, root)

    def test_sdk_source_runtime_is_explicitly_cpu_only(self):
        self.assertIn('SDL_Init(SDL_INIT_EVENTS)', sdl.WITNESS)
        self.assertNotIn('SDL_CreateWindow', sdl.WITNESS)
        self.assertNotIn('SDL_INIT_VIDEO', sdl.WITNESS)
        self.assertNotIn('SDL_Vulkan', sdl.WITNESS)

    def test_saved_receipt_rechecks_artifact_and_cannot_grant_authority(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            managed = root / 'artifacts/native-dependencies'
            source = managed / 'source'
            (source / 'tree').mkdir(parents=True)
            build = source / 'tree/CMakeLists.txt'
            build.write_text('# self-written inert source fixture\n')
            manifest = source / 'source-manifest.json'
            manifest.write_text(json.dumps({'repository': sdl.REPOSITORY, 'revision': sdl.REVISION,
                                'files': [{'path': 'CMakeLists.txt', 'sha256': native.sha256(build)}]}))
            executable, artifact, tool = (managed / name for name in ('inert-executable', 'inert-archive', 'inert-tool'))
            for path in (executable, artifact, tool):
                path.write_text('inert authored fixture, never executed\n')
            with patch.object(sdl, 'SOURCE_MANIFEST_SHA256', native.sha256(manifest)):
                proof = sdl.SdlWitness(sdl.REPOSITORY, sdl.REVISION, native.fingerprint(manifest, root),
                    'arm64-apple-darwin', (native.fingerprint(tool, root),), sdl.source_inputs(source, root), (),
                    sdl.FLAGS, native.sha256(build), native.fingerprint(artifact, root),
                    hashlib.sha256(sdl.WITNESS.encode()).hexdigest(), native.fingerprint(executable, root),
                    sdl.EXPECTED, 0)
                receipt = managed / 'receipt.json'
                data = asdict(proof)
                receipt.write_text(json.dumps(data))
                self.assertEqual(sdl.verify_receipt(receipt, root)['runtime_exit_code'], 0)
                artifact.write_text('changed inert artifact')
                with self.assertRaisesRegex(ValueError, 'fingerprint changed'):
                    sdl.verify_receipt(receipt, root)
                data['link_authority'] = True
                receipt.write_text(json.dumps(data))
                with self.assertRaisesRegex(ValueError, 'configuration'):
                    sdl.verify_receipt(receipt, root)


if __name__ == '__main__':
    unittest.main()
