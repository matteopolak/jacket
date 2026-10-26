import json
import os
import stat
import tempfile
import unittest
from pathlib import Path


import compare_compilers as compare
from toolkit.repo import cargo_binary
import compare_grid as grid
import compare_rewrite as rewrite
from compare_core import (A_ONLY, AGREE, B_ONLY, BOTH_REJECT, CRASH, FAILURE, OK, OUTPUT_DIFFERS, REJECTED, RUNTIME,
                          TIMEOUT, Outcome, Program, Record, classify, cluster_by_error, cluster_by_tag,
                          cluster_rejections, first_error, normalise_error, reduce_lines, summarise)

ROOT = Path(__file__).resolve().parents[2]


def find_jaic():
    candidates = [os.environ.get('JAIC')]
    candidates += [str(cargo_binary('jaic', 'release')), str(cargo_binary('jaic'))]
    return next((c for c in candidates if c and Path(c).is_file()), None)


def ok(stdout='1\n'):
    return Outcome(OK, 0, stdout)


def rejected(error='error: nope'):
    return Outcome(REJECTED, 1, '', error)


class NormalisationTests(unittest.TestCase):
    def test_drops_location_colour_and_paths(self):
        raw = '\x1b[1;31m/tmp/work/prog.jai:12:5: error: type mismatch in /tmp/x/y.jai at 3:4\x1b[0m'
        self.assertEqual(normalise_error(raw), 'error: type mismatch in FILE at L:C')

    def test_same_complaint_elsewhere_compares_equal(self):
        self.assertEqual(normalise_error('a/b.jai:1:2: error: cannot infer'), normalise_error('c.jai:90:7: error: cannot infer'))

    def test_first_error_skips_timing_lines_and_notes(self):
        stderr = 'jaic-timing: front end 0.1 1\nx.jai:3:4: error: first\nx.jai:5:6: error: second\n'
        self.assertEqual(first_error(stderr), 'error: first')

    def test_first_error_falls_back_to_first_line(self):
        self.assertEqual(first_error('something odd happened\n'), 'something odd happened')
        self.assertEqual(first_error(''), '')


class ClassificationTests(unittest.TestCase):
    def test_pairs(self):
        self.assertEqual(classify(ok(), ok()), AGREE)
        self.assertEqual(classify(ok('1\n'), ok('2\n')), OUTPUT_DIFFERS)
        self.assertEqual(classify(ok(), rejected()), A_ONLY)
        self.assertEqual(classify(rejected(), ok()), B_ONLY)
        self.assertEqual(classify(rejected('error: a'), rejected('error: b')), BOTH_REJECT)
        self.assertEqual(classify(ok(), Outcome(TIMEOUT)), FAILURE)
        self.assertEqual(classify(Outcome(CRASH, -11), rejected()), FAILURE)

    def test_runtime_failures_compare_as_a_fact_not_by_exit_code(self):
        a, b = Outcome(RUNTIME, 1, 'x\n'), Outcome(RUNTIME, 134, 'x\n')
        self.assertEqual(classify(a, b), AGREE)
        self.assertEqual(classify(a, Outcome(RUNTIME, 1, 'y\n')), OUTPUT_DIFFERS)
        self.assertEqual(classify(ok('x\n'), Outcome(RUNTIME, 1, 'x\n')), OUTPUT_DIFFERS)

    def test_exit_codes_matter_for_successful_runs(self):
        self.assertEqual(classify(ok(), Outcome(OK, 3, '1\n')), OUTPUT_DIFFERS)

    def test_different_messages_do_not_disagree(self):
        record = Record(Program('p', 'x'), rejected('error: a'), rejected('error: b'))
        self.assertFalse(record.disagrees)
        self.assertTrue(record.diagnostics_differ)


class ClusterTests(unittest.TestCase):
    def records(self):
        def record(name, tags, a, b):
            return Record(Program(name, 'x\n' * len(name), tuple(tags)), a, b)
        return [
            record('aa', ['kind:s', 'context:decl'], ok(), rejected('error: cannot infer')),
            record('b', ['kind:s', 'context:arg'], ok(), rejected('error: cannot infer')),
            record('cccc', ['kind:i', 'context:decl'], ok('1\n'), ok('2\n')),
            record('ddd', ['kind:i', 'context:arg'], ok(), ok()),
            record('ee', ['kind:i', 'context:ret', 'variant:near_miss'], rejected('error: bad'), rejected('error: bad')),
            record('fff', ['kind:i', 'context:ret'], rejected('error: bad'), rejected('error: bad')),
        ]

    def test_summary_counts(self):
        counts = summarise(self.records())
        self.assertEqual((counts[A_ONLY], counts[OUTPUT_DIFFERS], counts[AGREE], counts[BOTH_REJECT]), (2, 1, 1, 2))

    def test_error_clusters_group_by_message_and_show_the_smallest(self):
        clusters = cluster_by_error(self.records())
        self.assertEqual([(c.verdict, len(c.records)) for c in clusters], [(A_ONLY, 2), (OUTPUT_DIFFERS, 1)])
        self.assertEqual(clusters[0].key, 'error: cannot infer')
        self.assertEqual(clusters[0].smallest.program.name, 'b')

    def test_tag_clusters_rank_by_share(self):
        groups = {tag: (len(found), total) for tag, found, total in cluster_by_tag(self.records())}
        self.assertEqual(groups['kind:s'], (2, 2))
        self.assertEqual(groups['kind:i'], (1, 4))
        self.assertNotIn('variant:near_miss', groups)

    def test_rejection_clusters_leave_out_near_misses(self):
        clusters = cluster_rejections(self.records())
        self.assertEqual([(c.key, [r.program.name for r in c.records]) for c in clusters], [('error: bad', ['fff'])])


class ReducerTests(unittest.TestCase):
    def test_keeps_only_the_lines_that_matter(self):
        lines = [f'line {i}' for i in range(20)]
        reduced = reduce_lines(lines, lambda c: 'line 3' in c and 'line 14' in c)
        self.assertEqual(reduced, ['line 3', 'line 14'])

    def test_respects_the_budget(self):
        calls = []
        reduce_lines(['a'] * 50, lambda c: calls.append(1) or True, budget=5)
        self.assertLessEqual(len(calls), 5)


class GridTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.programs = grid.grid()

    def test_names_are_unique_and_filesystem_safe(self):
        names = [p.name for p in self.programs]
        self.assertEqual(len(names), len(set(names)))
        for name in names:
            self.assertRegex(name, r'^[\w.-]+$')

    def test_programs_are_complete(self):
        for p in self.programs:
            self.assertNotRegex(p.source, r'@\w+@', p.name)
            self.assertIn('main :: ()', p.source, p.name)
            self.assertIn('print(', p.source, p.name)
            self.assertTrue(p.tags[0].startswith('axis:'), p.name)
            self.assertEqual(p.source.count('{'), p.source.count('}'), p.name)
            self.assertEqual(p.source.count('('), p.source.count(')'), p.name)

    def test_every_kind_and_context_is_used(self):
        used = {t for p in self.programs for t in p.tags}
        for kind in grid.KINDS:
            self.assertIn(f'kind:{kind.name}', used)
        for context in grid.CONTEXTS:
            self.assertIn(f'context:{context.name}', used)

    def test_valid_and_near_miss_variants(self):
        by_name = {p.name: p for p in self.programs}
        valid = by_name['grid_struct_positional__typed_declaration__valid']
        miss = by_name['grid_struct_positional__typed_declaration__near_miss']
        self.assertIn('res: V = .{3, 4};', valid.source)
        self.assertIn('res: V = .{1, 2, 3};', miss.source)
        self.assertIn('variant:near_miss', miss.tags)
        self.assertNotIn('grid_uninitialized__typed_declaration__near_miss', by_name)

    def test_uninitialized_only_where_allowed(self):
        contexts = {t for p in self.programs if 'kind:uninitialized' in p.tags for t in p.tags if t.startswith('context:')}
        self.assertEqual(contexts, {'context:typed_declaration', 'context:field_default'})

    def test_operator_contexts_expand_over_operators(self):
        names = {p.name for p in self.programs}
        for op in ('plus', 'times', 'equal'):
            self.assertIn(f'grid_struct_positional__operator_left_{op}__valid', names)
        self.assertIn('grid_struct_named__operator_scalar_right_times__valid', names)

    def test_axes_can_be_chosen(self):
        self.assertTrue(all('axis:using' in p.tags for p in grid.grid(['using'])))
        with self.assertRaises(ValueError):
            grid.grid(['nope'])

    def test_generation_is_deterministic(self):
        self.assertEqual([p.source for p in grid.grid()], [p.source for p in self.programs])

    def test_size(self):
        self.assertGreater(len(self.programs), 500)


SAMPLE = '''#import "Basic";
V :: struct { a: s32; b: s32; }
make :: () -> V {
    return V.{1, 2};
}
main :: () {
    v: V = V.{3, 4};
    n: s32 = 5;
    k := n + 2;
    n += 1;
    print("%\\n", k * 3);
}
'''


class RewriteTests(unittest.TestCase):
    def rewrites(self, name):
        return rewrite.rewrite_text(SAMPLE, 'sample.jai', names=[name])

    def test_erase_literal_type(self):
        sources = [p.source for p in self.rewrites('erase_literal_type')]
        self.assertTrue(any('v: V = .{3, 4};' in s for s in sources))
        self.assertTrue(any('return .{1, 2};' in s for s in sources))

    def test_swap_commutative(self):
        self.assertTrue(any('k := 2 + n;' in p.source for p in self.rewrites('swap_commutative')))

    def test_typed_to_cast_and_infer(self):
        self.assertTrue(any('n := cast(s32) 5;' in p.source for p in self.rewrites('typed_to_cast')))
        self.assertTrue(any('k: type_of(n + 2) = n + 2;' in p.source for p in self.rewrites('infer_to_type_of')))

    def test_compound_assignment_expands(self):
        self.assertTrue(any('n = n + (1);' in p.source for p in self.rewrites('expand_compound_assignment')))

    def test_inline_single_use_local_is_tagged_noisy(self):
        inlined = self.rewrites('inline_single_use_local')
        self.assertTrue(inlined)
        self.assertTrue(all('noisy:evaluation_order' in p.tags for p in inlined))
        self.assertTrue(any('print("%\\n", (n + 2) * 3);' in p.source and 'k :=' not in p.source for p in inlined))

    def test_programs_carry_the_rewrite_and_source_tags(self):
        for p in rewrite.rewrite_text(SAMPLE, 'sample.jai'):
            self.assertIn('axis:rewrite', p.tags)
            self.assertTrue(any(t.startswith('rewrite:') for t in p.tags))
            self.assertIn('source:sample.jai', p.tags)
            self.assertNotEqual(p.source, SAMPLE)

    def test_a_file_without_main_is_skipped(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'lib.jai'
            path.write_text('x :: 1;\n')
            self.assertEqual(rewrite.rewrite_files([path]), [])

    def test_site_limit(self):
        text = '\n'.join(f'    x{i}: s32 = {i};' for i in range(10)).join(['main :: () {\n', '\n}\n'])
        self.assertEqual(len(rewrite.rewrite_text(text, 't', names=['typed_to_cast'], max_sites=2)), 2)


class EndToEndTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.jaic = find_jaic()
        if not cls.jaic:
            raise unittest.SkipTest('no jaic binary (set JAIC or build target/release/jaic)')

    def run_compare(self, *argv):
        with tempfile.TemporaryDirectory() as out:
            code = compare.main([*argv, '--out', out, '--jobs', '2', '--timeout', '120', 'grid:using'])
            return code, json.loads((Path(out) / 'results.json').read_text()), (Path(out) / 'report.md').read_text()

    def test_a_compiler_agrees_with_itself(self):
        code, data, report = self.run_compare('--a', self.jaic, '--b', self.jaic)
        self.assertEqual(code, 0)
        summary = data['summary']
        self.assertEqual(summary['total'], len(grid.grid(['using'])))
        for name in ('a_only', 'b_only', 'output_differs', 'failure'):
            self.assertEqual(summary[name], 0)
        self.assertGreater(summary['agree'], 0)
        self.assertIn('No disagreements', report)

    def test_a_compiler_that_rejects_everything_shows_as_a_only(self):
        with tempfile.TemporaryDirectory() as d:
            fake = Path(d) / 'fake-compiler'
            fake.write_text('#!/bin/sh\necho "prog.jai:1:1: error: nothing compiles here" >&2\nexit 1\n')
            fake.chmod(fake.stat().st_mode | stat.S_IXUSR)
            code, data, report = self.run_compare('--a', self.jaic, '--b', str(fake))
        self.assertEqual(code, 1)
        summary = data['summary']
        self.assertGreater(summary['a_only'], 0)
        self.assertEqual(summary['b_only'], 0)
        self.assertEqual(data['error_clusters'][0]['message'], 'error: nothing compiles here')
        self.assertIn('a_only', report)
        self.assertEqual(data['b']['dialect'], 'plain')

    def test_a_crashing_compiler_does_not_stop_the_run(self):
        with tempfile.TemporaryDirectory() as d:
            fake = Path(d) / 'crashing'
            fake.write_text('#!/bin/sh\nkill -SEGV $$\n')
            fake.chmod(fake.stat().st_mode | stat.S_IXUSR)
            code, data, _ = self.run_compare('--a', self.jaic, '--b', str(fake))
        self.assertEqual(code, 1)
        self.assertEqual(data['summary']['failure'], data['summary']['total'])


if __name__ == '__main__':
    unittest.main()
