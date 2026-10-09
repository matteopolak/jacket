import json
import unittest

import benchgen
from lsp_bench import (FrameDecoder, FrameError, METRICS, capabilities, compare, cell, delete_lines_edit, encode,
                       insert_edit, mask_line, markdown, parse_cpu_time, parse_servers, pick_positions,
                       summarize, summarize_metric, typing_script, utf16_col)

SOURCE = '''#import "Basic";

Vec :: struct { x: int; y: int; }

helper :: (v: Vec) -> int {
    // comment with ignored_word
    total := v.x + 1;
    s := "string.with.dots";
    return total * v.y;
}

second :: (n: int) -> int {
    out := n + helper(.{1, 2});
    log_it(out);
    return out;
}
'''


def run(seconds, cpu=0.0, rss=100.0):
    return {'seconds': seconds, 'cpu_user': cpu, 'cpu_sys': 0.0, 'rss_mib': rss}


def report(seconds, rss, cpu=1.0):
    metric = summarize_metric([run(seconds, cpu, rss)])
    return {'results': {'jailsp/focus/pull': {'metrics': {'hover_first': metric}, 'peak_rss_mib': rss,
                                              'cpu_seconds': cpu, 'info': {}}}}


class FramingTests(unittest.TestCase):
    def test_encode_counts_bytes_not_characters(self):
        frame = encode({'a': 'é'})
        header, _, body = frame.partition(b'\r\n\r\n')
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


class PositionTests(unittest.TestCase):
    def test_masking_keeps_length_and_hides_comments_and_strings(self):
        line = 'a := "b.c"; // d.e'
        masked, depth = mask_line(line)
        self.assertEqual(len(masked), len(line))
        self.assertEqual(masked.rstrip(), 'a := "   ";')
        self.assertEqual(mask_line('x /* y', 0)[1], 1)
        self.assertEqual(mask_line('z */ w', 1), ('     w', 0))

    def test_utf16_columns_count_surrogate_pairs(self):
        self.assertEqual(utf16_col('a\U0001F600b', 2), 3)
        self.assertEqual(utf16_col('aéb', 2), 2)

    def test_picks_are_deterministic_and_inside_bodies(self):
        first = pick_positions(SOURCE)
        self.assertEqual(first, pick_positions(SOURCE))
        lines = SOURCE.split('\n')
        hover = first['hover']
        self.assertEqual(lines[hover['line']][hover['character']:][:len(hover['name'])], hover['name'])
        self.assertNotEqual(hover['name'], 'ignored_word')
        self.assertIn(first['declaration']['name'], ('helper', 'second'))
        self.assertTrue(lines[first['insert_line']].strip() != '' or lines[first['insert_line']] == '')
        member = first['member']
        self.assertEqual(lines[member['line']][member['character'] - 1], '.')
        self.assertNotIn('string', member['base'])

    def test_fraction_moves_the_pick_and_no_bodies_gives_none(self):
        early, late = pick_positions(SOURCE, 0.0), pick_positions(SOURCE, 0.99)
        self.assertLess(early['hover']['line'], late['hover']['line'])
        self.assertIsNone(pick_positions('x :: 1;\n'))

    def test_generated_programs_have_positions(self):
        text = benchgen.generate(2000, 1)['main.jai']
        pick = pick_positions(text)
        self.assertIsNotNone(pick)
        self.assertIsNotNone(pick['member'])

    def test_one_file_generation_is_a_single_document(self):
        files = benchgen.generate(3000, 1, 1)
        self.assertEqual(list(files), ['main.jai'])
        self.assertGreater(len(files['main.jai']), 50_000)

    def test_edits(self):
        self.assertEqual(insert_edit(3, 4, 'x')['range']['end'], {'line': 3, 'character': 4})
        self.assertEqual(delete_lines_edit(2)['range']['end'], {'line': 3, 'character': 0})
        self.assertEqual(typing_script('  ', 'ab'), [(2, 'a'), (3, 'b')])


class StatisticsTests(unittest.TestCase):
    def test_first_session_is_cold_and_the_rest_give_the_median(self):
        row = summarize_metric([run(5.0, 1.0, 90), run(2.0, 0.5, 100), run(3.0, 0.7, 80), run(2.5, 0.6, 85)])
        self.assertEqual((row['cold'], row['warm']), (5.0, 2.5))
        self.assertEqual((row['cpu_cold'], row['cpu_warm'], row['rss_warm']), (1.0, 0.6, 85))
        self.assertEqual((row['min'], row['max'], row['status']), (2.0, 5.0, 'ok'))

    def test_failures_are_statuses_not_numbers(self):
        row = summarize_metric([{'status': 'limit', 'message': 'too big'}, {'status': 'limit', 'message': ''}])
        self.assertEqual((row['cold'], row['warm'], row['status'], row['message']), (None, None, 'limit', 'too big'))
        mixed = summarize_metric([run(1.0), {'status': 'timeout', 'message': 'slow'}])
        self.assertEqual((mixed['cold'], mixed['warm'], mixed['status']), (1.0, 1.0, 'timeout'))

    def test_summary_totals_cpu_and_peak(self):
        sessions = [{'metrics': {'hover_first': run(1.0)}, 'peak_rss_mib': 50, 'cpu_user_s': 1.0, 'cpu_sys_s': 0.5, 'info': {}},
                    {'metrics': {'hover_first': run(2.0)}, 'peak_rss_mib': 70, 'cpu_user_s': 2.0, 'cpu_sys_s': 0.5, 'info': {}}]
        row = summarize(sessions)
        self.assertEqual((row['peak_rss_mib'], row['cpu_seconds'], row['cpu_seconds_cold']), (70, 2.5, 1.5))

    def test_ps_cpu_times(self):
        self.assertEqual(parse_cpu_time('0:01.50'), 1.5)
        self.assertEqual(parse_cpu_time('1:02:03'), 3723)
        self.assertEqual(parse_cpu_time('1-00:00:10'), 86410)


class CompareTests(unittest.TestCase):
    def test_regressions_need_relative_and_absolute_growth(self):
        old = report(2.0, 500)
        self.assertEqual(compare(old, report(2.1, 510), 0.10, 0.1, 16), [])
        self.assertEqual(len(compare(old, report(2.6, 500), 0.10, 0.1, 16)), 1)
        self.assertTrue(any('peak_rss_mib' in line for line in compare(old, report(2.0, 700), 0.10, 0.1, 16)))
        self.assertEqual(compare(report(0.001, 50), report(0.003, 50), 0.10, 0.005, 16), [])

    def test_cpu_and_step_rss_regressions(self):
        old = report(1.0, 500, cpu=1.0)
        new = report(1.0, 500, cpu=2.0)
        self.assertTrue(any(line.endswith('(+100%)') and 'cpu_seconds' in line for line in compare(old, new, 0.1, 0.1, 16)))
        bigger = report(1.0, 500)
        bigger['results']['jailsp/focus/pull']['metrics']['hover_first']['rss_warm'] = 900
        self.assertTrue(any('rss' in line for line in compare(old, bigger, 0.1, 0.1, 16)))

    def test_a_metric_that_stopped_working_regresses_but_unsupported_does_not(self):
        old = report(1.0, 100)
        broken = report(1.0, 100)
        broken['results']['jailsp/focus/pull']['metrics']['hover_first'] = summarize_metric([{'status': 'timeout', 'message': ''}])
        self.assertEqual(len(compare(old, broken, 0.1, 0.1, 16)), 1)
        broken['results']['jailsp/focus/pull']['metrics']['hover_first'] = summarize_metric([{'status': 'unsupported', 'message': ''}])
        self.assertEqual(compare(old, broken, 0.1, 0.1, 16), [])

    def test_keys_of_other_servers_are_not_compared(self):
        other = report(9.0, 900)
        other['results'] = {'jails/focus/pull': other['results']['jailsp/focus/pull']}
        self.assertEqual(compare(report(1.0, 100), other, 0.1, 0.1, 16), [])


class ReportTests(unittest.TestCase):
    def test_markdown_lists_statuses_and_every_workload(self):
        names = list(METRICS)
        rows = {'jailsp/focus/pull': {'metrics': {n: summarize_metric([run(0.5, 0.4, 120)]) for n in names},
                                      'peak_rss_mib': 130.0, 'cpu_seconds': 1.2,
                                      'info': {'document_bytes': 2048, 'diagnostic_codes': {'jai-limit': 1},
                                               'diagnostics_count': 1, 'hover_result': False}}}
        rows['jailsp/focus/pull']['metrics']['hover_first'] = summarize_metric([{'status': 'unsupported', 'message': 'no'}])
        rows['jailsp/focus/pull']['metrics']['references'] = summarize_metric(
            [{'status': 'limit', 'message': 'document is not open'}])
        text = markdown({'date': '2026-10-09T00:00:00', 'machine': {'machine': 'arm64', 'cpus': 10, 'system': 'Darwin', 'release': '27'},
                         'servers': {'jailsp': {'version': 'jailsp 0.6', 'commit': 'abc', 'dirty': False, 'rustc': 'rustc'}},
                         'settings': {'repeat': 3}, 'results': rows, 'failures': {'x/y/pull': 'missing'}})
        self.assertIn('| jailsp/focus/pull | 2 |', text)
        self.assertIn('unsupported', text)
        self.assertIn('Warm CPU time', text)
        self.assertIn('{\'jai-limit\': 1}', text)
        self.assertIn('`jailsp/focus/pull` references: limit', text)
        self.assertIn('`x/y/pull`: missing', text)
        self.assertEqual(cell(None, 'warm'), '-')

    def test_capabilities_pull_or_push(self):
        self.assertIn('diagnostic', capabilities('pull')['textDocument'])
        self.assertNotIn('diagnostic', capabilities('push')['textDocument'])

    def test_servers_are_named_commands(self):
        self.assertEqual(parse_servers(['a=/x/y --stdio', 'b=z']), {'a': ['/x/y', '--stdio'], 'b': ['z']})
        self.assertEqual(list(parse_servers([])), ['jailsp'])
        with self.assertRaises(SystemExit):
            parse_servers(['nocommand'])


if __name__ == '__main__':
    unittest.main()
