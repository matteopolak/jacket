"""Regress explicit/configured external targets without invoking Cargo or rustup."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import cargo_build_paths as paths


class CargoBuildPathsTests(unittest.TestCase):
    def test_cli_wins_and_relative_paths_use_the_repository(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with patch.object(paths.subprocess, 'check_output') as cargo:
                selected = paths.configured_target_directory('own-external/cache',
                    {'CARGO_TARGET_DIR': '/other/target'}, ['unused-cargo'], root)
                self.assertEqual(selected.path, root / 'own-external/cache')
                self.assertEqual(selected.source, 'command-line')
                cargo.assert_not_called()

    def test_environment_external_target_does_not_query_or_fall_back(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with patch.object(paths.subprocess, 'check_output') as cargo:
                selected = paths.configured_target_directory(None,
                    {'CARGO_TARGET_DIR': '/Volumes/CodexBuilds/targets/jai'}, ['unused-cargo'], root)
                self.assertEqual(selected.path, Path('/Volumes/CodexBuilds/targets/jai'))
                self.assertEqual(selected.source, 'environment')
                cargo.assert_not_called()

    def test_empty_environment_is_invalid_instead_of_an_internal_fallback(self):
        with patch.object(paths.subprocess, 'check_output') as cargo:
            for value in ('', '   '):
                with self.subTest(value=value), self.assertRaisesRegex(ValueError, 'must not be empty'):
                    paths.configured_target_directory(None, {'CARGO_TARGET_DIR': value}, ['cargo'], Path('/own/repo'))
            cargo.assert_not_called()

    def test_actual_cargo_metadata_uses_locked_offline_pinned_command(self):
        root = Path('/own/repo')
        prefix = ['/own/rustup', 'run', 'nightly-2026-08-29', 'cargo']
        environment = {'CARGO_BUILD_TARGET_DIR': '/Volumes/CodexBuilds/targets/jai'}
        with patch.object(paths.subprocess, 'check_output', return_value=json.dumps(
                {'target_directory': '/Volumes/CodexBuilds/targets/jai'})) as cargo:
            selected = paths.configured_target_directory(None, environment, prefix, root)
        expected = prefix + ['metadata', '--format-version', '1', '--no-deps', '--locked', '--offline']
        cargo.assert_called_once_with(expected, cwd=root, env=environment, text=True)
        self.assertEqual(selected.path, Path('/Volumes/CodexBuilds/targets/jai'))
        self.assertEqual(selected.receipt()['metadata_command'], expected)
        self.assertEqual(selected.source, 'cargo-metadata')

    def test_invalid_metadata_and_cargo_failure_cannot_select_a_fallback(self):
        for value in (None, '', 'relative/cache', 7):
            with self.subTest(value=value), patch.object(paths.subprocess, 'check_output',
                    return_value=json.dumps({'target_directory': value})):
                with self.assertRaisesRegex(ValueError, 'absolute target_directory'):
                    paths.configured_target_directory(None, {}, ['cargo'], Path('/own/repo'))
        with patch.object(paths.subprocess, 'check_output', side_effect=subprocess.CalledProcessError(1, ['own-cargo'])):
            with self.assertRaises(subprocess.CalledProcessError):
                paths.configured_target_directory(None, {}, ['cargo'], Path('/own/repo'))

    def test_original_targets_and_symlink_aliases_are_rejected_in_every_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            protected = root / 'reference'; protected.mkdir()
            alias = root / 'external-alias'; alias.symlink_to(protected, target_is_directory=True)
            for directory in (protected / 'cache', alias / 'cache', root / 'corpus/cache', root / 'vendor/cache', root / '.git/cache'):
                with self.subTest(directory=directory):
                    with self.assertRaisesRegex(ValueError, 'protected original'):
                        paths.configured_target_directory(directory, {}, ['cargo'], root)
                    with self.assertRaisesRegex(ValueError, 'protected original'):
                        paths.configured_target_directory(None, {'CARGO_TARGET_DIR': str(directory)}, ['cargo'], root)
                    with patch.object(paths.subprocess, 'check_output', return_value=json.dumps({'target_directory': str(directory)})):
                        with self.assertRaisesRegex(ValueError, 'protected original'):
                            paths.configured_target_directory(None, {}, ['cargo'], root)

    def test_toolchain_pin_ignores_an_inherited_channel_and_rejects_protected_tool(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'rust-toolchain.toml').write_text('[toolchain]\nchannel = "nightly-2026-08-29"\n')
            tool = root / 'own-rustup'; tool.write_text('inert own path fixture')
            with patch.object(paths.shutil, 'which', return_value=str(tool)):
                command = paths.pinned_cargo_command(root, {'RUSTUP_TOOLCHAIN': 'other-channel'})
            self.assertEqual(command, [str(tool), 'run', 'nightly-2026-08-29', 'cargo'])
            protected = root / 'reference'; protected.mkdir(); blocked = protected / 'rustup'; blocked.write_text('inert own fixture')
            alias = root / 'tool-alias'; alias.symlink_to(blocked)
            with patch.object(paths.shutil, 'which', return_value=str(alias)):
                with self.assertRaisesRegex(ValueError, 'protected original'):
                    paths.pinned_cargo_command(root, {})
