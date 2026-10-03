"""Exercise the runner's execution and dependency boundaries with own helpers."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock
import run_fuzz


class FuzzRunnerTests(unittest.TestCase):
    def test_target_override_and_environment_do_not_fall_back_to_internal_storage(self):
        explicit = Path('/own/ssd/explicit')
        environment = {'CARGO_TARGET_DIR': '/own/ssd/configured'}
        with mock.patch.object(run_fuzz.subprocess, 'check_output') as metadata:
            self.assertEqual(run_fuzz.configured_target_directory(explicit, environment, 'cargo'), explicit)
            self.assertEqual(run_fuzz.configured_target_directory(None, environment, 'cargo'),
                             Path('/own/ssd/configured'))
            metadata.assert_not_called()
        with self.assertRaisesRegex(RuntimeError, 'must not be empty'):
            run_fuzz.configured_target_directory(None, {'CARGO_TARGET_DIR': ''}, 'cargo')

    def test_cargo_configuration_selects_the_default_target_volume(self):
        environment = {}
        with mock.patch.object(run_fuzz.subprocess, 'check_output',
                               return_value=json.dumps({'target_directory': '/own/ssd/default'})) as metadata:
            self.assertEqual(run_fuzz.configured_target_directory(None, environment, 'own-cargo'),
                             Path('/own/ssd/default'))
            metadata.assert_called_once_with(
                ['own-cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked', '--offline'],
                cwd=run_fuzz.ROOT, env=environment, text=True)

    def test_artifacts_parent_does_not_exclude_fuzz_source_or_lock(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'artifacts/own-checkpoint/source'
            for name in ('Cargo.lock', 'Cargo.toml', 'src/lib.rs', 'seeds/own.jai',
                         'target/generated.rs', 'corpus/new-input', 'artifacts/run.json'):
                path = root / 'fuzz' / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('independently authored capture fixture')
            inputs = run_fuzz.fuzz_source_hashes(root)
            self.assertEqual(set(inputs), {'fuzz/Cargo.lock', 'fuzz/Cargo.toml',
                                          'fuzz/src/lib.rs', 'fuzz/seeds/own.jai'})
            destination = root / 'captured'
            run_fuzz.benchmark.copy_snapshot(root, destination, inputs)
            self.assertEqual(run_fuzz.digest(destination / 'fuzz/Cargo.lock'), inputs['fuzz/Cargo.lock'])

    def test_protected_tool_symlink_is_refused_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            protected = root / 'reference'
            protected.mkdir()
            own = protected / 'own-tool'
            own.write_text('never execute this fixture')
            link = root / 'tool-link'
            link.symlink_to(own)
            with mock.patch.object(run_fuzz, 'ROOT', root), mock.patch.object(run_fuzz.shutil, 'which', return_value=str(link)):
                with self.assertRaisesRegex(RuntimeError, 'protected original'):
                    run_fuzz.trusted_tool('cargo-fuzz')

    def test_internal_cargo_build_uses_locked_offline_resolution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            received = root / 'args.json'
            real = root / 'own-cargo'
            real.write_text(f'#!{sys.executable}\nimport json,sys\n'
                            f'open({str(received)!r},"w").write(json.dumps(sys.argv[1:]))\n')
            real.chmod(0o755)
            shim = run_fuzz.cargo_shim(root, real)
            subprocess.run([str(shim), 'build', '--manifest-path', 'own/Cargo.toml'], check=True)
            args = json.loads(received.read_text())
            self.assertIn('--locked', args)
            self.assertIn('--offline', args)
            self.assertEqual(args[:3], ['build', '--manifest-path', 'own/Cargo.toml'])

    def test_build_success_is_insufficient_without_executed_inputs(self):
        for text in ('Finished dev profile', 'Done 0 runs in 0 second(s)'):
            with self.assertRaises(RuntimeError):
                run_fuzz.completed_runs(text)
        self.assertEqual(run_fuzz.completed_runs('#100 DONE cov: 42\nDone 100 runs in 1 second(s)'), 100)

    def test_low_space_refuses_a_campaign(self):
        with mock.patch.object(run_fuzz.shutil, 'disk_usage', return_value=mock.Mock(free=1)):
            with self.assertRaisesRegex(RuntimeError, '2 GiB'):
                run_fuzz.require_disk_space(Path('.'))

    def test_campaign_checks_source_and_separate_build_storage(self):
        paths = (Path('/own/source-volume'), Path('/own/build-volume'))
        process = mock.Mock()
        process.wait.return_value = 0
        with mock.patch.object(run_fuzz.subprocess, 'Popen', return_value=process):
            with mock.patch.object(run_fuzz, 'require_disk_space') as check:
                self.assertEqual(run_fuzz.run_campaign(['own-fuzzer'], None, {},
                                                      storage_paths=paths), 0)
        self.assertEqual(check.call_args_list, [mock.call(path) for path in paths])

    def test_timeout_stops_descendant_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready = root / 'child-ready'
            escaped = root / 'child-escaped'
            child = root / 'child.py'
            child.write_text('import pathlib,time\n'
                             f'pathlib.Path({str(ready)!r}).write_text("ready")\n'
                             'time.sleep(1)\n'
                             f'pathlib.Path({str(escaped)!r}).write_text("escaped")\n')
            launcher = root / 'launcher.py'
            launcher.write_text('import subprocess,sys,time\n'
                                f'subprocess.Popen([sys.executable,{str(child)!r}])\n'
                                'time.sleep(60)\n')
            real_popen = subprocess.Popen

            def ready_process(*args, **kwargs):
                process = real_popen(*args, **kwargs)
                deadline = time.monotonic() + 5
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(ready.exists(), 'test must actually start the descendant')
                return process

            with (root / 'log').open('w') as output:
                with mock.patch.object(run_fuzz.subprocess, 'Popen', side_effect=ready_process):
                    with self.assertRaises(subprocess.TimeoutExpired):
                        run_fuzz.run_campaign([sys.executable, str(launcher)], output, os.environ, timeout=0.2)
            time.sleep(1.1)
            self.assertFalse(escaped.exists(), 'timed-out descendant must stop before further writes')


if __name__ == '__main__':
    unittest.main()
