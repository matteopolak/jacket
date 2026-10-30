import argparse
import os
import sys
import tempfile
import time
import unittest
from pathlib import Path

from bench.cmd_compile import Attempt, Job, JobRunner, first_difference, normalized, plan_jobs, stage_stats
from bench.dialects import (Compiler, JaicDialect, Mode, PlainDialect, Request, detect_dialect, executable_file,
                            parse_compiler)
from bench.measure import Run, run_measured
from bench.stages import Bucket
from bench.workloads import Category, Plan, RepoDirectory, Workload, all_workloads


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

    def test_linker_import_library_line_is_not_compiler_output(self):
        text = 'ok\n   Creating library C:\\x\\a.lib and object C:\\x\\a.exp\n'
        self.assertEqual(normalized(text), ['ok'])
        self.assertEqual(normalized('Creating library of things\n'), ['Creating library of things'])

    def test_helpers(self):
        self.assertEqual(normalized('a  \n\n b\n'), ['a', ' b'])
        self.assertEqual(first_difference(['a'], ['a', 'b']), "1 lines vs 2 lines, the first extra one 'b'")


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
        self.assertEqual(two.driver(job_, two.compilers[0], own).command, [str(Path('/x/a')), 'first.jai', '-timings'])
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
                         [str(compiler), 'main.jai', '-check', '-timings'])
        self.assertEqual(dialect.invocation(compiler, self.request(Mode.INTERP)).command[2], '-interp')
        build = dialect.invocation(compiler, self.request(Mode.OPTIMIZED))
        self.assertEqual(build.command, [str(compiler), 'main.jai', '-o', '-timings', '-exe', str(Path('/s/program'))])
        self.assertEqual(build.executable, executable_file(Path('/s')))

    def test_timings_come_before_the_metaprogram_arguments(self):
        command = JaicDialect().invocation(Path('jaic'), self.request(Mode.BUILD, ['release'], redirect=False)).command
        self.assertEqual(command[command.index('-timings'):], ['-timings', '-', 'release'])
        self.assertNotIn('-exe', command)

    def test_a_plain_binary_gets_only_the_portable_command_line(self):
        with tempfile.TemporaryDirectory() as scratch:
            build = Request(Path('/w/main.jai'), Path('/w'), Mode.OPTIMIZED, (), Path(scratch), import_dirs=('/m',))
            command = PlainDialect().invocation(Path('/old/jaic'), build).command
            self.assertNotIn('-timings', command)
            self.assertEqual(command[1:], ['main.jai', '-no_color', '-import_dir', '/m', '-o',
                                           '-exe', 'program', '-output_path', scratch])
            check = PlainDialect().invocation(Path('/old/jaic'), Request(Path('/w/main.jai'), Path('/w'), Mode.INTERP, (), Path(scratch)))
            self.assertEqual(check.command, [str(Path('/old/jaic')), 'wrapper.jai', '-no_color'])
            wrapper = (Path(scratch) / 'wrapper.jai').read_text(encoding='utf-8')
            self.assertIn('do_output=false', wrapper)
            self.assertIn('#run main();', wrapper)
            self.assertNotIn('-check', check.command)

    def test_dialect_comes_from_probing_the_binary_not_its_name(self):
        with tempfile.TemporaryDirectory() as folder:
            def fake(name, help_text):
                if sys.platform == 'win32':  # a batch file runs as a program; no shell scripts
                    path = Path(folder) / (name + '.bat')
                    path.write_text(f'@echo {help_text}\r\n', encoding='utf-8')
                    return path
                path = Path(folder) / name
                path.write_text(f'#!/bin/sh\necho "{help_text}"\n', encoding='utf-8')
                path.chmod(0o755)
                return path
            self.assertIsInstance(detect_dialect(fake('jaic', '  -timings  print phases')), JaicDialect)
            self.assertIsInstance(detect_dialect(fake('jaic-old', '  -check  only check')), PlainDialect)
            self.assertIsInstance(detect_dialect(fake('anything', '  -timings  print phases')), JaicDialect)
            self.assertIsInstance(detect_dialect(Path(folder) / 'missing'), PlainDialect)
            self.assertEqual(parse_compiler(f'a={fake("x", "-timings")}').dialect.name, 'jaic')
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
        self.assertIsNone(PlainDialect().instruction_count(run(stderr='interpreter profile: 5 instructions')))

    def test_a_plain_binary_prints_no_detail(self):
        self.assertEqual(PlainDialect().detail(run(wall=0.7, stdout='hello\n')), {})

    def test_every_compiler_builds_the_phase_metaprogram_the_same_way(self):
        with tempfile.TemporaryDirectory() as scratch:
            request = Request(Path('/w/main.jai'), Path('/w'), Mode.INTERP, (), Path(scratch))
            jaic = JaicDialect().phase_invocation(Path('/bin/jaic'), request)
            ref = PlainDialect().phase_invocation(Path('/old/jaic'), request)
            source = (Path(scratch) / 'phases' / 'phase_timing.jai').read_text(encoding='utf-8')
        self.assertEqual(jaic.command, [str(Path('/bin/jaic')), 'phase_timing.jai'])
        self.assertEqual(ref.command, [str(Path('/old/jaic')), 'phase_timing.jai', '-no_color'])
        self.assertEqual(jaic.cwd, ref.cwd)
        entry = str(Path('/w/main.jai').resolve()).replace('\\', '\\\\')  # escaped inside a Jai string literal
        self.assertIn(f'ENTRY :: "{entry}";', source)
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


PYTHON = sys.executable


class MeasureTests(unittest.TestCase):
    def test_a_finished_process_reports_status_output_and_resources(self):
        code = 'import sys; print("out"); print("err", file=sys.stderr); sys.exit(3)'
        result = run_measured([PYTHON, '-c', code])
        self.assertEqual((result.returncode, result.stdout.strip(), result.stderr.strip()), (3, 'out', 'err'))
        self.assertFalse(result.ok)
        self.assertGreater(result.rss_mib, 0)
        self.assertGreaterEqual(result.cpu, 0)

    def test_stdin_and_timeout(self):
        echo = run_measured([PYTHON, '-c', 'import sys; sys.stdout.write(sys.stdin.read())'], stdin_bytes=b'hi')
        self.assertEqual(echo.stdout, 'hi')
        slow = run_measured([PYTHON, '-c', 'import time; time.sleep(5)'], timeout=0.2)
        self.assertTrue(slow.timed_out)
        self.assertIn('timed out', slow.failure())

    # os.kill(pid, 0) probes on POSIX but would terminate the process on Windows; test_windows covers the job tree kill.
    @unittest.skipIf(sys.platform == 'win32', 'probes the child with os.kill(pid, 0); test_windows covers job objects')
    def test_a_timeout_kills_the_children_too(self):
        with tempfile.TemporaryDirectory() as scratch:
            pidfile = Path(scratch) / 'child.pid'
            result = run_measured(['/bin/sh', '-c', f'sleep 30 & echo $! > {pidfile}; wait'], timeout=0.5)
            self.assertTrue(result.timed_out)
            child = int(pidfile.read_text())
        time.sleep(0.1)
        with self.assertRaises(ProcessLookupError):
            os.kill(child, 0)

    @unittest.skipIf(sys.platform == 'win32', 'probes the child with os.kill(pid, 0); test_windows covers job objects')
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
