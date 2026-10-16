"""Reviewed receiver policy and preserved public parameters in generation."""

import unittest

from generate_appkit import method


class Receivers(unittest.TestCase):
    def test_view_uses_supplied_instance(self):
        code = method(
            "NSView",
            "convertPoint",
            "self: id, point: NSPoint, fromView: *NSView",
            "NSPoint",
        )
        self.assertIn(
            'send(self, binding_selector("convertPoint:fromView:"), point, fromView)',
            code,
        )

    def test_event_class_properties_keep_legacy_parameter(self):
        for name, result in (
            ("keyRepeatDelay", "NSTimeInterval"),
            ("keyRepeatInterval", "NSTimeInterval"),
            ("pressedMouseButtons", "NSUInteger"),
        ):
            with self.subTest(name=name):
                code = method("NSEvent", name, "self: *NSEvent", result)
                self.assertIn(f"{name} :: (self: *NSEvent) -> O.{result}", code)
                self.assertIn(
                    f'send(binding_class("NSEvent"), binding_selector("{name}"))', code
                )

    def test_other_instance_and_class_receivers_unchanged(self):
        self.assertIn(
            'send(self, binding_selector("isARepeat"))',
            method("NSEvent", "isARepeat", "self: *NSEvent", "BOOL"),
        )
        self.assertIn(
            'send(binding_class("NSApplication"), binding_selector("sharedApplication"))',
            method("NSApplication", "sharedApplication", "", "id"),
        )


if __name__ == "__main__":
    unittest.main()
