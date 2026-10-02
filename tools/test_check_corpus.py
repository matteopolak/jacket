import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import subprocess

from check_corpus import (Evidence, Source, Stage, Status, classify, evaluate,
                          execute, inside, inventory, totals, validate_cases)

class AcceptanceTests(unittest.TestCase):
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

    def test_absent_output_timeout_and_signal_are_not_passes(self):
        with TemporaryDirectory() as directory:
            root = Path(directory); source = self.source(root)
            with patch('check_corpus.subprocess.run', return_value=subprocess.CompletedProcess([], 0, b'IR source', b'')):
                result = execute(root/'jai-rs', source, Stage.CODEGEN, 1, root/'out')
                self.assertEqual(result.status, Status.FAILED)
            with patch('check_corpus.subprocess.run', side_effect=subprocess.TimeoutExpired([], 1)):
                self.assertEqual(execute(root/'jai-rs', source, Stage.CHECK, 1, root/'out').status, Status.FAILED)

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
