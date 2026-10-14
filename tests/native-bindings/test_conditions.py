import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/native_bindings'))
from verify import enabled


class ConfigurationTests(unittest.TestCase):
    def test_conditions_use_explicit_configuration(self):
        self.assertFalse(enabled('ENABLED && OS == .WINDOWS','windows','x64'))
        self.assertTrue(enabled('ENABLED && OS == .WINDOWS','windows','x64',{'ENABLED':True}))
        self.assertFalse(enabled('ENABLED && OS == .WINDOWS','linux','arm64',{'ENABLED':True}))
        self.assertTrue(enabled('!USE_GLUE_NATIVE_PORT','android','arm64',{'USE_GLUE_NATIVE_PORT':False}))

    def test_unknown_conditions_are_rejected(self):
        with self.assertRaises(ValueError):
            enabled('UNREVIEWED_FLAG','linux','x64')


if __name__ == '__main__':
    unittest.main()
