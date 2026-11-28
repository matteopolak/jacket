"""`sdk_tool` picks one tool per OS and exits when it is missing."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from developer_checks import build_generator_platform as generator


class SdkToolTests(unittest.TestCase):
    def test_a_linux_sdk_uses_its_own_bin_and_never_the_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "bin").mkdir()
            with patch("platform.system", return_value="Linux"), patch("shutil.which", return_value="/usr/bin/clang"):
                with self.assertRaisesRegex(SystemExit, "does not exist"):
                    generator.sdk_tool(tmp, "clang", "clang")
                tool = Path(tmp) / "bin" / generator.exe_name("clang")
                tool.write_text("")
                self.assertEqual(generator.sdk_tool(tmp, "clang", "clang"), str(tool))

    def test_macos_uses_xcodes_tool_and_exits_without_it(self):
        with patch("platform.system", return_value="Darwin"):
            with patch("shutil.which", return_value="/usr/bin/ar"):
                self.assertEqual(generator.sdk_tool("/sdk", "llvm-ar", "ar"), "/usr/bin/ar")
            with patch("shutil.which", return_value=None):
                with self.assertRaisesRegex(SystemExit, "xcode-select"):
                    generator.sdk_tool("/sdk", "llvm-ar", "ar")


if __name__ == "__main__":
    unittest.main()
