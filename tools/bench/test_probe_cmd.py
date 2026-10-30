import argparse
import json
import os
import stat
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bench import cmd_probe
from bench.cmd_probe import Probe, Side
from bench.dialects import Compiler, PlainDialect

# A stand-in compiler: reads the wrapper named on its command line and behaves by what the wrapper
# contains, so the test needs no real compiler and no sandbox.
FAKE = '''#!/usr/bin/env python3
import os, sys
wrapper = open(sys.argv[1]).read()
entry = wrapper.split('#load "')[1].split('"')[0]
source = open(entry).read()
if 'BROKEN' in source:
    sys.stderr.write(os.environ.get('FAKE_ERROR', 'error') + '\\n')
    sys.exit(1)
if '#run main();' in wrapper:
    sys.stdout.write(os.environ.get('FAKE_OUT', 'hello') + '\\n')
    sys.exit(int(os.environ.get('FAKE_EXIT', '0')))
'''


class NoSandbox:
    def __init__(self, *args, **kwargs):
        pass

    def wrap(self, command):
        return command

    def close(self):
        pass

    @staticmethod
    def available():
        return ''


def arguments(**overrides) -> argparse.Namespace:
    values = dict(import_dir=[], sandbox_read=[], x64=False, timeout=30)
    values.update(overrides)
    return argparse.Namespace(**values)


class NormalizeTests(unittest.TestCase):
    def test_paths_colours_and_trailing_blanks_are_removed(self):
        names = {Path('/work/scratch'): '<scratch>', Path('/work'): '<w>'}
        text = '\x1b[31m/work/scratch/a.jai:1: x  \x1b[0m\r\n/work/b\n\n'
        self.assertEqual(cmd_probe.normalize(text, names), '<scratch>/a.jai:1: x\n<w>/b')

    def test_the_longer_path_wins_where_one_contains_the_other(self):
        names = {Path('/a'): '<a>', Path('/a/b'): '<b>'}
        self.assertEqual(cmd_probe.normalize('/a/b/c', names), '<b>/c')

    def test_empty_output_stays_empty(self):
        self.assertEqual(cmd_probe.normalize('', {}), '')


class CommandTests(unittest.TestCase):
    def setUp(self):
        self.compiler = Compiler('ref', Path('/x/jai'), PlainDialect())

    def test_the_command_line_is_the_portable_one(self):
        command = cmd_probe.command_for(self.compiler, 'probe.jai', ('/m',), False)
        self.assertEqual(command, ['/x/jai', 'probe.jai', '-no_color', '-import_dir', '/m'])

    def test_x64_runs_the_compiler_under_arch(self):
        command = cmd_probe.command_for(self.compiler, 'probe.jai', (), True)
        self.assertEqual(command[:3], ['arch', '-x86_64', '/x/jai'])

    def test_the_wrapper_runs_main_only_in_the_run_phase(self):
        self.assertNotIn('#run main', cmd_probe.WRAPPER.format(entry='/a.jai', run=''))
        self.assertIn('#run main();', cmd_probe.WRAPPER.format(entry='/a.jai', run=cmd_probe.RUN))


class ComparisonTests(unittest.TestCase):
    def side(self, **overrides):
        values = dict(name='s', compiled=True, exit=0, stdout='a', stderr='')
        values.update(overrides)
        return Side(**values)

    def test_equal_sides_have_no_differences(self):
        self.assertEqual(cmd_probe.compare_sides(self.side(), self.side()), [])

    def test_each_field_is_named(self):
        other = self.side(compiled=False, exit=1, stdout='b', stderr='e')
        self.assertEqual(cmd_probe.compare_sides(self.side(), other), ['compiled', 'exit', 'stdout', 'stderr'])

    def test_a_timeout_is_a_difference(self):
        self.assertEqual(cmd_probe.compare_sides(self.side(), self.side(timed_out=True)), ['timed_out'])

    def test_diff_text_is_cut_to_a_limit(self):
        long = '\n'.join(str(n) for n in range(100))
        text = cmd_probe.diff_text(long, '', limit=5)
        self.assertIn('more lines', text.splitlines()[-1])

    def test_render_summarises_and_hides_agreeing_output(self):
        agree = Probe('a.jai', {'jaic': self.side(), 'reference': self.side()})
        differ = Probe('b.jai', {'jaic': self.side(stdout='x'), 'reference': self.side(stdout='y')}, ['stdout'])
        text = cmd_probe.render([agree, differ], show=False)
        self.assertIn('AGREE  a.jai', text)
        self.assertIn('DIFFER  b.jai', text)
        self.assertIn('differs in: stdout', text)
        self.assertTrue(text.endswith('2 probed, 1 agree, 1 differ'))

    def test_json_carries_both_sides(self):
        data = Probe('a.jai', {'jaic': self.side(), 'reference': self.side(exit=2)}, ['exit']).to_json()
        self.assertFalse(data['agree'])
        self.assertEqual(data['sides']['reference']['exit'], 2)
        json.dumps(data)


class ProbeFileTests(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        root = Path(self.dir.name)
        self.fake = root / 'fake'
        self.fake.write_text(FAKE)
        self.fake.chmod(self.fake.stat().st_mode | stat.S_IEXEC)
        self.good = root / 'good.jai'
        self.good.write_text('main :: () {}\n')
        self.broken = root / 'broken.jai'
        self.broken.write_text('BROKEN\n')
        self.compilers = {name: Compiler(name, self.fake, PlainDialect()) for name in cmd_probe.SIDES}

    def probe(self, source, **env):
        with mock.patch.object(cmd_probe, 'Sandbox', NoSandbox), \
                mock.patch.object(cmd_probe, 'install_reads', lambda compiler: []), \
                mock.patch.dict(os.environ, env):
            return cmd_probe.probe_file(source, self.compilers, Path(self.dir.name) / 'scratch', arguments())

    def test_identical_compilers_agree_and_run_the_program(self):
        probe = self.probe(self.good, FAKE_OUT='hi')
        self.assertTrue(probe.agree)
        self.assertEqual(probe.sides['jaic'].stdout, 'hi')
        self.assertTrue(probe.sides['jaic'].compiled)

    def test_a_program_that_does_not_compile_reports_the_check(self):
        probe = self.probe(self.broken)
        side = probe.sides['jaic']
        self.assertFalse(side.compiled)
        self.assertEqual((side.exit, side.stdout), (1, ''))

    def test_the_exit_code_of_the_run_is_kept(self):
        self.assertEqual(self.probe(self.good, FAKE_EXIT='3').sides['reference'].exit, 3)

    def test_a_difference_is_found(self):
        pair = [Side('jaic', True, 0, 'a', ''), Side('reference', True, 0, 'b', '')]
        with mock.patch.object(cmd_probe, 'probe_side', side_effect=pair):
            probe = cmd_probe.probe_file(self.good, self.compilers, Path(self.dir.name), arguments())
        self.assertEqual(probe.differences, ['stdout'])


class ReferencePathTests(unittest.TestCase):
    def test_the_option_wins_over_the_environment(self):
        with mock.patch.dict(os.environ, {cmd_probe.REFERENCE_ENV: '/env/jai'}):
            self.assertEqual(cmd_probe.reference_path(argparse.Namespace(reference=Path('/opt/jai'))), Path('/opt/jai'))
            self.assertEqual(cmd_probe.reference_path(argparse.Namespace(reference=None)), Path('/env/jai'))

    def test_without_either_the_command_stops(self):
        with mock.patch.dict(os.environ, {}, clear=True), self.assertRaises(SystemExit):
            cmd_probe.reference_path(argparse.Namespace(reference=None))


if __name__ == '__main__':
    unittest.main()
