"""Selection, provenance, and process boundaries for real library acceptance."""
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch

from check_corpus import Source, digest
import check_standard_libraries as libraries


def source(identifier: str) -> Source:
    return Source(identifier, Path('/tmp/source.jai'), 'hash', 'hash', identifier.split(':')[0], None)


class StandardLibraryAcceptanceTests(unittest.TestCase):
    def test_only_actual_entrypoints_are_selected(self):
        self.assertTrue(libraries.role(source('reference:modules/File/module.jai'), {})[2])
        self.assertTrue(libraries.role(source('reference:modules/Hash_Table.jai'), {})[2])
        self.assertFalse(libraries.role(source('reference:modules/File/unix.jai'), {})[2])
        self.assertFalse(libraries.role(source('focus-editor/focus:modules/Input/windows.jai'), {})[2])

    def test_example_library_is_distinguished_from_example_program(self):
        entry = libraries.role(source('roeyb1/sgpu:examples/modules/example/module.jai'), {})
        self.assertEqual(entry[0], 'example-library-entrypoint')
        self.assertTrue(entry[2])
        self.assertFalse(libraries.role(source('roeyb1/sgpu:examples/01_memory.jai'), {})[2])

    def test_negative_requires_reviewed_expectation(self):
        identifier = 'withlang-dev/open-jai:test/language/assert/failure.jai'
        self.assertEqual(libraries.role(source(identifier), {})[0], 'test-fixture')
        self.assertEqual(libraries.role(source(identifier), {identifier: {'negative': {'check': 'expected'}}})[0],
                         'expected-negative')

    def test_observed_upstream_failure_preserves_reviewed_contract(self):
        item = source('withlang-dev/open-jai:test/language/assert/failure.jai')
        observation = {'role': 'expected-negative', 'module_candidate': False,
                       'reviewed_negative': {'check': 'specific assertion'},
                       'upstream_expected_failure_evidence': [{'line': 7}]}
        row = libraries.inventory_row(item, observation)
        self.assertEqual(row['role'], 'expected-negative')
        self.assertEqual(row['reviewed_negative'], {'check': 'specific assertion'})
        self.assertFalse(row['selected_library_root'])

    def test_directives_ignore_comments_and_quoted_examples(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'input.jai'
            path.write_text('/* #import "Wrong" */\ns := "#load x";\n#import "Basic";\n'
                            '#library,system "libc";\n#if OS == .MACOS {}\n')
            item = source('reference:modules/Example.jai')
            item.path = path
            observation = libraries.observations(item)
            self.assertEqual([(d['kind'], d['literal_operand']) for d in observation['directives']],
                             [('import', 'Basic'), ('library', 'libc')])
            self.assertEqual(observation['platform_spellings'], ['MACOS'])

    def test_here_string_body_is_not_a_dependency(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'input.jai'
            path.write_text('text := #string END\n#import "NotAModule";\nEND\n#import "Basic";')
            item = source('reference:modules/Example.jai')
            item.path = path
            self.assertEqual([d['literal_operand'] for d in libraries.observations(item)['directives']
                              if d['kind'] == 'import'], ['Basic'])

    def test_named_module_custom_directory_precedes_standard_library(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            custom = root/'corpus/upstream/Ivo-Balbaert--The_Way_to_Jai/my_modules'
            custom.mkdir(parents=True)
            item = Source('Ivo-Balbaert/The_Way_to_Jai:my_modules/Linked_List.jai', custom/'Linked_List.jai',
                          'hash', 'hash', 'Ivo-Balbaert/The_Way_to_Jai', None)
            self.assertTrue(libraries.role(item, {})[2])
            with patch.object(libraries, 'ROOT', root), patch.object(libraries, 'module_search_paths', return_value=[standard]):
                self.assertEqual(libraries.ordered_module_paths(item), [custom, standard])

    def test_process_environment_cannot_inherit_compiler_policy(self):
                                      'JAI_RS_RUNTIME_ENTRY': '1', 'KEEP_ME': 'yes'}):
            environment = libraries.controlled_environment({'JAI_RS_PRELOAD': 'off'})
        self.assertNotIn('JAI_RS_CLANG', environment)
        self.assertNotIn('JAI_RS_RUNTIME_ENTRY', environment)
        self.assertEqual(environment['JAI_RS_PRELOAD'], 'off')
        self.assertEqual(environment['KEEP_ME'], 'yes')

    def test_runtime_library_configuration_does_not_define_program_entrypoint(self):
        with patch.object(libraries, 'module_search_paths', return_value=[]):
            environment = libraries.profile_environment(source('reference:modules/File/module.jai'), 'runtime-library')
        self.assertEqual(environment['JAI_RS_RUNTIME_ENTRY'], '0')
        self.assertEqual(environment['JAI_RS_RUNTIME_INITIALIZATION'], '1')
        self.assertEqual(environment['JAI_RS_RUNTIME_BACKTRACE'], '0')
        self.assertEqual(environment['JAI_RS_PRELOAD'], 'search')

    def test_native_build_command_is_not_permitted(self):
        with self.assertRaisesRegex(ValueError, 'source-only'):
                              'build', {}, 1, Path('/tmp'))

    def test_blank_stderr_prefix_does_not_discard_source_diagnostic(self):
        process = subprocess.CompletedProcess([], 1, stderr=b'\n/tmp/source.jai:2:3: error: expected type\n')
        with patch.object(libraries.subprocess, 'run', return_value=process):
            result = libraries.execute(Path('/tmp/jai-rs'), source('reference:modules/File/module.jai'),
                                       'check-library', {}, 1, Path('/tmp'))
        self.assertEqual(result['diagnostic'], '/tmp/source.jai:2:3: error: expected type')
        self.assertIsNone(result['failure_kind'])

    def test_rust_panic_is_retained_and_grouped_as_compiler_failure(self):
        process = subprocess.CompletedProcess([], 101, stderr=(
            b"\nthread 'main' (123) panicked at crates/jai-modules/src/enum_parameters.rs:473:69:\n"
            b'index out of bounds: the len is 0 but the index is 0\n'))
        with patch.object(libraries.subprocess, 'run', return_value=process):
            result = libraries.execute(Path('/tmp/jai-rs'), source('reference:modules/File/module.jai'),
                                       'check-library', {}, 1, Path('/tmp'))
        self.assertEqual(result['status'], 'failed')
        self.assertEqual(result['failure_kind'], 'compiler-panic')
        self.assertTrue(result['diagnostic'].startswith("thread 'main'"))
        self.assertEqual(result['reason'], 'compiler panic at crates/jai-modules/src/enum_parameters.rs:473:69: '
                                         'index out of bounds: the len is 0 but the index is 0')
        rows = [{'id': 'reference:modules/File/module.jai', 'profile': 'runtime-library',
                 'stages': {'check': result}}]
        group = libraries.failure_groups(rows, [])[0]
        self.assertEqual(group['failure_kind'], 'compiler-panic')
        self.assertEqual(group['distinct_locations'], 0)
        self.assertNotIn('(123)', group['diagnostic'])

    def test_signal_is_not_a_source_rejection(self):
        process = subprocess.CompletedProcess([], -9, stderr=b'')
        with patch.object(libraries.subprocess, 'run', return_value=process):
            result = libraries.execute(Path('/tmp/jai-rs'), source('reference:modules/File/module.jai'),
                                       'parse', {}, 1, Path('/tmp'))
        self.assertEqual(result['failure_kind'], 'compiler-signal')
        self.assertEqual(result['reason'], 'compiler terminated by signal 9')

    def test_snapshot_is_content_addressed_and_rejects_reference_tools(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            compiler = root/'target/debug/jai-rs'
            compiler.parent.mkdir(parents=True)
            compiler.write_bytes(b'self-authored snapshot fixture; never executed')
            with patch.object(libraries, 'ROOT', root):
                snapshot = libraries.freeze_compiler(compiler)
                self.assertEqual(snapshot.parent.name, digest(compiler))
                self.assertEqual(snapshot.read_bytes(), compiler.read_bytes())
                self.assertEqual(libraries.freeze_compiler(compiler), snapshot)
                with self.assertRaisesRegex(ValueError, 'our built'):

    def test_dependency_diagnostics_group_verified_source_not_root(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'module.jai'
            path.write_text('self-authored fixture')
            dependency = Source('reference:modules/module.jai', path, digest(path), digest(path), 'reference', None)
            rows = [{'id': identifier, 'profile': 'preload', 'stages': {
                'check': {'status': 'failed', 'diagnostic': f'{path}:7:8: error: expected type', 'reason': ''}}}
                for identifier in ('reference:modules/File/module.jai', 'reference:modules/Thread/module.jai')]
            groups = libraries.failure_groups(rows, [dependency])
            self.assertEqual(groups[0]['root_count'], 2)
            self.assertEqual(groups[0]['distinct_locations'], 1)
            self.assertTrue(all(p['source_hash_verified'] for p in groups[0]['locations']))
            self.assertEqual(groups[0]['locations'][0]['diagnostic_source'], dependency.id)

    def test_unlocated_failures_do_not_invent_a_shared_location(self):
        rows = [{'id': identifier, 'profile': 'preload', 'stages': {
            'check': {'status': 'failed', 'diagnostic': 'unlocated compiler error', 'reason': ''}}}
            for identifier in ('reference:modules/File/module.jai', 'reference:modules/Thread/module.jai')]
        group = libraries.failure_groups(rows, [])[0]
        self.assertEqual(group['root_count'], 2)
        self.assertEqual(group['distinct_locations'], 0)
        self.assertEqual(group['unverified_root_locations'], 2)


if __name__ == '__main__':
    unittest.main()
