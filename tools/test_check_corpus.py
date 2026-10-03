import json
import hashlib
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import subprocess

from check_corpus import (Evidence, Source, Stage, Status, classify, evaluate,
                          execute, inside, inventory, totals, validate_cases, module_search_paths,
                          compiler_fingerprint, annotate_observed_compiler_inputs)

class AcceptanceTests(unittest.TestCase):
    def test_frozen_binary_inputs_are_observed_worktree_not_verified_build_inputs(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root/'jai-rs'; binary.write_bytes(b'frozen compiler')
            source = root/'Cargo.toml'; source.write_text('first observed tree')
            first = compiler_fingerprint(root, binary)
            source.write_text('later observed tree')
            second = compiler_fingerprint(root, binary)
            self.assertEqual(first['binary_sha256'], second['binary_sha256'])
            self.assertNotEqual(first['inputs_sha256'], second['inputs_sha256'])
            self.assertEqual(second['inputs_provenance'], 'observed-worktree')
            self.assertIs(second['build_inputs_verified'], False)
            legacy = {'compiler': {key: value for key, value in first.items()
                                   if key not in {'inputs_provenance', 'build_inputs_verified'}}}
            annotate_observed_compiler_inputs(legacy)
            self.assertEqual(legacy['compiler'], first)
            binary.write_bytes(b'different compiler')
            with self.assertRaisesRegex(ValueError, 'missing or changed'):
                annotate_observed_compiler_inputs(legacy)

    def source(self, base):
        path = base / 'source.jai'
        path.write_text('main :: () {}')
        return Source('test:source.jai', path, 'same', 'same', 'test', None)

    def test_negative_configuration_cannot_match_every_error(self):
        case = {'id':'negative.jai', 'kind':'negative', 'target':'host', 'dependencies':[], 'negative':{'check':''}}
        with self.assertRaises(ValueError): validate_cases({'format':1,'cases':[case]})
        case['negative']['check'] = 'intended failure'
        self.assertIn('negative.jai', validate_cases({'format':1,'cases':[case]}))

    def test_expected_rejections_require_exact_diagnostic_and_normal_failure(self):
        self.assertEqual(classify(1, 'type mismatch here', 'type mismatch'), Status.EXPECTED_REJECTION)
        for code, text in [(1, 'unsupported syntax'), (-11, 'type mismatch'), (2, 'type mismatch')]:
            self.assertEqual(classify(code, text, 'type mismatch'), Status.FAILED)
        self.assertEqual(classify(0, '', 'type mismatch'), Status.UNEXPECTED_ACCEPTANCE)

    def test_no_lexical_pass_becomes_compilation_and_failure_stops_pipeline(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            with patch('check_corpus.execute', side_effect=[Evidence(Status.PASSED), Evidence(Status.PASSED), Evidence(Status.FAILED)]) as run:
                result = evaluate(self.source(root), {}, root/'jai-rs', Stage.BUILD, 1, root/'output')
            self.assertEqual(run.call_count, 3)
            self.assertEqual(result.stages['lex'].status, Status.PASSED)
            self.assertEqual(result.stages['parse'].status, Status.PASSED)
            for stage in ['codegen', 'build']:
                self.assertEqual(result.stages[stage].status, Status.NOT_RUN)
            self.assertEqual(totals([result])['check'], {'failed': 1})

    def test_parse_stage_is_measured_and_failure_stops_check(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            with patch('check_corpus.execute', side_effect=[Evidence(Status.PASSED), Evidence(Status.FAILED)]) as run:
                result = evaluate(self.source(root), {}, root/'jai-rs', Stage.CHECK, 1, root/'out')
            self.assertEqual([c.args[2] for c in run.call_args_list], [Stage.LEX, Stage.PARSE])
            self.assertEqual(result.stages['parse'].status, Status.FAILED)
            self.assertEqual(result.stages['check'].status, Status.NOT_RUN)

    def test_parse_only_selection_and_negative_expectation(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            case = {'kind':'negative', 'negative':{'parse':'expected identifier'}}
            with patch('check_corpus.execute', side_effect=[Evidence(Status.PASSED), Evidence(Status.EXPECTED_REJECTION)]) as run:
                result = evaluate(self.source(root), case, root/'jai-rs', Stage.PARSE, 1, root/'out')
            self.assertEqual(run.call_args.args[-1], 'expected identifier')
            self.assertEqual(result.stages['parse'].status, Status.EXPECTED_REJECTION)
            self.assertEqual(result.stages['check'].status, Status.NOT_RUN)

    def test_source_drift_blocks_every_execution(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root); source.sha256 = 'modified'
            with patch('check_corpus.execute') as run:
                result = evaluate(source, {}, root/'jai-rs', Stage.BUILD, 1, root/'out')
            run.assert_not_called()
            self.assertEqual(result.stages['check'].status, Status.BLOCKED)

    def test_no_main_module_check_is_separate_from_build(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            with patch('check_corpus.execute', return_value=Evidence(Status.PASSED)) as run:
                result = evaluate(self.source(root), {'kind':'module'}, root/'jai-rs', Stage.BUILD, 1, root/'out')
            self.assertEqual(run.call_count, 4)
            self.assertEqual(result.stages['check'].status, Status.PASSED)
            self.assertEqual(result.stages['build'].status, Status.BLOCKED)
            self.assertEqual(result.stages['run'].status, Status.BLOCKED)

    def test_module_check_uses_library_policy_and_program_keeps_entry_policy(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            with patch('check_corpus.subprocess.run', return_value=subprocess.CompletedProcess([], 0, b'', b'')) as run:
                module = execute(root/'jai-rs', self.source(root), Stage.CHECK, 1, root/'out', library=True)
                self.assertEqual(module.status, Status.PASSED)
                self.assertEqual(run.call_args.args[0][1], 'check-library')
                execute(root/'jai-rs', self.source(root), Stage.CHECK, 1, root/'out')
                self.assertEqual(run.call_args.args[0][1], 'check')

    def test_module_paths_are_real_ordered_and_override_ambient_configuration(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            project = root/'corpus/upstream/owner--project'
            local = project/'examples/modules'; local.mkdir(parents=True)
            modules = project/'Modules'; modules.mkdir()
            path = project/'examples/first.jai'; path.write_text('main :: () {}')
            source = Source('owner/project:examples/first.jai',path,'same','same','owner/project',None)
            with patch('check_corpus.ROOT',root):
                self.assertEqual(module_search_paths(source),[local.resolve(),modules.resolve(),standard.resolve()])
                with patch('check_corpus.os.environ',{'JAI_RS_MODULE_PATH':'unrecorded-ambient','JAI_RS_STDLIB':'ambient-stdlib','JAI_RS_RUNTIME_SUPPORT':'search'}), patch('check_corpus.subprocess.run',return_value=subprocess.CompletedProcess([],0,b'',b'')) as run:
                    evidence = execute(root/'jai-rs',source,Stage.CHECK,1,root/'out',library=True)
                self.assertEqual(run.call_args.kwargs['env']['JAI_RS_MODULE_PATH'],evidence.environment['JAI_RS_MODULE_PATH'])
                self.assertNotIn('unrecorded-ambient',evidence.environment['JAI_RS_MODULE_PATH'])
                self.assertNotIn('JAI_RS_STDLIB',run.call_args.kwargs['env'])
                self.assertEqual(evidence.environment['JAI_RS_PRELOAD'],'off')
                self.assertEqual(evidence.environment['JAI_RS_RUNTIME_SUPPORT'],'off')
                with patch('check_corpus.subprocess.run',return_value=subprocess.CompletedProcess([],0,b'',b'')):
                    bootstrap = execute(root/'jai-rs',source,Stage.CHECK,1,root/'out',bootstrap='search')
                self.assertEqual(bootstrap.environment['JAI_RS_PRELOAD'],'search')
                self.assertEqual(bootstrap.environment['JAI_RS_STDLIB'],str(standard.resolve()))

    def test_absent_output_timeout_and_signal_are_not_passes(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root)
            with patch('check_corpus.subprocess.run', return_value=subprocess.CompletedProcess([], 0, b'IR source', b'')):
                result = execute(root/'jai-rs', source, Stage.CODEGEN, 1, root/'out')
                self.assertEqual(result.status, Status.FAILED)
            with patch('check_corpus.subprocess.run', side_effect=subprocess.TimeoutExpired([], 1)):
                self.assertEqual(execute(root/'jai-rs', source, Stage.CHECK, 1, root/'out').status, Status.FAILED)

    def test_old_stage_artifact_cannot_satisfy_success_without_output(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root); output = root/'output'
            for stage in (Stage.CODEGEN, Stage.BUILD):
                output.write_bytes(b'old LLVM output from earlier stage')
                output.chmod(0o755)
                with patch('check_corpus.subprocess.run',return_value=subprocess.CompletedProcess([],0,b'',b'')):
                    evidence = execute(root/'jai-rs',source,stage,1,output,artifact_root=root)
                self.assertEqual(evidence.status,Status.FAILED)
                self.assertEqual(evidence.reason,'compiler did not produce output')
                self.assertIsNone(evidence.output_sha256)
                self.assertFalse(output.exists())

    def test_fresh_artifact_hash_and_executable_build_provenance(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root); output = root/'output'
            output.write_bytes(b'stale IR')
            fresh = b'fresh generated artifact'
            def produce(command, **kwargs):
                artifact = Path(command[-1])
                self.assertFalse(artifact.exists())
                artifact.write_bytes(fresh)
                return subprocess.CompletedProcess(command,0,b'',b'')
            with patch('check_corpus.subprocess.run',side_effect=produce):
                ir = execute(root/'jai-rs',source,Stage.CODEGEN,1,output,artifact_root=root)
                build = execute(root/'jai-rs',source,Stage.BUILD,1,output,artifact_root=root)
            self.assertEqual(ir.status,Status.PASSED)
            self.assertEqual(ir.output_sha256,hashlib.sha256(fresh).hexdigest())
            self.assertEqual(build.status,Status.FAILED)
            self.assertEqual(build.reason,'build artifact is not executable')
            def executable(command, **kwargs):
                result = produce(command,**kwargs)
                Path(command[-1]).chmod(0o755)
                return result
            with patch('check_corpus.subprocess.run',side_effect=executable):
                build = execute(root/'jai-rs',source,Stage.BUILD,1,output,artifact_root=root)
            self.assertEqual(build.status,Status.PASSED)
            self.assertEqual(build.output_sha256,hashlib.sha256(fresh).hexdigest())
            with patch('check_corpus.subprocess.run',return_value=subprocess.CompletedProcess([],0,b'',b'')):
                checked = execute(root/'jai-rs',source,Stage.CHECK,1,output)
            self.assertIsNone(checked.output_sha256)

    def test_artifact_guard_preserves_source_and_aliases(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root)
            alias = root/'alias'; alias.symlink_to(source.path)
            with patch('check_corpus.subprocess.run') as run:
                for output in (source.path,alias):
                    evidence = execute(root/'jai-rs',source,Stage.CODEGEN,1,output,artifact_root=root)
                    self.assertEqual(evidence.status,Status.BLOCKED)
                mismatch = execute(root/'jai-rs',source,Stage.CODEGEN,1,root/'output',artifact_root=root/'another')
                self.assertEqual(mismatch.status,Status.BLOCKED)
            run.assert_not_called()
            self.assertEqual(source.path.read_text(),'main :: () {}')
            self.assertTrue(alias.is_symlink())

    def test_inventory_rejects_unlisted_sources_and_path_escape(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); (root/'corpus').mkdir(); (root/'reference').mkdir()
            (root/'corpus/reference-inputs.json').write_text(json.dumps({'format':1,'files':[]}))
            (root/'corpus/upstreams.json').write_text(json.dumps({'format':1,'projects':[]}))
            self.assertEqual(inventory(root), [])
            with self.assertRaises(ValueError): inventory(root)
            for path in ['../escape', '/absolute', 'a/../../escape', 'a\\escape']:
                with self.assertRaises(ValueError): inside(root,path)

if __name__ == '__main__':
    unittest.main()
