import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench.sandbox import Sandbox, environment


class ProfileTests(unittest.TestCase):
    def test_the_profile_denies_the_network_and_writes_outside_the_given_paths(self):
        profile = Sandbox(reads=[Path('/opt/compiler')], writes=[Path('/scratch/run')]).seatbelt_profile()
        self.assertIn('(deny network*)', profile)
        self.assertIn('(deny file-write*)', profile)
        self.assertIn('(subpath "/opt/compiler")', profile)
        write_rule = next(line for line in profile.splitlines() if line.startswith('(allow file-write*'))
        self.assertIn('(subpath "/scratch/run")', write_rule)
        self.assertNotIn('/opt/compiler', write_rule)

    def test_bwrap_mounts_reads_read_only_and_drops_the_network(self):
        arguments = Sandbox(reads=[Path('/opt/compiler')], writes=[Path('/scratch/run')]).bwrap_arguments()
        self.assertIn('--unshare-net', arguments)
        self.assertIn(['--ro-bind', '/opt/compiler', '/opt/compiler'],
                      [arguments[i:i + 3] for i in range(len(arguments))])
        self.assertIn(['--bind', '/scratch/run', '/scratch/run'], [arguments[i:i + 3] for i in range(len(arguments))])


@unittest.skipUnless(sys.platform == 'darwin' and not Sandbox.available(), 'needs sandbox-exec')
class SeatbeltTests(unittest.TestCase):
    def run_inside(self, *command: str) -> int:
        with tempfile.TemporaryDirectory() as scratch:
            sandbox = Sandbox(writes=[Path(scratch)])
            return subprocess.run(sandbox.wrap(list(command)), cwd=scratch, env=environment(Path(scratch)),
                                  capture_output=True).returncode

    def test_the_scratch_folder_is_writable(self):
        self.assertEqual(self.run_inside('/usr/bin/touch', 'made-here'), 0)

    def test_the_home_folder_is_neither_readable_nor_writable(self):
        self.assertNotEqual(self.run_inside('/bin/ls', str(Path.home() / 'Library')), 0)
        self.assertNotEqual(self.run_inside('/usr/bin/touch', str(Path.home() / '.bench-sandbox-probe')), 0)

    def test_there_is_no_network(self):
        self.assertNotEqual(self.run_inside('/usr/bin/curl', '-s', '--max-time', '5', 'https://example.com'), 0)


if __name__ == '__main__':
    unittest.main()
