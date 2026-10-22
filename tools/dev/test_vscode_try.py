"""The throwaway editor profile must not silently run an installed toolchain."""
import json
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

import dev


class VscodeTryTests(unittest.TestCase):
    def test_terminal_and_editor_paths_select_one_toolchain(self):
        for platform, key, separator in [("darwin", "osx", ":"), ("linux", "linux", ":"), ("win32", "windows", ";")]:
            with self.subTest(platform=platform):
                settings = dev.vscode_settings(Path("/scratch/release"), Path("/scratch/jaifmt"), platform)
                env = settings[f"terminal.integrated.env.{key}"]
                self.assertEqual(env["PATH"], separator.join(["/scratch/release", "/scratch", "${env:PATH}"]))
                self.assertEqual(env["JAIC_STDLIB"], settings["jai.stdlib.path"])
                self.assertEqual(Path(settings["jai.compiler.path"]).parent, Path("/scratch/release"))

    def test_launcher_builds_jacket_and_writes_the_terminal_environment(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "release").mkdir()
            (root / "release" / ("jaic" + dev.EXE)).touch()
            formatter = root / "jaifmt"
            formatter.touch()
            with patch.object(dev.shutil, "which", return_value="available"), \
                 patch.object(dev, "cargo_build", return_value=0) as build, \
                 patch.object(dev, "jaifmt_path", return_value=formatter), \
                 patch.object(dev, "newest", return_value=0), \
                 patch.object(dev, "TARGET", root), \
                 patch.object(dev, "run", return_value=0), \
                 patch.object(dev.tempfile, "mkdtemp", return_value=str(root)), \
                 patch.object(dev.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)):
                self.assertEqual(dev.cmd_vscode_try(types.SimpleNamespace(debug=False, jobs="2", folder=tmp)), 0)
            args = build.call_args.args[0]
            self.assertIn("jacket", args)
            self.assertIn("jaic-cli", args)
            settings = json.loads((root / "user/User/settings.json").read_text())
            terminal = next(v for k, v in settings.items() if k.startswith("terminal.integrated.env."))
            self.assertTrue(terminal["PATH"].startswith(str(root / "release")))


if __name__ == "__main__":
    unittest.main()
