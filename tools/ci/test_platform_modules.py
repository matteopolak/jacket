import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import platform_modules


class PlatformModulesTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        for folder, text in {
            "Mac/module.jai": "x :: OS == .MACOS;",
            "Mac/tests/t.jai": "",
            "Neutral/module.jai": "y :: 1;",
            "Neutral/tests/t.jai": "z :: OS == .MACOS;",
            "Extensions/Gui/a.m": "#import <Cocoa/Cocoa.h>",
            "internal/win/a.c": "#include <windows.h>",
        }.items():
            path = root / "stdlib" / folder
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.old = (platform_modules.ROOT, platform_modules.STDLIB)
        platform_modules.ROOT, platform_modules.STDLIB = root, root / "stdlib"

    def tearDown(self):
        platform_modules.ROOT, platform_modules.STDLIB = self.old
        self.tmp.cleanup()

    def test_a_module_is_listed_for_the_os_its_own_sources_name(self):
        self.assertEqual(platform_modules.modules("macos"), ["stdlib/Mac", "stdlib/Extensions/Gui"])
        self.assertEqual(platform_modules.modules("windows"), ["stdlib/internal/win"])

    def test_tests_do_not_make_a_module_specific(self):
        self.assertNotIn("stdlib/Neutral", platform_modules.modules("macos"))


if __name__ == "__main__":
    unittest.main()
