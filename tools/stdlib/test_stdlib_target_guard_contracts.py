"""Reviewed three-module guard boundaries, not a failure-derived golden refresh."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "linux-x64",
    "linux-arm64",
    "macos-x64",
    "macos-arm64",
    "windows-x64",
    "windows-arm64",
    "wasm",
)
SUPPORTED_OS = {
    "macOS": {"macos"},
    "nvidia_aftermath": {"windows"},
    "nvtt": {"linux", "macos", "windows"},
}


def owned_expectations(text):
    result = {}
    for line in text.splitlines():
        if not line.strip() or line.startswith("#") or line.startswith("skip "):
            continue
        key, diagnostic = line.split(": ", 1)
        module = key.split()[0]
        if module in {*SUPPORTED_OS, "macos"}:
            if key in result:
                raise ValueError("duplicate owned target expectation: " + key)
            result[key] = diagnostic
    return result


class ReviewedTargetGuards(unittest.TestCase):
    def test_exact_required_negatives_leave_all_reviewed_positives_required(self):
        actual = owned_expectations((ROOT / "tests/compiler/stdlib-targets.txt").read_text())
        expected = {
            f"{module} {target}": f"{module}/module.jai: #assert failed: {module} requires its documented native platform"
            for module, supported in SUPPORTED_OS.items()
            for target in TARGETS
            if target.split("-")[0] not in supported
        }
        self.assertEqual(actual, expected)
        self.assertEqual(len(expected), 11)
        for module, supported in SUPPORTED_OS.items():
            for target in TARGETS:
                if target.split("-")[0] in supported:
                    self.assertNotIn(f"{module} {target}", actual)

    def test_public_name_and_facade_guard_come_from_current_canonical_sources(self):
        for module in SUPPORTED_OS:
            self.assertTrue((ROOT / "stdlib" / module / "module.jai").is_file())
            first_line = (
                (ROOT / "stdlib" / module / "module.jai").read_text().splitlines()[0]
            )
            self.assertTrue(first_line.startswith("#assert OS == "))
            self.assertIn(
                f'"{module} requires its documented native platform"', first_line
            )
        # Probe actual names, not case-insensitive host filesystem lookup.
        names = {path.name for path in (ROOT / "stdlib").iterdir()}
        self.assertIn("macOS", names)
        self.assertNotIn("macos", names)

    def test_required_guard_is_not_satisfied_by_arbitrary_binding_errors(self):
        actual = owned_expectations((ROOT / "tests/compiler/stdlib-targets.txt").read_text())
        for diagnostic in actual.values():
            self.assertIn("/module.jai: #assert failed:", diagnostic)
            self.assertNotIn("unknown", diagnostic)
            self.assertNotIn("Windows.jai:", diagnostic)


if __name__ == "__main__":
    unittest.main()
