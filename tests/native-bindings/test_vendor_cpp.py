import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'tools/native_bindings'))
from vendor_cpp import jai_type, default_value


class VendorCppTests(unittest.TestCase):
    def test_const_reference_and_array_pointer_types(self):
        self.assertEqual(jai_type('const Surface &'), '*Surface')
        self.assertEqual(jai_type('const float *'), '*float')
        self.assertEqual(jai_type('unsigned int'), 'u32')

    def test_native_pointer_and_float_defaults(self):
        def value(code, native):
            parameter = {'init':'c', 'type':{'qualType':native},
                         'range':{'begin':{'offset':0}, 'end':{'offset':len(code)-1, 'tokLen':1}}}
            return default_value(parameter, code)
        self.assertEqual(value('float x = 0.f', 'float'), ' = 0.0')
        self.assertEqual(value('const float *params = 0', 'const float *'), ' = null')
        self.assertEqual(value('bool enabled = 0', 'bool'), ' = false')


if __name__ == '__main__':
    unittest.main()
