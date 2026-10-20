import argparse
import csv
import json
import tempfile
import unittest
from collections import Counter
from pathlib import Path

import cmd_compare
import pairing
import provenance
import quiet
from cmd_compare import Observed, compare_outputs, plan_jobs, summarize_samples


class InterleaveTests(unittest.TestCase):
    def test_every_round_has_every_binary_once(self):
        for order in pairing.interleave(['a', 'b'], 9, seed=3):
            self.assertEqual(sorted(order), ['a', 'b'])

    def test_each_binary_goes_first_equally_often(self):
        firsts = Counter(order[0] for order in pairing.interleave(['a', 'b'], 20, seed=7))
        self.assertEqual(firsts, {'a': 10, 'b': 10})

    def test_odd_round_counts_differ_by_at_most_one(self):
        firsts = Counter(order[0] for order in pairing.interleave(['a', 'b'], 7, seed=1))
        self.assertLessEqual(abs(firsts['a'] - firsts['b']), 1)

    def test_same_seed_same_order_and_other_seed_may_differ(self):
        self.assertEqual(pairing.interleave(['a', 'b'], 12, 5), pairing.interleave(['a', 'b'], 12, 5))
        self.assertNotEqual(pairing.interleave(['a', 'b'], 40, 5), pairing.interleave(['a', 'b'], 40, 6))

    def test_alternate_is_a_b_every_round(self):
        self.assertEqual(pairing.interleave(['a', 'b'], 3, 0, shuffle=False), [['a', 'b']] * 3)

    def test_block_seeds_depend_on_the_block(self):
        self.assertNotEqual(cmd_compare.block_seed(1, 'x/check/warm'), cmd_compare.block_seed(1, 'x/check/cold'))
        self.assertEqual(cmd_compare.block_seed(1, 'x'), cmd_compare.block_seed(1, 'x'))


class RatioTests(unittest.TestCase):
    def test_a_consistent_slowdown_is_significant(self):
        base = [1.00, 1.02, 0.99, 1.01, 1.00, 1.03]
        result = pairing.ratio_summary(base, [v * 1.3 for v in base], seed=1)
        self.assertEqual(result['verdict'], 'candidate slower')
        self.assertAlmostEqual(result['ratio'], 1.3)
        self.assertGreater(result['low'], 1.0)

    def test_a_consistent_speedup_is_significant(self):
        base = [1.0, 1.1, 0.9, 1.05, 0.95, 1.0]
        self.assertEqual(pairing.ratio_summary(base, [v * 0.5 for v in base], 1)['verdict'], 'candidate faster')

    def test_noise_around_one_is_not_a_difference(self):
        base = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
        candidate = [0.9, 1.1, 0.95, 1.05, 1.1, 0.9, 1.0, 1.02]
        self.assertEqual(pairing.ratio_summary(base, candidate, 1)['verdict'], 'no significant difference')

    def test_too_few_rounds_gives_no_verdict(self):
        result = pairing.ratio_summary([1.0, 1.0], [2.0, 2.0], 1)
        self.assertIn('too few rounds', result['verdict'])
        self.assertEqual(result['ratio'], 2.0)

    def test_the_interval_is_reproducible(self):
        base, candidate = [1, 2, 3, 4, 5, 6], [2, 2, 3, 5, 5, 7]
        self.assertEqual(pairing.ratio_summary(base, candidate, 9), pairing.ratio_summary(base, candidate, 9))


class OutputEqualityTests(unittest.TestCase):
    def seen(self, **changes):
        base = dict(compile_exit=0, compile_stdout=['ok'], program_exit=0, program_stdout=['sum 42'])
        return Observed(**{**base, **changes})

    def test_identical_behaviour_passes(self):
        self.assertEqual(compare_outputs(self.seen(), self.seen(), True), '')

    def test_program_stdout_difference_is_reported(self):
        reason = compare_outputs(self.seen(), self.seen(program_stdout=['sum 43']), True)
        self.assertIn('program output differs', reason)
        self.assertIn("'sum 42' vs 'sum 43'", reason)

    def test_program_exit_code_difference_is_reported(self):
        self.assertIn('program exit codes differ (0 vs 3)', compare_outputs(self.seen(), self.seen(program_exit=3), True))

    def test_a_build_error_names_the_binary(self):
        reason = compare_outputs(self.seen(), self.seen(error='exit 1: bad flag'), True)
        self.assertEqual(reason, 'candidate: exit 1: bad flag')
        self.assertTrue(compare_outputs(self.seen(error='x'), self.seen(), True).startswith('baseline:'))

    def test_compiler_output_is_compared_only_when_asked(self):
        other = self.seen(compile_stdout=['different'])
        self.assertIn('compiler output differs', compare_outputs(self.seen(), other, True))
        self.assertEqual(compare_outputs(self.seen(), other, False), '')


MACOS_THERM = 'Note: No thermal warning level has been recorded\nCPU_Scheduler_Limit = 100\nCPU_Available_CPUs = 10\nCPU_Speed_Limit = 100\n'
MACOS_THROTTLED = 'CPU_Scheduler_Limit = 100\nCPU_Speed_Limit = 70\n'
VM_STAT = ('Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free:      1000.\n'
           'Pages inactive:  2000.\nPages speculative: 500.\nPages active: 99999.\n')
PS = '  1 /sbin/launchd\n  42 /usr/bin/python3\n 100 /Users/me/.cargo/bin/cargo\n 101 /x/rustc\n 102 /System/mds\n 103 /usr/libexec/backupd\n 104 mdworker_shared\n'


def snap(load=0.5, throttled=False, power='ac', free=8000.0, noisy=()):
    return {'time': 0, 'cpus': 10, 'load': [load, load, load], 'thermal': {'throttled': throttled},
            'power': power, 'free_mib': free, 'noisy': list(noisy)}


class QuietTests(unittest.TestCase):
    def test_parsers(self):
        self.assertFalse(quiet.parse_thermal_macos(MACOS_THERM)['throttled'])
        self.assertTrue(quiet.parse_thermal_macos(MACOS_THROTTLED)['throttled'])
        self.assertEqual(quiet.parse_power_macos("Now drawing from 'AC Power'\n"), 'ac')
        self.assertEqual(quiet.parse_power_macos("Now drawing from 'Battery Power'\n"), 'battery')
        self.assertEqual(quiet.parse_power_macos(''), 'unknown')
        self.assertEqual(quiet.parse_vm_stat(VM_STAT), 54.7)
        self.assertEqual(quiet.parse_meminfo('MemAvailable:    2048000 kB\n'), 2000.0)

    def test_noisy_processes_match_by_name_and_ignore_own_pids(self):
        self.assertEqual(quiet.noisy_processes(PS), ['backupd', 'cargo', 'mds', 'mdworker_shared', 'rustc'])
        self.assertEqual(quiet.noisy_processes(PS, {100, 101}), ['backupd', 'mds', 'mdworker_shared'])
        self.assertEqual(quiet.noisy_processes('  1 /usr/bin/python3\n'), [])

    def test_a_quiet_machine_passes(self):
        self.assertEqual(quiet.start_problems(snap(), 2.5), ([], []))
        self.assertEqual(quiet.block_flags(snap(), snap(), 2.5), [])

    def test_high_load_and_throttling_refuse_the_start(self):
        errors, _ = quiet.start_problems(snap(load=6.0, throttled=True), 2.5)
        self.assertEqual(len(errors), 2)

    def test_noise_and_battery_only_warn_at_the_start(self):
        errors, warnings = quiet.start_problems(snap(power='battery', noisy=['cargo']), 2.5)
        self.assertEqual(errors, [])
        self.assertEqual(len(warnings), 2)

    def test_a_block_is_flagged_for_every_state_change(self):
        flags = quiet.block_flags(snap(), snap(load=3.9, throttled=True, power='battery', free=4000.0, noisy=['rustc']), 2.5)
        text = '; '.join(flags)
        for expected in ('load 3.9 above', 'load rose', 'thermally limited', 'thermal state changed',
                         'power source changed', 'free memory fell', 'noisy processes: rustc'):
            self.assertIn(expected, text)

    def test_a_small_memory_dip_is_not_a_change(self):
        self.assertEqual(quiet.block_flags(snap(free=8000.0), snap(free=7800.0), 2.5), [])

    def test_default_load_limit_scales_with_cores(self):
        self.assertEqual(quiet.default_max_load(16), 4.0)
        self.assertEqual(quiet.default_max_load(2), 1.0)

    def test_snapshot_has_every_field(self):
        shot = quiet.snapshot()
        for field in ('load', 'thermal', 'power', 'free_mib', 'noisy', 'cpus'):
            self.assertIn(field, shot)
        self.assertEqual(len(shot['load']), 3)


class ProvenanceTests(unittest.TestCase):
    def folder(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return Path(directory.name)

    def test_tree_hash_depends_on_names_and_contents(self):
        a, b = self.folder(), self.folder()
        for root in (a, b):
            (root / 'sub').mkdir()
            (root / 'main.jai').write_text('x')
            (root / 'sub' / 'part.jai').write_text('y')
        self.assertEqual(provenance.tree_sha256(a), provenance.tree_sha256(b))
        (b / 'sub' / 'part.jai').write_text('z')
        self.assertNotEqual(provenance.tree_sha256(a), provenance.tree_sha256(b))
        (b / 'sub' / 'part.jai').write_text('y')
        (b / 'sub' / 'part.jai').rename(b / 'sub' / 'other.jai')
        self.assertNotEqual(provenance.tree_sha256(a), provenance.tree_sha256(b))

    def test_binary_facts_hash_and_version(self):
        root = self.folder()
        binary = root / 'tool'
        binary.write_text('#!/bin/sh\necho "tool 1.2.3"\n')
        binary.chmod(0o755)
        facts = provenance.binary_facts(binary)
        self.assertEqual(facts['version'], 'tool 1.2.3')
        self.assertEqual(facts['sha256'], provenance.sha256_file(binary))
        self.assertEqual(len(facts['sha256']), 64)
        self.assertEqual(provenance.binary_facts(root / 'missing'), {})

    def test_machine_specs_name_the_hardware_and_os(self):
        specs = provenance.machine_specs()
        for field in ('cpu', 'cpus', 'memory_gib', 'system', 'os_version'):
            self.assertIn(field, specs)

    def test_raw_csv_keeps_every_sample_in_order(self):
        path = self.folder() / 'raw.csv'
        samples = [{'workload': 'w', 'mode': 'check', 'phase': 'warm', 'round': r, 'position': p, 'compiler': c,
                    'wall_s': 1.5, 'cpu_s': 2.0, 'rss_mib': 10.0, 'exit': 0, 'stages': {'link': 0.25}}
                   for r, order in enumerate([['baseline', 'candidate'], ['candidate', 'baseline']])
                   for p, c in enumerate(order)]
        provenance.write_raw_csv(path, samples)
        with path.open() as handle:
            rows = list(csv.DictReader(handle))
        self.assertEqual([row['compiler'] for row in rows],
                         ['baseline', 'candidate', 'candidate', 'baseline'])
        self.assertEqual(rows[0]['stages'], 'link=0.250000')
        self.assertEqual(list(rows[0]), provenance.RAW_COLUMNS)


def samples_for(ratio, rounds=6, stages=False):
    out = []
    for r in range(rounds):
        for name, factor in (('baseline', 1.0), ('candidate', ratio)):
            wall = (1.0 + 0.01 * r) * factor
            out.append({'workload': 'w', 'mode': 'build', 'phase': 'warm', 'round': r, 'position': 0,
                        'compiler': name, 'wall_s': wall, 'cpu_s': wall * 2, 'rss_mib': 100.0 * factor,
                        'exit': 0, 'stages': {'link': 0.2 * factor} if stages else {}})
    return out


class SummaryTests(unittest.TestCase):
    def test_paired_ratio_and_other_metrics(self):
        summary = summarize_samples(samples_for(1.25, stages=True), seed=1)
        self.assertAlmostEqual(summary['wall_ratio']['ratio'], 1.25)
        self.assertEqual(summary['wall_ratio']['verdict'], 'candidate slower')
        self.assertAlmostEqual(summary['cpu_ratio'], 1.25)
        self.assertAlmostEqual(summary['rss_ratio'], 1.25)
        self.assertAlmostEqual(summary['stages']['link']['ratio'], 1.25)
        self.assertEqual(summary['baseline']['wall']['n'], 6)

    def test_no_stage_table_without_stage_data(self):
        self.assertNotIn('stages', summarize_samples(samples_for(1.0), seed=1))

    def test_render_shows_the_ratio_verdict_flags_and_skips(self):
        summary = summarize_samples(samples_for(0.5, stages=True), seed=1)
        block = {'workload': 'w', 'mode': 'build', 'phase': 'warm', 'status': 'ok', 'reason': '', 'summary': summary,
                 'flags': ['load 9 above 2.5 before']}
        skipped = {'workload': 'p', 'mode': 'check', 'phase': 'both', 'status': 'skipped', 'reason': 'baseline: exit 1', 'flags': []}
        binary = {'path': '/x', 'version': 'jaic 1', 'sha256': 'ab' * 32, 'dialect': 'jaic'}
        text = cmd_compare.render({
            'date': '2026-01-01T00:00:00', 'machine': {'cpu': 'cpu', 'cpus': 4, 'memory_gib': 8, 'system': 'Darwin', 'release': '1'},
            'binaries': {'baseline': binary, 'candidate': binary},
            'settings': {'rounds': 6, 'cold_rounds': 2, 'order': 'shuffle', 'seed': 1}, 'purge': 'not purged',
            'start_warnings': ['noisy processes running: cargo'], 'blocks': [block, skipped]})
        for expected in ('candidate faster', '0.500', 'flagged', 'load 9 above', 'Not timed', 'baseline: exit 1',
                         'noisy processes running: cargo', '## Steps'):
            self.assertIn(expected, text)


class PlanTests(unittest.TestCase):
    def args(self, **changes):
        base = dict(sizes='10000,100000', seed=4, no_corpus=True, modes='check,build,optimized', only='')
        return argparse.Namespace(**{**base, **changes})

    def test_generated_workloads_carry_the_seed_and_all_modes(self):
        jobs = plan_jobs(self.args())
        self.assertEqual([j.key for j in jobs], ['gen-10k/check', 'gen-10k/build', 'gen-10k/optimized',
                                                 'gen-100k/check', 'gen-100k/build', 'gen-100k/optimized'])
        self.assertEqual(jobs[0].workload.source.seed, 4)

    def test_modes_and_only_filter(self):
        jobs = plan_jobs(self.args(modes='check', only='100k'))
        self.assertEqual([j.key for j in jobs], ['gen-100k/check'])

    def test_corpus_workloads_are_added_unless_skipped(self):
        names = {j.workload.name for j in plan_jobs(self.args(no_corpus=False))}
        self.assertIn('jails', names)
        self.assertEqual(json.dumps(sorted(n for n in names if n.startswith('gen-'))), '["gen-100k", "gen-10k"]')


if __name__ == '__main__':
    unittest.main()


class SameCommandLineTests(unittest.TestCase):
    def test_a_mixed_pair_runs_both_binaries_the_portable_way(self):
        from dialects import Compiler, JaicDialect, PlainDialect
        from cmd_compare import same_command_line
        mixed = {'baseline': Compiler('baseline', Path('/a'), PlainDialect()),
                 'candidate': Compiler('candidate', Path('/b'), JaicDialect())}
        self.assertEqual({c.dialect.name for c in same_command_line(mixed).values()}, {'plain'})
        both = {n: Compiler(n, Path('/' + n), JaicDialect()) for n in ('baseline', 'candidate')}
        self.assertEqual({c.dialect.name for c in same_command_line(both).values()}, {'jaic'})
