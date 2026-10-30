"""The Windows parts of the harness. The first group checks text builders and platform switches on any OS
(the OS calls are replaced); the second runs the real Job Object and AppContainer code and needs Windows."""
import argparse
import ctypes
import os
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path
from unittest import mock

from bench import cmd_wsb, common, quiet, winapi, winsandbox
from bench.dialects import JaicDialect, Mode, PlainDialect, Request, executable_file
from bench.measure import run_measured
from bench.sandbox import Sandbox, SandboxedCommand, environment

WINDOWS = sys.platform == 'win32'


class PureHelperTests(unittest.TestCase):
    def test_the_environment_block_is_sorted_ignoring_case_and_double_terminated(self):
        block = winapi.environment_block({'b': '2', 'A': '1', 'C': '3'})
        self.assertEqual(block, 'A=1\0b=2\0C=3\0\0')

    def test_busy_cores_counts_the_non_idle_share_of_all_cores(self):
        # Kernel time includes idle: 100 ticks passed on each of 4 cores' worth, 75 of them idle.
        self.assertEqual(winapi.busy_cores((0, 0, 0), (75, 80, 20), 4), 1.0)
        self.assertEqual(winapi.busy_cores((0, 0, 0), (100, 100, 0), 8), 0.0)
        self.assertEqual(winapi.busy_cores((5, 5, 5), (5, 5, 5), 8), 0.0)

    def test_units(self):
        self.assertEqual(winapi.seconds_from_100ns(25_000_000), 2.5)
        self.assertEqual(winapi.mib(3 * 2**20), 3)
        self.assertEqual([winapi.power_source_from_line_status(n) for n in (0, 1, 255)], ['battery', 'ac', 'unknown'])

    def test_the_power_plan_is_read_from_powercfg_output(self):
        text = 'Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)\n'
        self.assertEqual(winapi.parse_power_scheme(text), 'Balanced')
        self.assertEqual(winapi.parse_power_scheme('nothing here'), '')


class GrantTests(unittest.TestCase):
    def test_a_folder_grant_is_inherited_and_read_only(self):
        with tempfile.TemporaryDirectory() as folder:
            command = winsandbox.grant_command(Path(folder), 'S-1-15-2-1', write=False)
        self.assertEqual(command[0], 'icacls')
        self.assertEqual(command[2:], ['/grant', '*S-1-15-2-1:(OI)(CI)(RX)'])

    def test_the_scratch_grant_modifies_and_a_file_grant_has_no_inheritance(self):
        with tempfile.TemporaryDirectory() as folder:
            self.assertEqual(winsandbox.grant_command(Path(folder), 'S-1', write=True)[-1], '*S-1:(OI)(CI)(M)')
            file = Path(folder) / 'tool.exe'
            file.write_bytes(b'')
            self.assertEqual(winsandbox.grant_command(file, 'S-1', write=False)[-1], '*S-1:(RX)')

    def test_revoking_removes_only_the_container_grants(self):
        self.assertEqual(winsandbox.revoke_command(Path('x'), 'S-1')[1:], ['x', '/remove:g', '*S-1'])

    def test_loopback_exemption_commands(self):
        self.assertEqual(winsandbox.loopback_command('S-1', True), ['CheckNetIsolation', 'LoopbackExempt', '-a', '-p=S-1'])
        self.assertEqual(winsandbox.loopback_command('S-1', False)[2], '-d')

    def test_a_drive_root_or_the_home_folder_is_never_granted(self):
        with self.assertRaises(winsandbox.ContainerError):
            winsandbox.reject_broad(Path(Path.home().anchor))
        with self.assertRaises(winsandbox.ContainerError):
            winsandbox.reject_broad(Path.home())
        winsandbox.reject_broad(Path(tempfile.gettempdir()) / 'bench-scratch')

    def test_container_names_are_unique_and_valid(self):
        names = {winsandbox.new_container_name() for _ in range(5)}
        self.assertEqual(len(names), 5)
        self.assertTrue(all(name.replace('.', '').isalnum() for name in names))


class WindowsSandboxConfigTests(unittest.TestCase):
    def test_the_vm_has_no_network_and_maps_what_it_is_told(self):
        text = winsandbox.windows_sandbox_config(
            [(Path('C:/compilers'), 'C:\\bench\\ref', True), (Path('C:/out & more'), 'C:\\bench\\out', False)],
            'cmd.exe /c C:\\bench\\out\\run.cmd')
        root = ET.fromstring(text)
        self.assertEqual(root.findtext('Networking'), 'Disable')
        folders = root.findall('MappedFolders/MappedFolder')
        self.assertEqual([f.findtext('ReadOnly') for f in folders], ['true', 'false'])
        self.assertEqual(folders[1].findtext('HostFolder'), str(Path('C:/out & more')))
        self.assertEqual(root.findtext('LogonCommand/Command'), 'cmd.exe /c C:\\bench\\out\\run.cmd')


class WsbCommandTests(unittest.TestCase):
    def arguments(self, folder: Path, *command: str):
        parser = argparse.ArgumentParser()
        cmd_wsb.add_arguments(parser)
        return parser.parse_args(['--map', f'ref={folder}', '--out-dir', str(folder / 'out'), '--', *command])

    def test_placeholders_become_guest_folders_and_the_harness_runs_without_its_own_sandbox(self):
        with tempfile.TemporaryDirectory() as folder:
            config, script = cmd_wsb.build(self.arguments(Path(folder), 'compare', '--baseline', '{ref}\\jai.exe',
                                                          '--out-dir', '{out}'))
        self.assertIn('--baseline C:\\bench\\ref\\jai.exe', script)
        self.assertIn('--out-dir C:\\bench\\out', script)
        self.assertTrue(script.rstrip().endswith('shutdown /s /t 5'))
        self.assertIn('--no-sandbox', script)
        self.assertIn('<Networking>Disable</Networking>', config)
        self.assertIn('cmd.exe /c C:\\bench\\out\\run.cmd', config)

    def test_a_missing_command_is_refused(self):
        with tempfile.TemporaryDirectory() as folder, self.assertRaises(SystemExit):
            cmd_wsb.build(self.arguments(Path(folder)))


class PlatformSwitchTests(unittest.TestCase):
    def test_windows_commands_are_quoted_for_the_windows_shell(self):
        argv = ['python', 'C:\\Program Files\\x.py', '--a', 'b c']
        with mock.patch.object(sys, 'platform', 'win32'):
            line = common.join_command(argv)
            self.assertEqual(line, subprocess.list2cmdline(argv))
            self.assertEqual(common.split_command('"C:\\Program Files\\jailsp.exe" --stdio'),
                             ['C:\\Program Files\\jailsp.exe', '--stdio'])
            self.assertEqual(common.split_command('C:\\tools\\jailsp.exe'), ['C:\\tools\\jailsp.exe'])

    def test_posix_commands_still_use_shlex(self):
        with mock.patch.object(sys, 'platform', 'linux'):
            self.assertEqual(common.join_command(['a b', 'c']), "'a b' c")
            self.assertEqual(common.split_command("'a b' c"), ['a b', 'c'])

    def test_executables_get_the_platform_suffix_and_compilers_get_the_bare_name(self):
        scratch = Path('scratch')
        request = Request(scratch / 'main.jai', scratch, Mode.BUILD, (), scratch)
        with mock.patch('bench.dialects.EXE_SUFFIX', '.exe'):
            self.assertEqual(executable_file(scratch), scratch / 'program.exe')
            self.assertEqual(request.executable, scratch / 'program.exe')
            jaic = JaicDialect().invocation(Path('jaic.exe'), request)
            self.assertEqual(jaic.command[jaic.command.index('-exe') + 1], str(scratch / 'program'))
            self.assertEqual(jaic.executable, scratch / 'program.exe')
            plain = PlainDialect().invocation(Path('jai.exe'), request)
            self.assertEqual(plain.command[plain.command.index('-exe') + 1], 'program')
            self.assertEqual(plain.executable, scratch / 'program.exe')

    def test_the_sandbox_on_windows_wraps_nothing_into_the_command_line(self):
        sandbox = Sandbox()
        with mock.patch.object(sys, 'platform', 'win32'):
            wrapped = sandbox.wrap(['jai.exe', 'main.jai'])
        self.assertIsInstance(wrapped, SandboxedCommand)
        self.assertEqual(list(wrapped), ['jai.exe', 'main.jai'])
        self.assertIs(wrapped.sandbox, sandbox)

    def test_the_container_is_made_on_first_use_with_the_sandbox_paths_and_closed_once(self):
        sandbox = Sandbox(reads=[Path('r')], writes=[Path('w')], loopback=True)
        with mock.patch('bench.winsandbox.AppContainer') as container:
            container.return_value.sid_pointer = 1234
            self.assertEqual(sandbox.container_sid(), 1234)
            self.assertEqual(sandbox.container_sid(), 1234)
            container.assert_called_once_with([Path('r')], [Path('w')], True)
            sandbox.close()
            sandbox.close()
            container.return_value.close.assert_called_once_with()

    def test_the_sandbox_reports_windows_unavailability_honestly(self):
        with mock.patch.object(sys, 'platform', 'win32'), \
                mock.patch('bench.winsandbox.unavailable_reason', return_value='icacls is missing'):
            self.assertEqual(Sandbox.available(), 'icacls is missing')

    def test_temporary_folders_are_set_for_windows_programs(self):
        with tempfile.TemporaryDirectory() as folder, mock.patch.object(sys, 'platform', 'win32'):
            env = environment(Path(folder))
        self.assertEqual(env['TEMP'], env['TMP'])
        self.assertEqual(env['TEMP'], env['TMPDIR'])

    def test_a_windows_run_goes_through_the_job_object_code(self):
        fake = mock.Mock(wall=1.5, cpu=2.0, rss_mib=30.0, commit_mib=40.0, returncode=0, timed_out=False)
        with mock.patch.object(sys, 'platform', 'win32'), mock.patch('bench.winjob.run', return_value=fake) as run, \
                mock.patch('bench.winjob.open_null'):
            result = run_measured(['jai.exe'], timeout=5)
        run.assert_called_once()
        self.assertEqual((result.wall, result.cpu, result.rss_mib, result.commit_mib), (1.5, 2.0, 30.0, 40.0))
        self.assertTrue(result.ok)

    def test_a_sandboxed_windows_run_passes_the_container_sid(self):
        sandbox = mock.Mock()
        sandbox.container_sid.return_value = 99
        command = SandboxedCommand(['jai.exe'], sandbox)
        fake = mock.Mock(wall=1, cpu=1, rss_mib=1, commit_mib=1, returncode=0, timed_out=False)
        with mock.patch.object(sys, 'platform', 'win32'), mock.patch('bench.winjob.run', return_value=fake) as run, \
                mock.patch('bench.winjob.open_null'):
            run_measured(command, timeout=5)
        self.assertEqual(run.call_args.args[-1], 99)


class QuietOnWindowsTests(unittest.TestCase):
    def test_the_load_is_the_busy_core_count_three_times(self):
        with mock.patch.object(sys, 'platform', 'win32'), mock.patch('bench.winapi.sample_busy_cores', return_value=1.2345):
            self.assertEqual(quiet.load_average(), (1.23, 1.23, 1.23))

    def test_the_threshold_rules_judge_busy_cores_like_a_load_average(self):
        calm = {'time': 0, 'cpus': 8, 'load': [0.4] * 3, 'thermal': {'throttled': None}, 'power': 'ac',
                'free_mib': None, 'noisy': []}
        busy = {**calm, 'load': [5.0] * 3}
        limit = quiet.default_max_load(8)
        self.assertEqual(quiet.block_flags(calm, calm, limit), [])
        self.assertTrue(quiet.block_flags(calm, busy, limit))
        errors, _ = quiet.start_problems(busy, limit)
        self.assertTrue(errors)

    def test_noisy_processes_are_read_from_tasklist(self):
        text = '"cargo.exe","1","Console","1","10 K"\n"Link.exe","2","Console","1","1 K"\n"notepad.exe","3","Console","1","1 K"\n'
        self.assertEqual(quiet.parse_tasklist(text), ['cargo', 'link'])
        self.assertEqual(quiet.parse_tasklist(''), [])

    def test_the_window_purge_is_reported_as_not_done(self):
        with mock.patch.object(sys, 'platform', 'win32'):
            done, note = quiet.purge_caches()
        self.assertFalse(done)
        self.assertIn('Windows', note)


# ------------------------------------------------------------------------------------ real Windows

PYTHON = sys.executable


@unittest.skipUnless(WINDOWS, 'needs Windows')
class JobObjectTests(unittest.TestCase):
    def test_structures_match_the_windows_headers(self):
        if ctypes.sizeof(ctypes.c_void_p) == 8:
            self.assertEqual(ctypes.sizeof(winapi.JOBOBJECT_EXTENDED_LIMIT_INFORMATION), 144)
            self.assertEqual(ctypes.sizeof(winapi.JOBOBJECT_BASIC_ACCOUNTING_INFORMATION), 48)
            self.assertEqual(ctypes.sizeof(winapi.PROCESS_MEMORY_COUNTERS), 72)
            self.assertEqual(ctypes.sizeof(winapi.STARTUPINFOW), 104)
            self.assertEqual(ctypes.sizeof(winapi.STARTUPINFOEXW), 112)
            self.assertEqual(ctypes.sizeof(winapi.MEMORYSTATUSEX), 64)
            self.assertEqual(ctypes.sizeof(winapi.SECURITY_CAPABILITIES), 24)
        self.assertEqual(ctypes.sizeof(winapi.FILETIME), 8)

    def test_exit_code_output_and_wall_time(self):
        run = run_measured([PYTHON, '-c', 'import sys; print("out"); print("err", file=sys.stderr); sys.exit(3)'], timeout=60)
        self.assertEqual((run.returncode, run.stdout.strip(), run.stderr.strip()), (3, 'out', 'err'))
        self.assertGreater(run.wall, 0)
        self.assertGreater(run.rss_mib, 1)
        # Peak commit and peak working set are not ordered: the working set also counts shared pages (DLLs).
        self.assertGreater(run.commit_mib, 0)

    def test_cpu_time_includes_a_child_process(self):
        child = 'import time; t=time.perf_counter()\nwhile time.perf_counter()-t < 0.6: pass'
        parent = f'import subprocess, sys; subprocess.run([sys.executable, "-c", {child!r}])'
        run = run_measured([PYTHON, '-c', parent], timeout=60)
        self.assertTrue(run.ok, run.stderr)
        self.assertGreater(run.cpu, 0.5)

    def test_a_timeout_kills_the_whole_tree(self):
        marker = tempfile.NamedTemporaryFile(delete=False)
        marker.close()
        child = f'import time; time.sleep(3); open({marker.name!r}, "w").write("survived")'
        parent = f'import subprocess, sys, time; subprocess.Popen([sys.executable, "-c", {child!r}]); time.sleep(60)'
        try:
            run = run_measured([PYTHON, '-c', parent], timeout=1)
            self.assertTrue(run.timed_out)
            self.assertLess(run.wall, 20)
            import time
            time.sleep(4)
            self.assertEqual(Path(marker.name).read_text(), '')
        finally:
            os.unlink(marker.name)

    def test_stdin_bytes_reach_the_program(self):
        run = run_measured([PYTHON, '-c', 'import sys; print(sys.stdin.read().upper())'], stdin_bytes=b'abc', timeout=60)
        self.assertEqual(run.stdout.strip(), 'ABC')

    def test_the_environment_and_directory_are_used(self):
        with tempfile.TemporaryDirectory() as folder:
            env = {**os.environ, 'BENCH_PROBE': 'yes'}
            run = run_measured([PYTHON, '-c', 'import os; print(os.environ["BENCH_PROBE"], os.getcwd())'], cwd=Path(folder),
                               env=env, timeout=60)
            self.assertEqual(run.stdout.split()[0], 'yes')
            self.assertEqual(Path(run.stdout.split(None, 1)[1].strip()).resolve(), Path(folder).resolve())

    def test_a_missing_program_raises(self):
        with self.assertRaises(OSError):
            run_measured(['definitely-not-a-program.exe'], timeout=5)

    def test_machine_facts_and_snapshot(self):
        facts = winapi.machine_facts()
        self.assertTrue(facts['cpu'])
        self.assertGreater(facts['memory_bytes'], 0)
        snap = quiet.snapshot()
        self.assertEqual(len(snap['load']), 3)
        self.assertIn(snap['power'], ('ac', 'battery', 'unknown'))


@unittest.skipUnless(WINDOWS and not Sandbox.available(), 'needs Windows with AppContainer support')
class AppContainerTests(unittest.TestCase):
    def run_inside(self, scratch: Path, code: str, **settings):
        sandbox = Sandbox(reads=[Path(sys.base_prefix)], writes=[scratch], **settings)
        try:
            return run_measured(sandbox.wrap([PYTHON, '-c', code]), cwd=scratch, env=environment(scratch), timeout=60)
        finally:
            sandbox.close()

    def test_the_scratch_folder_is_writable_and_the_home_folder_is_not_readable(self):
        with tempfile.TemporaryDirectory() as folder:
            scratch = Path(folder)
            self.assertTrue(self.run_inside(scratch, 'open("made-here", "w").write("x")').ok)
            self.assertTrue((scratch / 'made-here').exists())
            code = f'import os; os.listdir({str(Path.home())!r})'
            self.assertFalse(self.run_inside(scratch, code).ok)

    def test_there_is_no_network(self):
        with tempfile.TemporaryDirectory() as folder:
            code = 'import socket; socket.create_connection(("example.com", 80), timeout=5)'
            self.assertFalse(self.run_inside(Path(folder), code).ok)

    def test_the_grants_are_removed_when_the_sandbox_closes(self):
        with tempfile.TemporaryDirectory() as folder:
            self.run_inside(Path(folder), 'pass')
            listing = subprocess.run(['icacls', folder], capture_output=True, text=True).stdout
            self.assertNotIn('S-1-15-2-', listing)

    def test_the_container_process_is_measured_through_the_job(self):
        with tempfile.TemporaryDirectory() as folder:
            run = self.run_inside(Path(folder), 'import time\nt=time.perf_counter()\nwhile time.perf_counter()-t<0.4: pass')
            self.assertTrue(run.ok, run.stderr)
            self.assertGreater(run.cpu, 0.3)
            self.assertGreater(run.rss_mib, 1)


if __name__ == '__main__':
    unittest.main()
