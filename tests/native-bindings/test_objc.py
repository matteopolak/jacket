import sys
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/native_bindings'))
from objc import materialize_methods
from generate import procedure_parts


class ObjCTests(unittest.TestCase):
    def test_typed_return_and_parameter_labels(self):
        code = '''Object :: struct {
    value :: (self: *Object, first: u32, next: u32) -> NSRect;
}'''
        output = materialize_methods('Object', code, procedure_parts)
        self.assertIn('value:next:', output)
        self.assertIn('-> NSRect #c_call', output)
        self.assertIn('binding_objc_send_stret', output)

    def test_underscore_selector_and_class_receiver(self):
        code = '''Object :: struct {
    makeWithBytes_length :: (bytes: *void, length: u64) -> *Object;
}'''
        output = materialize_methods('Object', code, procedure_parts)
        self.assertIn('makeWithBytes:length:', output)
        self.assertIn('binding_objc_class("Object")', output)


if __name__ == '__main__':
    unittest.main()
