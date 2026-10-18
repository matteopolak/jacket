import argparse
import os
import tempfile
import time
import unittest
from pathlib import Path

from cmd_compile import Attempt, Job, JobRunner, first_difference, normalized, plan_jobs, stage_stats
from dialects import Compiler, JaicDialect, Mode, ReferenceDialect, Request, parse_compiler
from measure import Run, run_measured
from stages import Bucket
from workloads import Category, Plan, RepoDirectory, Workload, all_workloads


def run(wall=1.0, returncode=0, stdout='', stderr='', timed_out=False):
    return Run(wall=wall, cpu=wall, rss_mib=10.0, returncode=returncode, stdout=stdout, stderr=stderr, timed_out=timed_out)


def runner(*names):
    args = argparse.Namespace(timeout=10, warmup=1, repeat=2, no_sandbox=True)
    compilers = [Compiler(name, Path('/x') / name, JaicDialect()) for name in names]
    return JobRunner(compilers, Path('/tmp'), args)


def job(compare_output=True):
    workload = Workload('w', Category.STARTUP, RepoDirectory(Path('.')), {Mode.BUILD: Plan('w.jai')},
                        compare_output=compare_output)
    return Job(workload, Mode.BUILD)


class GateTests(unittest.TestCase):
    def test_agreeing_compilers_pass(self):
        attempts = {'a': Attempt(run(), ['1', '2']), 'b': Attempt(run(), ['1', '2'])}
        self.assertEqual(runner('a', 'b').gate(job(), attempts), '')

    def test_a_compiler_that_fails_to_build_names_itself_and_why(self):
        attempts = {'a': Attempt(run(), ['1']), 'ref': Attempt(run(returncode=1), [], 'exit 1: unknown syntax')}
        self.assertEqual(runner('a', 'ref').gate(job(), attempts), 'ref: exit 1: unknown syntax')

    def test_different_output_is_reported_with_the_first_difference(self):
        attempts = {'a': Attempt(run(), ['x', 'one']), 'b': Attempt(run(), ['x', 'two'])}
        reason = runner('a', 'b').gate(job(), attempts)
        self.assertIn('output differs (a vs b)', reason)
        self.assertIn("line 2: 'one' vs 'two'", reason)

    def test_output_is_not_compared_when_the_workload_does_not_ask(self):
        attempts = {'a': Attempt(run(), ['x']), 'b': Attempt(run(), ['y'])}
        self.assertEqual(runner('a', 'b').gate(job(compare_output=False), attempts), '')

    def test_helpers(self):
        self.assertEqual(normalized('a  \n\n b\n'), ['a', ' b'])
        self.assertEqual(first_difference(['a'], ['a', 'b']), '1 lines vs 2 lines')


class DriverTests(unittest.TestCase):
    def request(self, redirect, args=()):
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(lambda: __import__('shutil').rmtree(scratch, ignore_errors=True))
        return Request(Path('/w/first.jai'), Path('/w'), Mode.BUILD, tuple(args), scratch, redirect)

    def test_a_project_that_chooses_its_own_output_is_not_wrapped_in_the_phase_metaprogram(self):
        # The wrapper runs in another folder and hands its default options to the project's nested workspace,
        # so a project like focus (custom link command inherited) or forbear (relative vendor paths) breaks.
        two = runner('a', 'b')
        job_ = job()
        own = self.request(redirect=False)
        self.assertEqual(two.driver(job_, two.compilers[0], own).command, ['/x/a', 'first.jai', '-timings'])
        wrapped = two.driver(job_, two.compilers[0], self.request(redirect=True))
        self.assertEqual(wrapped.command[1], 'phase_timing.jai')

    def test_stage_timing_through_the_wrapper_is_skipped_for_such_a_project(self):
        two = runner('a', 'b')
        two.args.phase_repeat = 1
        two.compile = lambda *a: self.fail('the phase-timing wrapper must not run')
        two.add_stages(job(), {'a': self.request(False), 'b': self.request(False)}, {})


class StageStatTests(unittest.TestCase):
    def test_median_per_bucket_over_rounds(self):
        stats = stage_stats([{Bucket.TOTAL: 1.0, Bucket.LINK: 0.1}, {Bucket.TOTAL: 3.0, Bucket.LINK: 0.3},
                             {Bucket.TOTAL: 2.0, Bucket.LINK: 0.2}])
        self.assertEqual(stats['total'].median, 2.0)
        self.assertAlmostEqual(stats['link'].median, 0.2)
        self.assertNotIn('codegen', stats)


class DialectTests(unittest.TestCase):
    def request(self, mode, args=(), redirect=True):
        return Request(Path('/w/main.jai'), Path('/w'), mode, tuple(args), Path('/s'), redirect)

    def test_jaic_modes(self):
        dialect = JaicDialect()
        compiler = Path('/bin/jaic')
        self.assertEqual(dialect.invocation(compiler, self.request(Mode.CHECK)).command,
                         ['/bin/jaic', 'main.jai', '-check', '-timings'])
        self.assertEqual(dialect.invocation(compiler, self.request(Mode.INTERP)).command[2], '-interp')
        build = dialect.invocation(compiler, self.request(Mode.OPTIMIZED))
        self.assertEqual(build.command, ['/bin/jaic', 'main.jai', '-o', '-timings', '-exe', '/s/program'])
        self.assertEqual(build.executable, Path('/s/program'))

    def test_timings_come_before_the_metaprogram_arguments(self):
        command = JaicDialect().invocation(Path('jaic'), self.request(Mode.BUILD, ['release'], redirect=False)).command
        self.assertEqual(command[command.index('-timings'):], ['-timings', '-', 'release'])
        self.assertNotIn('-exe', command)

    def test_reference_check_goes_through_a_wrapper_that_writes_nothing(self):
        with tempfile.TemporaryDirectory() as scratch:
            request = Request(Path('/w/main.jai'), Path('/w'), Mode.INTERP, (), Path(scratch))
            invocation = ReferenceDialect().invocation(Path('/ref/jai'), request)
            wrapper = (Path(scratch) / 'wrapper.jai').read_text()
        self.assertEqual(invocation.command[1], 'wrapper.jai')
        self.assertEqual(invocation.cwd, Path(scratch))
        self.assertIn('do_output=false', wrapper)
        self.assertIn('#load "/w/main.jai"', wrapper)
        self.assertIn('#run main();', wrapper)

    def test_reference_build_names_its_output(self):
        invocation = ReferenceDialect().invocation(Path('/ref/jai'), self.request(Mode.OPTIMIZED))
        self.assertEqual(invocation.command[:3], ['/ref/jai', 'main.jai', '-no_color'])
        self.assertIn('-o', invocation.command)
        self.assertEqual(invocation.executable, Path('/s/program'))

    def test_dialect_comes_from_the_binary_name(self):
        self.assertIsInstance(parse_compiler('a=/x/target/release/jaic').dialect, JaicDialect)
        self.assertIsInstance(parse_compiler('ref=/x/bin/other-compiler').dialect, ReferenceDialect)
        with self.assertRaises(ValueError):
            parse_compiler('no-path')

    def test_jaic_detail_total_is_the_wall_time(self):
        self.assertEqual(JaicDialect().detail(run(wall=0.7, stderr='no timings here')), {Bucket.TOTAL: 0.7})
        stages = JaicDialect().detail(run(wall=0.7, stderr='jaic-timing: front end 0.4 1\njaic-timing: total 0.5 1\n'))
        self.assertEqual(stages[Bucket.TOTAL], 0.7)  # the process, which includes start-up the compiler does not time
        self.assertAlmostEqual(stages[Bucket.OTHER], 0.3)

    def test_jaic_instruction_count_comes_from_the_profile_line(self):
        dialect = JaicDialect()
        self.assertEqual(dialect.instruction_count(run(stderr='x\ninterpreter profile: 119840746 instructions\n')), 119840746)
        self.assertIsNone(dialect.instruction_count(run(stderr='nothing')))
        self.assertIsNone(ReferenceDialect().instruction_count(run(stderr='interpreter profile: 5 instructions')))

    def test_the_reference_prints_no_detail(self):
        self.assertEqual(ReferenceDialect().detail(run(wall=0.7, stdout='hello\n')), {})

    def test_every_compiler_builds_the_phase_metaprogram_the_same_way(self):
        with tempfile.TemporaryDirectory() as scratch:
            request = Request(Path('/w/main.jai'), Path('/w'), Mode.INTERP, (), Path(scratch))
            jaic = JaicDialect().phase_invocation(Path('/bin/jaic'), request)
            ref = ReferenceDialect().phase_invocation(Path('/ref/jai'), request)
            source = (Path(scratch) / 'phases' / 'phase_timing.jai').read_text()
        self.assertEqual(jaic.command, ['/bin/jaic', 'phase_timing.jai'])
        self.assertEqual(ref.command, ['/ref/jai', 'phase_timing.jai', '-no_color'])
        self.assertEqual(jaic.cwd, ref.cwd)
        self.assertIn('ENTRY :: "/w/main.jai";', source)
        self.assertIn('WRITES_EXECUTABLE :: false;', source)
        self.assertIn('RUN_MAIN :: true;', source)
        self.assertEqual(source.count('// settings:'), 1)


class WorkloadTests(unittest.TestCase):
    def test_names_are_unique_and_modes_valid(self):
        workloads = all_workloads()
        names = [w.name for w in workloads]
        self.assertEqual(len(names), len(set(names)))
        for w in workloads:
            self.assertTrue(w.plans, w.name)
            self.assertNotIn('/', w.name)

    def test_every_category_has_a_workload(self):
        self.assertEqual({w.category for w in all_workloads()}, set(Category))

    def test_selection(self):
        args = argparse.Namespace(modes='check', only='hello', categories='')
        self.assertEqual([j.key for j in plan_jobs(args)], ['hello/check'])
        args = argparse.Namespace(modes='interp', only='', categories='interpreter')
        self.assertTrue(all(j.mode is Mode.INTERP for j in plan_jobs(args)))

    def test_runtime_workloads_are_buildable(self):
        for w in all_workloads():
            if w.runtime:
                self.assertTrue(any(m.writes_executable for m in w.modes), w.name)


class MeasureTests(unittest.TestCase):
    def test_a_finished_process_reports_status_output_and_resources(self):
        result = run_measured(['/bin/sh', '-c', 'echo out; echo err >&2; exit 3'])
        self.assertEqual((result.returncode, result.stdout.strip(), result.stderr.strip()), (3, 'out', 'err'))
        self.assertFalse(result.ok)
        self.assertGreater(result.rss_mib, 0)
        self.assertGreaterEqual(result.cpu, 0)

    def test_stdin_and_timeout(self):
        self.assertEqual(run_measured(['/bin/cat'], stdin_bytes=b'hi').stdout, 'hi')
        slow = run_measured(['/bin/sleep', '5'], timeout=0.2)
        self.assertTrue(slow.timed_out)
        self.assertIn('timed out', slow.failure())

    def test_a_timeout_kills_the_children_too(self):
        with tempfile.TemporaryDirectory() as scratch:
            pidfile = Path(scratch) / 'child.pid'
            result = run_measured(['/bin/sh', '-c', f'sleep 30 & echo $! > {pidfile}; wait'], timeout=0.5)
            self.assertTrue(result.timed_out)
            child = int(pidfile.read_text())
        time.sleep(0.1)
        with self.assertRaises(ProcessLookupError):
            os.kill(child, 0)

    def test_children_left_behind_by_a_finished_command_are_killed(self):
        with tempfile.TemporaryDirectory() as scratch:
            pidfile = Path(scratch) / 'child.pid'
            run_measured(['/bin/sh', '-c', f'sleep 30 > /dev/null 2>&1 & echo $! > {pidfile}'])
            child = int(pidfile.read_text())
        time.sleep(0.1)
        with self.assertRaises(ProcessLookupError):
            os.kill(child, 0)

    def test_failure_text_is_the_last_lines(self):
        self.assertEqual(run(returncode=1, stderr='a\nb\nc\n').failure(), 'exit 1: b | c')


if __name__ == '__main__':
    unittest.main()
