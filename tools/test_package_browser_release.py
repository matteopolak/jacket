"""Consumer archive proofs with authored inert fixtures; build/VM calls are mocked."""
from datetime import date
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import package_browser_release as release

COMMIT = 'a' * 40


def fixture(root):
    root.mkdir()
    (root / 'rust-toolchain.toml').write_text('[toolchain]\nchannel="nightly-2026-08-29"\n')


def staged(stage):
    stage.mkdir()
    for name in release.REQUIRED - {'build-metadata.json', 'jai_wasm.wasm'}:
        (stage / name).parent.mkdir(parents=True, exist_ok=True)
        (stage / name).write_text('own inert fixture\n')
    wasm = b'\x00asm\x01\x00\x00\x00'
    (stage / 'jai_wasm.wasm').write_bytes(wasm)
    (stage / 'build-metadata.json').write_text(json.dumps({'wasm_sha256': hashlib.sha256(wasm).hexdigest(), 'wasm_build_path': '/private/host/cache/module.wasm'}))


class BrowserReleaseTests(unittest.TestCase):
    def test_consumer_inventory_is_exhaustive_relocatable_and_reproducible(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary); root = base / 'source'; fixture(root)
            def run(command, **kwargs):
                if '--release' in command:
                    self.assertEqual(command[command.index('--target-dir') + 1], '/Volumes/CodexBuilds/targets/jai')
                    staged(Path(command[command.index('--output') + 1]))
                if '--report' in command:
                    Path(command[command.index('--report') + 1]).write_text(json.dumps({'commit': COMMIT, 'runtime': True, 'lsp': False}))
                return subprocess.CompletedProcess(command, 0)
            with patch.object(release, 'clean_revision', return_value=COMMIT), patch.object(release.subprocess, 'run', side_effect=run) as calls:
                one = release.package(root, base / 'one', Path('/Volumes/CodexBuilds/targets/jai'))
                two = release.package(root, base / 'two', Path('/Volumes/CodexBuilds/targets/jai'))
            self.assertTrue(any(Path(command.args[0][0]).name == 'node' for command in calls.call_args_list))
            self.assertEqual(one, two)
            self.assertEqual((base / 'one' / release.ARCHIVE).read_bytes(), (base / 'two' / release.ARCHIVE).read_bytes())
            self.assertFalse(one['dirty_checkout']); self.assertEqual(one['commit'], COMMIT)
            with zipfile.ZipFile(base / 'one' / release.ARCHIVE) as archive:
                self.assertEqual(archive.namelist(), [item['path'] for item in one['files']])
                self.assertEqual(sorted(archive.namelist()), sorted(release.REQUIRED))
                self.assertNotIn(release.MANIFEST, archive.namelist())
                metadata = json.loads(archive.read('build-metadata.json'))
                self.assertEqual(metadata['commit'], COMMIT)
                self.assertEqual(metadata['toolchain'], 'nightly-2026-08-29')
                self.assertEqual(metadata['wasm_sha256'], hashlib.sha256(archive.read('jai_wasm.wasm')).hexdigest())
                self.assertNotIn('/private/host', archive.read('build-metadata.json').decode())
                for item in one['files']:
                    content = archive.read(item['path'])
                    self.assertEqual(len(content), item['size'])
                    self.assertEqual(hashlib.sha256(content).hexdigest(), item['sha256'])
                nested = base / 'site/jai' / COMMIT; archive.extractall(nested)
                self.assertEqual(json.loads((nested / 'build-metadata.json').read_text())['commit'], COMMIT)
            self.assertEqual(one['schema_version'], 2); self.assertNotIn('entrypoint', one)
            self.assertEqual(one['archive']['sha256'], hashlib.sha256((base / 'one' / release.ARCHIVE).read_bytes()).hexdigest())

    def test_actual_probe_failure_never_publishes_assets(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary); root = base / 'source'; fixture(root); output = base / 'output'
            def run(command, **kwargs):
                if '--release' in command: staged(Path(command[command.index('--output') + 1]))
                if Path(command[0]).name == 'node': raise subprocess.CalledProcessError(1, command)
                if '--report' in command:
                    Path(command[command.index('--report') + 1]).write_text(json.dumps({'commit': COMMIT, 'runtime': True, 'lsp': False}))
                return subprocess.CompletedProcess(command, 0)
            with patch.object(release, 'clean_revision', return_value=COMMIT), patch.object(release.subprocess, 'run', side_effect=run):
                with self.assertRaises(subprocess.CalledProcessError): release.package(root, output)
            self.assertFalse(output.exists())

    def test_dirty_source_and_source_drift_cannot_claim_a_clean_revision(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary); root = base / 'source'; fixture(root); output = base / 'output'
            with patch.object(release, 'clean_revision', side_effect=ValueError('dirty source')), patch.object(release.subprocess, 'run') as build:
                with self.assertRaises(ValueError): release.package(root, output)
                build.assert_not_called()
            def run(command, **kwargs):
                if '--release' in command: staged(Path(command[command.index('--output') + 1]))
                if '--report' in command:
                    Path(command[command.index('--report') + 1]).write_text(json.dumps({'commit': COMMIT, 'runtime': True, 'lsp': False}))
                return subprocess.CompletedProcess(command, 0)
            with patch.object(release, 'clean_revision', side_effect=[COMMIT, 'b' * 40]), patch.object(release.subprocess, 'run', side_effect=run):
                with self.assertRaisesRegex(ValueError, 'source changed'): release.package(root, output)
            self.assertFalse(output.exists())

    def test_archive_rejects_unsafe_paths_symlinks_native_files_and_size_overflow(self):
        for name in ('../engine.mjs', '/engine.mjs', 'a//engine.mjs', 'protected/secret.jai', 'engine\\x.mjs', 'native.so', 'x\n.mjs', 'index.html', 'style.css'):
            with self.subTest(name=name), self.assertRaises(ValueError): release.asset_path(name)
        # The tour ships as a folder tree of sources and its index.
        for name in ('tour.json', 'tour/main.jai', 'tour/meta/macros.jai', 'tour/tour.md'):
            self.assertEqual(release.asset_path(name), name)
        with tempfile.TemporaryDirectory() as temporary:
            stage = Path(temporary); (stage / 'escape.mjs').symlink_to('/outside/no-read')
            with self.assertRaisesRegex(ValueError, 'symlinks'): release.assets(stage)
            (stage / 'escape.mjs').unlink(); (stage / 'oversized.mjs').write_bytes(b'12345')
            with patch.object(release, 'MAX_FILE_BYTES', 4), self.assertRaisesRegex(ValueError, 'size limit'): release.assets(stage)

    def test_changed_asset_bytes_and_nonempty_destination_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary); stage = base / 'stage'; staged(stage)
            inventory = release.assets(stage); (stage / 'engine.mjs').write_text('changed own source')
            with self.assertRaisesRegex(ValueError, 'changed before archiving'): release.write_archive(stage, base / 'bad.zip', inventory)
            root = base / 'source'; fixture(root); output = base / 'existing'; output.mkdir(); (output / 'keep.txt').write_text('preserve')
            with patch.object(release, 'clean_revision', return_value=COMMIT), patch.object(release.subprocess, 'run') as build:
                with self.assertRaisesRegex(ValueError, 'never overwritten'): release.package(root, output)
                build.assert_not_called()
            self.assertEqual((output / 'keep.txt').read_text(), 'preserve')

    def test_output_volume_headroom_refuses_all_builds_before_the_disk_floor(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary); root = base / 'source'; fixture(root)
            with patch.object(release, 'clean_revision', return_value=COMMIT), patch.object(release.shutil, 'disk_usage') as disk, patch.object(release.subprocess, 'run') as build:
                disk.return_value.free = 2 * 1024**3
                with self.assertRaisesRegex(ValueError, 'headroom'): release.package(root, base / 'output')
                build.assert_not_called()
            self.assertFalse((base / 'output').exists())

    def test_supplied_tool_symlink_alias_is_refused_without_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'source'; fixture(root)
            original = root / 'protected/original-node'; original.parent.mkdir(); original.write_text('inert original fixture')
            alias = root / 'alias'; alias.symlink_to(original)
            with patch.object(release.shutil, 'which', return_value=str(alias)), self.assertRaisesRegex(ValueError, 'protected'):
                release.browser_tool('node', root)

    def test_toolchain_age_gate_is_exact_and_fails_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'source'; fixture(root)
            self.assertEqual(release.pinned_channel(root, date(2026, 9, 12)), 'nightly-2026-08-29')
            with self.assertRaisesRegex(ValueError, '14 days'): release.pinned_channel(root, date(2026, 9, 11))
            (root / 'rust-toolchain.toml').write_text('[toolchain]\nchannel="nightly"\n')
            with self.assertRaisesRegex(ValueError, 'exact dated'): release.pinned_channel(root)


if __name__ == '__main__':
    unittest.main()
