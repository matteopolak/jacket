"""`dev nvim` fails, with a hint, when it cannot run the Neovim plugin tests."""
import argparse
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from unittest.mock import patch

import dev


class NvimTests(unittest.TestCase):
    def run_nvim(self, found, banner):
        args = argparse.Namespace(unit=True, jobs="1")
        out = io.StringIO()
        with patch("shutil.which", return_value=found), \
                patch("subprocess.run", return_value=type("R", (), {"stdout": banner})()), \
                redirect_stdout(out), redirect_stderr(out):
            code = dev.cmd_nvim(args)
        return code, out.getvalue()

    def test_a_missing_nvim_is_an_error_with_an_install_hint(self):
        code, text = self.run_nvim(None, "")
        self.assertEqual(code, 1)
        self.assertIn("0.11", text)
        self.assertIn("install", text.lower())

    def test_an_old_nvim_is_an_error(self):
        code, text = self.run_nvim("/usr/bin/nvim", "NVIM v0.10.4\nBuild type: Release\n")
        self.assertEqual(code, 1)
        self.assertIn("NVIM v0.10.4", text)

    def test_an_unreadable_version_is_an_error(self):
        code, _ = self.run_nvim("/usr/bin/nvim", "")
        self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()
