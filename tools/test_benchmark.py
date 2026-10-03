import importlib.util
import errno
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('benchmark', Path(__file__).with_name('benchmark.py'))
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


def fixture(root):
    for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml',
                 'tools/benchmark.py', 'tools/benchmark_resources.py', 'tools/check_dependency_age.py', 'crates/example/src/lib.rs'):
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('fixture')


def artifact(suite, executable, kind='bench'):
    return json.dumps({'reason': 'compiler-artifact', 'target': {'name': suite, 'kind': [kind]},
                       'executable': str(executable)})


class BenchmarkProvenanceTests(unittest.TestCase):
    def test_native_tool_capture_uses_configured_precedence_and_rejects_original_aliases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tool = root / 'independent/bin/clang'
            tool.parent.mkdir(parents=True)
            tool.write_bytes(b'own inert tool path fixture')
            environment = {'JAI_RS_CLANG': str(tool), 'LLVM_SYS_221_PREFIX': str(root / 'missing')}
            self.assertEqual(benchmark.native_tool_candidate(environment, root), tool.resolve())
            protected = root / 'vendor/clang'
            protected.parent.mkdir()
            protected.write_bytes(b'original inert fixture')
            alias = root / 'alias'
            alias.symlink_to(protected)
            with self.assertRaisesRegex(ValueError, 'protected inputs'):
                benchmark.native_tool_candidate({'JAI_RS_CLANG': str(alias)}, root)
            with self.assertRaisesRegex(ValueError, 'empty'):
                benchmark.native_tool_candidate({'JAI_RS_CLANG': ''}, root)

    def test_native_outputs_are_own_named_files_inside_the_retained_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'native-artifacts'
            root.mkdir()
            own = root / 'own-generated-program'
            own.write_bytes(b'own output fixture')
            hashes = benchmark.native_artifact_hashes([{'native_artifacts': [str(own)]}], root)
            self.assertEqual(hashes, {str(own.resolve()): benchmark.digest(own)})
            outside = Path(directory) / 'own-generated-program'
            outside.write_bytes(b'outside fixture')
            for path in (outside, root / 'missing', root / 'supplied-library.so'):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    benchmark.native_artifact_hashes([{'native_artifacts': [str(path)]}], root)

    def test_optional_hardware_details_can_be_unavailable_without_fabricating_values(self):
        with patch.object(benchmark.platform, 'system', return_value='Darwin'), \
             patch.object(benchmark.platform, 'processor', return_value='arm'), \
             patch.object(benchmark.subprocess, 'check_output', side_effect=OSError('restricted read')):
            hardware = benchmark.host_hardware()
        self.assertEqual(hardware['processor'], 'arm')
        self.assertIn('hw.model', hardware['unavailable'])
        self.assertNotIn('hw.model', {key: value for key, value in hardware.items() if key != 'unavailable'})

    def test_snapshot_detects_added_deleted_and_changed_source_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            before = benchmark.source_hashes(root)
            (root / 'crates/example/src/lib.rs').write_text('changed')
            (root / 'crates/example/source.jai').write_text('self_written :: 1;')
            (root / 'tools/check_dependency_age.py').unlink()
            # Source enumeration requires the policy script, so compare deletion via hashes.
            after = {name: benchmark.digest(root / name) for name in before if (root / name).exists()}
            after['crates/example/source.jai'] = benchmark.digest(root / 'crates/example/source.jai')
            self.assertEqual(benchmark.changed_inputs(before, after), [
                'crates/example/source.jai', 'crates/example/src/lib.rs', 'tools/check_dependency_age.py'])

    def test_copy_snapshot_rejects_a_changed_input_before_it_can_be_built(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            inputs = benchmark.source_hashes(root)
            (root / 'crates/example/src/lib.rs').write_text('edited during capture')
            with self.assertRaisesRegex(RuntimeError, 'source changed while capturing snapshot'):
                benchmark.copy_snapshot(root, root / 'captured', inputs)

    def test_only_selected_cargo_benchmark_artifacts_inside_target_are_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'target'
            target.mkdir()
            executable = target / 'compiler'
            executable.write_bytes(b'trusted benchmark fixture')
            output = '\n'.join([json.dumps({'reason': 'build-finished', 'success': True}),
                                artifact('compiler', executable), artifact('unrelated', '/other/executable')])
            self.assertEqual(benchmark.benchmark_executables(output, ['compiler'], target),
                             {'compiler': executable.resolve()})
            with self.assertRaisesRegex(ValueError, 'outside the build directory or missing'):
                benchmark.benchmark_executables(artifact('compiler', Path(directory) / 'foreign'), ['compiler'], target)
            with self.assertRaisesRegex(ValueError, 'missing benchmark executables'):
                benchmark.benchmark_executables(artifact('compiler', executable, 'bin'), ['compiler'], target)

    def test_source_corpus_fingerprint_changes_without_archiving_original_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            reference.parent.mkdir()
            reference.write_text('example :: 1;')
            before = benchmark.corpus_hashes(root, False, True)
            reference.write_text('example :: 2;')
            self.assertNotEqual(before, benchmark.corpus_hashes(root, False, True))
            shim = root / 'crates/example/src/debug_shim.cpp'
            shim.write_text('extern "C" int own_shim() { return 42; }')
            (shim.parent / 'supplied-native.o').write_bytes(b'not a source input')
            inputs = benchmark.source_hashes(root)
            self.assertIn('crates/example/src/debug_shim.cpp', inputs)
            self.assertNotIn('crates/example/src/supplied-native.o', inputs)
            benchmark.copy_snapshot(root, root / 'captured', inputs)
            self.assertEqual((root / 'captured/crates/example/src/debug_shim.cpp').read_text(), shim.read_text())

    def test_smoke_and_measurement_flags_preserve_locked_offline_builds(self):
        command = benchmark.build_command(['compiler', 'vm', 'discovery'], True)
        for flag in ('--locked', '--offline', '--no-run', '--message-format=json'):
            self.assertIn(flag, command)
        smoke = benchmark.measurement_arguments(25, True, True, False)
        self.assertIn('--test', smoke)
        self.assertNotIn('--sample-count', smoke)
        self.assertIn('--include-ignored', smoke)
        self.assertEqual(smoke[-2:], ['--skip', 'reference_lex'])
        self.assertEqual(benchmark.measurement_arguments(25, False, False, True),
                         ['--color', 'never', '--bench', '--sample-count', '25', '--sample-size', '1'])

    def test_actual_divan_measurements_are_required_instead_of_test_tree_output(self):
        # Own float-alias benchmark output, captured with Divan --bench and ten samples.
        actual = Path(__file__).with_name('testdata').joinpath('divan-measurement.txt').read_text()
        self.assertEqual(benchmark.measured_rows(actual, 10), 3)
        for output in ('', 'vm\n╰─ checked_call_execute\n   ╰─ 4\n',
                       actual.replace('│ 10      │ 10', '│ 9       │ 9'),
                       actual.replace('alloc:', 'omitted:')):
            with self.subTest(output=output[:80]), self.assertRaises(ValueError):
                benchmark.measured_rows(output, 10)

    def test_failed_dependency_age_gate_is_recorded_and_cannot_start_a_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                 patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                 patch.object(benchmark, 'host_hardware', return_value={}), \
                 patch.object(benchmark.subprocess, 'run', return_value=SimpleNamespace(returncode=7)) as run, \
                 patch('sys.argv', ['benchmark.py', '--smoke', '--offline', '--skip-reference']), \
                 patch('builtins.print'):
                with self.assertRaises(SystemExit):
                    benchmark.main()
            self.assertEqual(run.call_count, 1)
            metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
            self.assertFalse(metadata['valid'])
            self.assertEqual(metadata['dependency_age']['exit_code'], 7)
            self.assertEqual(metadata['runs'], [])
            self.assertIn('verification failed', metadata['error'])

    def test_disk_full_during_final_metadata_preserves_initial_invalid_record(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            save = benchmark.save_metadata
            calls = 0
            def fail_final(destination, metadata):
                nonlocal calls
                calls += 1
                if calls == 2:
                    raise OSError(errno.ENOSPC, 'No space left on device')
                save(destination, metadata)
            with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                 patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                 patch.object(benchmark, 'host_hardware', return_value={}), \
                 patch.object(benchmark.subprocess, 'run', return_value=SimpleNamespace(returncode=7)), \
                 patch.object(benchmark, 'save_metadata', side_effect=fail_final), \
                 patch('sys.argv', ['benchmark.py', '--smoke', '--offline', '--skip-reference']), \
                 patch('builtins.print'):
                with self.assertRaises(SystemExit):
                    benchmark.main()
            metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
            self.assertFalse(metadata['valid'])
            self.assertEqual(metadata['error'], 'run did not complete')
            self.assertEqual(metadata['runs'], [])

    def test_partial_metadata_write_does_not_replace_the_previous_record(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory)
            benchmark.save_metadata(destination, {'valid': False})
            write = Path.write_text
            def partial_write(path, content, **kwargs):
                write(path, content[:5], **kwargs)
                raise OSError(errno.ENOSPC, 'No space left on device')
            with patch.object(Path, 'write_text', partial_write):
                with self.assertRaises(OSError):
                    benchmark.save_metadata(destination, {'valid': True})
            self.assertEqual(json.loads((destination / 'metadata.json').read_text()), {'valid': False})
            self.assertFalse((destination / 'metadata.tmp').exists())

    def test_isolated_source_change_during_build_prevents_any_benchmark_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            def run(command, **kwargs):
                if command[0] == 'cargo':
                    (Path(kwargs['cwd']) / 'crates/example/src/lib.rs').write_text('new source during build')
                return SimpleNamespace(returncode=0)
            with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                 patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                 patch.object(benchmark, 'host_hardware', return_value={}), \
                 patch.object(benchmark.subprocess, 'run', side_effect=run) as mocked, \
                 patch('sys.argv', ['benchmark.py', '--smoke', '--offline', '--skip-reference']), \
                 patch('builtins.print'):
                with self.assertRaises(SystemExit):
                    benchmark.main()
            self.assertEqual(mocked.call_count, 2)
            metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
            self.assertFalse(metadata['valid'])
            self.assertEqual(metadata['changed_build_inputs'], ['crates/example/src/lib.rs'])
            self.assertEqual(metadata['runs'], [])

    def test_live_edits_during_build_do_not_change_the_compiled_snapshot(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            executable = root / 'target/vm'
            executable.parent.mkdir()
            executable.write_bytes(b'own benchmark fixture')
            def run(command, **kwargs):
                if command[0] == 'cargo':
                    build_source = Path(kwargs['cwd'])
                    self.assertEqual((build_source / 'crates/example/src/lib.rs').read_text(), 'fixture')
                    self.assertEqual(command[command.index('--manifest-path') + 1], str(build_source / 'Cargo.toml'))
                    self.assertEqual(kwargs['env']['JAI_BENCH_CORPUS_ROOT'], str(root))
                    (root / 'crates/example/src/lib.rs').write_text('new live source')
                    kwargs['stdout'].write(artifact('vm', executable) + '\n')
                elif len(command) > 4 and Path(command[4]).parent.name == 'executables':
                    executable.unlink(missing_ok=True)
                    if '--bench' in command:
                        kwargs['stdout'].write(Path(__file__).with_name('testdata').joinpath('divan-measurement.txt').read_text().replace('│ 10      │ 10', '│ 2       │ 2'))
                    kwargs['stderr'].write('JAI_BENCH_WORKLOAD ' + json.dumps({
                        'name': 'explicit_budget', 'fuel': 160_000_000, 'steps': 148_099_100,
                        'expected_outcome': 'complete', 'live_allocations': 2,
                    }) + '\n')
                return SimpleNamespace(returncode=0)
            with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                 patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                 patch.object(benchmark, 'host_hardware', return_value={}), \
                 patch.object(benchmark.subprocess, 'run', side_effect=run), \
                 patch('sys.argv', ['benchmark.py', '--bench', 'vm', '--samples', '2', '--offline']), \
                 patch('builtins.print'):
                benchmark.main()
            metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
            self.assertTrue(metadata['valid'])
            self.assertEqual(metadata['source_changes_after_build'], ['crates/example/src/lib.rs'])
            self.assertEqual(len(metadata['runs']), 2)
            self.assertFalse(executable.exists())
            self.assertEqual(Path(metadata['executables']['vm']['path']).read_bytes(), b'own benchmark fixture')
            for record in metadata['runs']:
                self.assertEqual(record['workloads'][0]['fuel'], 160_000_000)
                self.assertEqual(record['workloads'][0]['steps'], 148_099_100)
                self.assertEqual(record['workloads'][0]['expected_outcome'], 'complete')
            self.assertEqual((Path(metadata['build_source_directory']) / 'crates/example/src/lib.rs').read_text(), 'fixture')

    def test_retained_source_and_archive_changes_invalidate_the_completed_smoke(self):
        for changed in ('source', 'archive'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture(root)
                executable = root / 'target/vm'
                executable.parent.mkdir()
                executable.write_bytes(b'own benchmark fixture')
                def run(command, **kwargs):
                    if command[0] == 'cargo':
                        kwargs['stdout'].write(artifact('vm', executable) + '\n')
                    elif len(command) > 4 and Path(command[4]).parent.name == 'executables':
                        destination = Path(command[4]).parent.parent
                        path = destination / ('source/crates/example/src/lib.rs' if changed == 'source'
                                              else 'compiler-source.tar.gz')
                        path.write_bytes(b'changed retained input')
                    return SimpleNamespace(returncode=0)
                with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                     patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                     patch.object(benchmark, 'host_hardware', return_value={}), \
                     patch.object(benchmark.subprocess, 'run', side_effect=run), \
                     patch('sys.argv', ['benchmark.py', '--bench', 'vm', '--smoke', '--offline']), \
                     patch('builtins.print'):
                    with self.assertRaises(SystemExit):
                        benchmark.main()
                metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
                self.assertFalse(metadata['valid'])
                self.assertEqual(metadata['runs'][0]['exit_code'], 0)
                self.assertIn('changed during measurement', metadata['error'])

    def test_executable_change_during_smoke_prevents_timing_a_different_binary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            executable = root / 'target/vm'
            executable.parent.mkdir()
            executable.write_bytes(b'first own benchmark')
            def run(command, **kwargs):
                if command[0] == 'cargo':
                    kwargs['stdout'].write(artifact('vm', executable) + '\n')
                elif len(command) > 4 and Path(command[4]).parent.name == 'executables':
                    Path(command[4]).write_bytes(b'replaced own benchmark')
                return SimpleNamespace(returncode=0)
            with patch.object(benchmark, 'ROOT', root), patch.object(benchmark, 'version', return_value='fixture'), \
                 patch.object(benchmark.platform, 'platform', return_value='fixture'), \
                 patch.object(benchmark, 'host_hardware', return_value={}), \
                 patch.object(benchmark.subprocess, 'run', side_effect=run) as mocked, \
                 patch('sys.argv', ['benchmark.py', '--bench', 'vm', '--samples', '2', '--offline']), \
                 patch('builtins.print'):
                with self.assertRaises(SystemExit):
                    benchmark.main()
            self.assertEqual(mocked.call_count, 3)
            metadata = json.loads(next((root / 'artifacts/benchmarks').rglob('metadata.json')).read_text())
            self.assertFalse(metadata['valid'])
            self.assertEqual(len(metadata['runs']), 1)
            self.assertTrue(metadata['runs'][0]['smoke'])
            self.assertIn('executable changed during measurement', metadata['error'])


if __name__ == '__main__':
    unittest.main()
