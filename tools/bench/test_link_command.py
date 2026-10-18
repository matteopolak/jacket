import subprocess
import sys
import unittest
from pathlib import Path

from common import default_binary

HERE = Path(__file__).parent
JAIC = default_binary('jaic')


class LinkCommandTests(unittest.TestCase):
    def arguments(self):
        out = subprocess.run([str(JAIC), 'test_link_command.jai'], cwd=HERE, capture_output=True, text=True, timeout=120)
        self.assertEqual(out.returncode, 0, out.stderr)
        return out.stdout.split()

    @unittest.skipUnless(JAIC.exists() and sys.platform != 'win32', 'needs a release jaic and an OS with a system linker')
    def test_inputs_follow_the_fixed_arguments(self):
        arguments = self.arguments()
        self.assertEqual(arguments[-5:], ['-o', 'out/program', 'a.o', 'b.o', '-lm'])
        if arguments[0] == 'xcrun':
            self.assertEqual(arguments[1:3], ['ld', '-arch'])
            self.assertIn('-lSystem', arguments)
            self.assertEqual(arguments[arguments.index('-syslibroot') + 1], '/sdk')
            self.assertEqual(arguments[arguments.index('-platform_version') + 2:][:2], ['99.0', '99.0'])
        else:
            self.assertEqual(arguments[0], 'cc')


if __name__ == '__main__':
    unittest.main()
