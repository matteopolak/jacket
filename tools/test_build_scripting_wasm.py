"""Check configured wasm artifact discovery and staging with authored inert bytes."""
import hashlib
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import build_scripting_wasm as wasm


class ScriptingWasmBuildTests(unittest.TestCase):
    def test_metadata_target_drives_build_staging_and_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); root = base / 'source'; root.mkdir()
            target = base / 'external-apfs/cache'; compiled = target / 'wasm32-unknown-unknown/release/jai_wasm.wasm'
            compiled.parent.mkdir(parents=True); compiled.write_bytes(b'\x00asm\x01\x00\x00\x00')
            glue = root / 'crates/jai-wasm/js'; glue.mkdir(parents=True)
            (glue / 'engine.mjs').write_text('// authored inert staging fixture\n')
            (glue / 'README.md').write_text('own bundle readme fixture\n')
            (glue / 'unrelated.mjs').write_text('not part of the bundle\n')
            driver = root / 'tools/jaifmt/playground.jai'; driver.parent.mkdir(parents=True)
            driver.write_text('// own formatter driver fixture\n')
            example = root / 'examples/walk'; (example / 'part').mkdir(parents=True)
            (example / 'main.jai').write_text('#load "part/one.jai";\n')
            (example / 'part/one.jai').write_text('// own nested example fixture\n')
            (example / '.hidden').write_text('editor state, not shipped\n')
            (root / 'tests').mkdir()
            (root / 'tests/examples.json').write_text(json.dumps({'cases': [
                {'id': 'walk', 'bundle': 'walk', 'directory': 'examples/walk', 'main': 'main.jai'},
                {'id': 'native-only', 'directory': 'examples/walk', 'main': 'main.jai'}]}))
            prefix = ['/own/rustup', 'run', 'nightly-2026-08-29', 'cargo']
            with patch.object(wasm, 'ROOT', root), patch.dict(os.environ, {}, clear=True), \
                 patch.object(wasm, 'pinned_cargo_command', return_value=prefix), \
                 patch.object(wasm.subprocess, 'check_output', return_value=json.dumps({'target_directory': str(target)})) as metadata, \
                 patch.object(wasm.subprocess, 'run') as build, \
                 patch.object(wasm.shutil, 'disk_usage', return_value=SimpleNamespace(free=8*1024**3)), \
                 patch('sys.argv', ['build_scripting_wasm.py', '--release']), patch('builtins.print'):
                wasm.main()
            command = build.call_args.args[0]
            self.assertEqual(command[:len(prefix)], prefix)
            self.assertEqual(command[command.index('--target-dir')+1], str(target))
            self.assertEqual(build.call_args.kwargs['env']['CARGO_TARGET_DIR'], str(target))
            self.assertIn('--offline', command); self.assertIn('--locked', command)
            self.assertEqual(metadata.call_args.args[0][:len(prefix)], prefix)
            output = root / 'artifacts/scripting-runtime'
            self.assertEqual((output / 'jai_wasm.wasm').read_bytes(), compiled.read_bytes())
            receipt = json.loads((output / 'build-metadata.json').read_text())
            self.assertEqual(receipt['target_directory']['path'], str(target))
            self.assertEqual(receipt['wasm_build_path'], str(compiled))
            self.assertEqual(receipt['wasm_sha256'], hashlib.sha256(compiled.read_bytes()).hexdigest())
            self.assertFalse((root / 'target').exists())
            self.assertEqual(sorted(path.name for path in output.iterdir()),
                             ['README.md', 'build-metadata.json', 'engine.mjs', 'jai_wasm.wasm', 'jaifmt-playground.jai',
                              'walk', 'walk.json'])
            self.assertEqual((output / 'jaifmt-playground.jai').read_text(), '// own formatter driver fixture\n')
            # Bundled examples are copied as a tree (without dotfiles) and indexed.
            self.assertEqual(json.loads((output / 'walk.json').read_text()),
                             {'schema_version': 1, 'main': 'main.jai', 'files': ['main.jai', 'part/one.jai']})
            self.assertEqual((output / 'walk/part/one.jai').read_text(), '// own nested example fixture\n')
            self.assertFalse((output / 'walk/.hidden').exists())

    def test_separate_build_volume_floor_refuses_build(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve() / 'source'; root.mkdir()
            target = Path(temporary).resolve() / 'external-target'; target.mkdir()
            with patch.object(wasm, 'ROOT', root), patch.dict(os.environ, {'CARGO_TARGET_DIR': str(target)}), \
                 patch.object(wasm, 'pinned_cargo_command', return_value=['own-cargo']), \
                 patch.object(wasm.shutil, 'disk_usage', side_effect=lambda path: SimpleNamespace(free=1 if path == target else 8*1024**3)), \
                 patch.object(wasm.subprocess, 'run') as build, \
                 patch('sys.argv', ['build_scripting_wasm.py']):
                with self.assertRaisesRegex(SystemExit, 'each used volume'):
                    wasm.main()
                build.assert_not_called()

    def test_jaifmt_wasm_is_compiled_by_the_given_native_jaic(self):
        command = wasm.jaifmt_wasm_command(Path('/own/jaic'), Path('/src'), Path('/out/jaifmt.wasm'))
        self.assertEqual(command[:3], ['/own/jaic', 'build', '/src/tools/jaifmt/wasm.jai'])
        self.assertEqual(command[command.index('-os') + 1], 'wasm')
        self.assertEqual(command[command.index('-o') + 1], '/out/jaifmt.wasm')
        self.assertIn('-O2', command)

    def test_explicit_external_target_is_in_the_pinned_build_command(self):
        prefix = ['own-rustup', 'run', 'nightly-2026-08-29', 'cargo']
        command = wasm.build_command(prefix, Path('/Volumes/CodexBuilds/targets/jai'), False)
        self.assertEqual(command[command.index('--target-dir')+1], '/Volumes/CodexBuilds/targets/jai')
        self.assertNotIn('--release', command)
