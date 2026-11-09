import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench import phases
from bench.common import default_binary
from toolkit.repo import ROOT

JAIC = default_binary('jaic')

LINKS = 'main :: () {}\n'
# The compiler accepts the call; only the system linker finds the symbol missing.
DOES_NOT_LINK = 'libc :: #system_library "libc";\nno_such_symbol_for_link_test :: () #foreign libc;\nmain :: () { no_such_symbol_for_link_test(); }\n'


@unittest.skipUnless(JAIC.exists() and sys.platform != 'win32', 'needs a release jaic and an OS with a system linker')
class PhaseLinkTests(unittest.TestCase):
    def build(self, source):
        with tempfile.TemporaryDirectory() as scratch:
            where = Path(scratch)
            (where / 'main.jai').write_text(source, encoding='utf-8')
            wrapper = phases.program(where / 'main.jai', where, writes_executable=True, optimized=False, run_main=False)
            (where / phases.WRAPPER_NAME).write_text(wrapper, encoding='utf-8')
            shutil.copy(phases.LINK_COMMAND, where / phases.LINK_COMMAND.name)
            env = dict(os.environ, JAIC_STDLIB=str(ROOT / 'stdlib'))
            return subprocess.run([str(JAIC), phases.WRAPPER_NAME], cwd=where, env=env, capture_output=True, text=True, timeout=120)

    def test_a_successful_link_is_timed(self):
        out = self.build(LINKS)
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertIn('bench-phase: link ', out.stdout)

    def test_a_failed_link_fails_the_build_and_reports_no_link_time(self):
        out = self.build(DOES_NOT_LINK)
        self.assertNotEqual(out.returncode, 0, out.stdout)
        self.assertNotIn('bench-phase: link ', out.stdout)
        self.assertIn('the linker failed', out.stderr)


if __name__ == '__main__':
    unittest.main()
