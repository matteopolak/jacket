"""Source contracts only; Windows execution is a separate required gate."""

from pathlib import Path
import re
import unittest

from native_bindings.config import LIBRARIES

ROOT = Path(__file__).resolve().parents[1]
SYMBOLS = {"_beginthreadex", "_endthreadex", "getenv", "_wgetenv"}


class WindowsCrtProvider(unittest.TestCase):
    def test_generator_uses_sdk_dynamic_import_library(self):
        # ucrt.lib is the SDK DLL import library. libucrt.lib and the MSVC
        # startup archive msvcrt.lib are not dynamically loadable providers.
        self.assertEqual(LIBRARIES.get("Crt"), "ucrt")

    def test_generated_private_library_mapping(self):
        source = (ROOT / "stdlib/Windows/declarations.jai").read_text()
        self.assertTrue('Crt :: #library,system "ucrt";' in source)
        self.assertLess(source.index("Crt ::"), source.index("#scope_export"))

    def test_all_reviewed_crt_signatures_preserved(self):
        reviewed = (ROOT / "docs/specs/stdlib/modules/windows.md").read_text()
        source = (ROOT / "stdlib/Windows/declarations.jai").read_text()
        signatures = re.findall(r"`([^`\n]+#foreign Crt)`", reviewed)
        self.assertEqual({line.split(" ::", 1)[0] for line in signatures}, SYMBOLS)
        for signature in signatures:
            self.assertIn(signature + ";", source)


if __name__ == "__main__":
    unittest.main()
