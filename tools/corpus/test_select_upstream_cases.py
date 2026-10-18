import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import select_upstream_cases as sel

CASES = [
    {"id": "a", "path": "o--r/x.jai", "only": ["windows-x64"], "only_reason": "r"},
    {"id": "b", "path": "o--s/x.jai", "only": ["linux-x64", "windows-x64"], "link": ["p--t"]},
    {"id": "c", "path": "o--u/x.jai", "only": ["windows-x64"], "disabled": "broken"},
    {"id": "d", "path": "o--v/x.jai"},
    {"id": "e", "path": "o--w/x.jai", "only": ["linux-x64"]},
]


class SelectTests(unittest.TestCase):
    def test_selects_enabled_cases_naming_the_platform(self):
        self.assertEqual([c["id"] for c in sel.select(CASES, "windows-x64")], ["a", "b"])
        self.assertEqual([c["id"] for c in sel.select(CASES, "linux-x64")], ["b", "e"])

    def test_repositories_cover_projects_and_links_once(self):
        chosen = sel.select(CASES, "windows-x64")
        self.assertEqual(sel.repositories(chosen), ["o/r", "o/s", "p/t"])

    def test_the_real_cases_select_the_x64_windows_checks(self):
        ids = [c["id"] for c in sel.select(sel.load(), "windows-x64")]
        for wanted in (
            "ttwj-28-28.5-get-cpu-info-feature-flags",
            "chess-jai-ui-check",
            "chess-jai-engine-check",
        ):
            self.assertIn(wanted, ids)
        self.assertTrue(any(i.startswith("ttwj-28-28.7-") for i in ids))
        self.assertTrue(any(i.startswith("ttwj-30-jai-raylib-build") for i in ids))


if __name__ == "__main__":
    unittest.main()
