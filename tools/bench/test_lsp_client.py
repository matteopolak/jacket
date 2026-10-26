import json
import unittest
from pathlib import Path

from bench import cmd_lsp
from bench import lsp_report
from bench import lsp_session
from bench.lsp_client import FrameDecoder, FrameError, capabilities, encode, parse_cpu_time
from bench.lsp_session import Document, Probe, ProbeFailed, Session, answer
from bench.results import Report, Result, Status
from bench.stats import Stat


class FramingTests(unittest.TestCase):
    def test_encode_counts_bytes_not_characters(self):
        header, _, body = encode({'a': 'é'}).partition(b'\r\n\r\n')
        self.assertEqual(header, b'Content-Length: %d' % len(body))
        self.assertEqual(json.loads(body), {'a': 'é'})

    def test_decoder_handles_split_and_joined_chunks(self):
        data = encode({'id': 1}) + encode({'id': 2, 'x': 'ü'})
        decoder = FrameDecoder()
        out = []
        for i in range(0, len(data), 3):
            out += decoder.push(data[i:i + 3])
        self.assertEqual([m['id'] for m in out], [1, 2])
        self.assertEqual([m['id'] for m in FrameDecoder().push(data)], [1, 2])

    def test_decoder_ignores_other_headers_and_rejects_bad_ones(self):
        body = b'{"id":7}'
        frame = b'content-type: x\r\nCONTENT-LENGTH: %d\r\n\r\n' % len(body) + body
        self.assertEqual(FrameDecoder().push(frame), [{'id': 7}])
        with self.assertRaises(FrameError):
            FrameDecoder().push(b'Foo: 1\r\n\r\n{}')
        with self.assertRaises(FrameError):
            FrameDecoder().push(b'Content-Length: x\r\n\r\n{}')

    def test_ps_cpu_times(self):
        self.assertEqual(parse_cpu_time('0:01.50'), 1.5)
        self.assertEqual(parse_cpu_time('1:02:03'), 3723)
        self.assertEqual(parse_cpu_time('1-00:00:10'), 86410)

    def test_capabilities_pull_or_push(self):
        self.assertIn('diagnostic', capabilities(True)['textDocument'])
        self.assertNotIn('diagnostic', capabilities(False)['textDocument'])


class AnswerTests(unittest.TestCase):
    def test_errors_become_the_right_exception(self):
        self.assertEqual(answer({'result': [1]}), [1])
        with self.assertRaises(lsp_session.Unsupported):
            answer({'error': {'code': -32601, 'message': 'no such method'}})
        with self.assertRaises(ProbeFailed):
            answer({'error': {'code': -32603, 'message': 'boom'}})


class FakeClient:
    notifications: list = []


class CapabilityTests(unittest.TestCase):
    def session(self, capabilities_):
        document = Document(Path(__file__), '', {})
        session = Session(FakeClient(), document.path.parent, document, False, 1, 1)
        session.server_capabilities = capabilities_
        return session

    def test_dotted_capability_paths(self):
        session = self.session({'hoverProvider': True, 'semanticTokensProvider': {'full': {'delta': True}},
                                'renameProvider': {'prepareProvider': True}, 'noProvider': False})
        self.assertTrue(session.supports(None))
        self.assertTrue(session.supports('hoverProvider'))
        self.assertTrue(session.supports('semanticTokensProvider.full.delta'))
        self.assertTrue(session.supports('renameProvider'))
        self.assertFalse(session.supports('noProvider'))
        self.assertFalse(session.supports('semanticTokensProvider.full.missing'))
        self.assertFalse(session.supports('documentFormattingProvider'))


class AggregationTests(unittest.TestCase):
    def record(self, outcomes):
        return cmd_lsp.SessionRecord(outcomes, 1.0, 0.5, 100.0, {'document_lines': 10})

    def test_samples_are_pooled_across_sessions(self):
        a = self.record({'hover_first': cmd_lsp.Outcome(samples=[0.010], cpu=0.01, rss_mib=100)})
        b = self.record({'hover_first': cmd_lsp.Outcome(samples=[0.030, 0.020], cpu=0.02, rss_mib=120)})
        result = cmd_lsp.pooled('hover_first', [a, b])
        self.assertEqual((result.wall.n, result.wall.median, result.wall.max), (3, 0.020, 0.030))
        self.assertEqual(result.info['samples'], 3)

    def test_failure_in_some_sessions_is_noted_but_the_rest_count(self):
        good = self.record({'x': cmd_lsp.Outcome(samples=[0.1])})
        bad = self.record({'x': cmd_lsp.Outcome(Status.FAILED, 'timeout')})
        result = cmd_lsp.pooled('x', [good, bad])
        self.assertEqual((result.status, result.info['failed_sessions']), (Status.OK, 1))

    def test_all_failed_or_unsupported(self):
        bad = self.record({'x': cmd_lsp.Outcome(Status.FAILED, 'timeout')})
        self.assertEqual(cmd_lsp.pooled('x', [bad]).status, Status.FAILED)
        skipped = self.record({'x': cmd_lsp.Outcome(Status.SKIPPED, 'server does not advertise y')})
        self.assertEqual(cmd_lsp.pooled('x', [skipped]).status, Status.SKIPPED)

    def test_empty_answers_are_counted(self):
        a = self.record({'x': cmd_lsp.Outcome(samples=[0.1], extra={'empty_answers': 1})})
        self.assertEqual(cmd_lsp.pooled('x', [a]).info['empty_answers'], 1)


class ReportTests(unittest.TestCase):
    def test_markdown_shows_percentiles_and_what_was_not_timed(self):
        stat = Stat.of([0.010, 0.020, 0.030])
        report = Report('lsp', {'repeat': 3, 'warmup': 1, 'diagnostics': 'push'}, {'jailsp': {'command': ['jailsp']}},
                        {'jailsp/gen/hover_first': Result(wall=stat, info={'samples': 3}),
                         'jailsp/gen/formatting': Result.skipped('server does not advertise documentFormattingProvider'),
                         'jailsp/gen/session': Result(wall=Stat.of([2.0]), info={'document_lines': 99})},
                        machine={'cpus': 10})
        text = lsp_report.render(report)
        self.assertIn('| hover_first | 20.0 | 29.0 |', text)
        self.assertIn('document 99 lines', text)
        self.assertIn('`jailsp/gen/formatting`: skipped: server does not advertise', text)

    def test_servers_are_named_commands(self):
        self.assertEqual(cmd_lsp.parse_servers(['a=/x/y --stdio', 'b=z']), {'a': ['/x/y', '--stdio'], 'b': ['z']})
        self.assertEqual(list(cmd_lsp.parse_servers([])), ['jailsp'])
        with self.assertRaises(SystemExit):
            cmd_lsp.parse_servers(['nocommand'])

    def test_probes_have_unique_names(self):
        names = [p.name for p in lsp_session.PROBES] + list(lsp_session.SESSION_PROBES)
        self.assertEqual(len(names), len(set(names)))
        self.assertTrue(all(isinstance(p, Probe) for p in lsp_session.PROBES))


if __name__ == '__main__':
    unittest.main()
