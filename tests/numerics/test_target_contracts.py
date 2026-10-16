"""Target-routing policy tests; these are not native/WASM execution evidence."""

import ast
from pathlib import Path
import unittest

from target_contracts import GETRECT_CASES, PORTABLE_CASES, target_contract


class TargetContracts(unittest.TestCase):
    def test_default_getrect_wasm_is_explicitly_unavailable(self):
        for source in GETRECT_CASES:
            with self.subTest(source=source):
                contract = target_contract(source, "wasm")
                self.assertFalse(contract.available)
                self.assertIn("Input.Event/Window_Type", contract.reason)

    def test_numerical_wasm_coverage_is_retained(self):
        for source in PORTABLE_CASES:
            with self.subTest(source=source):
                self.assertTrue(target_contract(source, "wasm").available)

    def test_all_existing_primary_cases_are_explicitly_inventoried(self):
        tree = ast.parse(Path(__file__).with_name("run.py").read_text())
        cases = next(
            ast.literal_eval(node.value)
            for node in tree.body
            if isinstance(node, ast.Assign)
            and any(isinstance(t, ast.Name) and t.id == "tests" for t in node.targets)
        )
        self.assertEqual(len(cases), 21)
        for source, _ in cases.values():
            for target in ["linux", "windows", "macos", "wasm"]:
                with self.subTest(source=source, target=target):
                    target_contract(source, target)

    def test_named_pasteboard_does_not_gain_linux_shutdown(self):
        source = "stdlib/GetRect/tests/native_clipboard.jai"
        self.assertTrue(target_contract(source, "macos").available)
        for target in ["linux", "windows", "wasm"]:
            contract = target_contract(source, target)
            self.assertFalse(contract.available)
            self.assertEqual(
                contract.alternative, "stdlib/GetRect/tests/clipboard_unavailable.jai"
            )

    def test_linux_cursor_callbacks_remain_enabled(self):
        self.assertTrue(
            target_contract("tests/numerics/cursor_callbacks.jai", "linux").available
        )
        self.assertFalse(
            target_contract("tests/numerics/native_ui_cursors.jai", "linux").available
        )
        self.assertTrue(
            target_contract("tests/numerics/native_ui_cursors.jai", "windows").available
        )
        self.assertTrue(
            target_contract("tests/numerics/native_ui_cursors.jai", "macos").available
        )

    def test_unavailable_provider_negatives_are_targeted(self):
        source = "stdlib/GetRect/tests/cursors_unavailable.jai"
        self.assertTrue(target_contract(source, "linux").available)
        self.assertTrue(target_contract(source, "wasm").available)
        self.assertFalse(target_contract(source, "macos").available)

    def test_real_linux_clipboard_is_distinct_from_the_mac_named_provider(self):
        source = "tests/numerics/clipboard_text_actions_linux.jai"
        self.assertTrue(target_contract(source, "linux").available)
        for target in ["macos", "windows", "wasm"]:
            self.assertFalse(target_contract(source, target).available)

    def test_unknown_fixture_never_defaults_to_skip_or_pass(self):
        with self.assertRaisesRegex(ValueError, "uninventoried fixture"):
            target_contract("tests/numerics/unreviewed.jai", "wasm")
        with self.assertRaisesRegex(ValueError, "uninventoried target"):
            target_contract("tests/numerics/core.jai", "unknown")


if __name__ == "__main__":
    unittest.main()
