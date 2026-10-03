"""Check native dispatch ABI boundaries without calling any native runtime."""
import unittest

from author_objective_c_dispatch import author, selector_spelling


class DispatchContractTests(unittest.TestCase):
    def test_private_selector_prefix_is_preserved(self):
        self.assertEqual(selector_spelling('_privateName'), '_privateName')
        self.assertEqual(selector_spelling('_privateWith_argument_'), '_privateWith:argument:')
        self.assertEqual(selector_spelling('addSubView_'), 'addSubview:')

    def test_large_aggregate_uses_architecture_specific_entry(self):
        source = '''NSScreen :: struct {
    frame :: (self: *NSScreen) -> NSRect #foreign Native_Adapters "frame_adapter";
}
frame: Selector;
'''
        out, done, gaps = author('Objective_C/AppKit.jai', source)
        self.assertFalse(gaps)
        self.assertEqual(done[0]['return_abi'], 'x64-stret-arm64-direct')
        self.assertIn('#if CPU == .X64', out)
        self.assertIn('output: *NSRect', out)
        self.assertIn('objc_msgSend_stret', out)
        self.assertIn('-> NSRect #c_call', out)

    def test_sdk_void_return_never_reads_undefined_return_register(self):
        source = '''NSWindow :: struct {
    setTitle :: (self: *NSWindow, title: *NSString) -> id #foreign Native_Adapters "title_adapter";
}
setTitle_: Selector;
'''
        out, done, gaps = author('Objective_C/AppKit.jai', source)
        self.assertFalse(gaps)
        self.assertIn('-> void #c_call', out)
        self.assertIn('return null;', out)
        self.assertNotIn('return invoke', out)
        self.assertIn('sdk_signature_correction', done[0])

    def test_snapshot_initializers_use_sdk_object_pointer_return(self):
        source = '''GCMicroGamepadSnapshot :: struct {
    initWithSnapshotData :: (self: *GCMicroGamepadSnapshot, data: *NSData) -> GCMicroGamepadSnapshot #foreign Native_Adapters "init_adapter";
}
initWithSnapshotData_: Selector;
'''
        out, done, gaps = author('Objective_C/GameController.jai', source)
        self.assertFalse(gaps)
        self.assertIn('-> *GCMicroGamepadSnapshot', out)
        self.assertIn('sdk_signature_correction', done[0])

    def test_native_selector_parameter_cannot_collide_with_user_parameter(self):
        source = '''NSTimer :: struct {
    target :: (self: *NSTimer, selector: Selector) #foreign Native_Adapters "timer_adapter";
}
target_: Selector;
'''
        out, done, gaps = author('Objective_C/Foundation.jai', source)
        self.assertFalse(gaps)
        self.assertIn('sdk_selector: SEL, selector: Selector', out)

    def test_unknown_aggregate_return_stays_explicit_gap(self):
        source = '''UnreviewedClass :: struct {
    value :: (self: *UnreviewedClass) -> UnreviewedRecord #foreign Native_Adapters "unknown_adapter";
}
value: Selector;
'''
        out, done, gaps = author('Objective_C/AppKit.jai', source)
        self.assertFalse(done)
        self.assertEqual(gaps[0]['reason'], 'unclassified return ABI')
        self.assertIn('#foreign Native_Adapters', out)


if __name__ == '__main__':
    unittest.main()
