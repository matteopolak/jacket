"""Real Meson child-discovery regression; run with the prepared tool venv Python."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import venv

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import window_render_mesa_windows as mesa


class MesonPython(unittest.TestCase):
    def test_prepared_python_beats_unprepared_python3_on_path(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "target", prefix="meson python's ") as temporary:
            directory = Path(temporary)
            wrong = directory / "unprepared"
            venv.EnvBuilder(with_pip=False).create(wrong)
            binaries = wrong / ("Scripts" if os.name == "nt" else "bin")
            if os.name == "nt":
                shutil.copy2(binaries / "python.exe", binaries / "python3.exe")
            env = dict(os.environ, PATH=str(binaries) + os.pathsep + os.environ["PATH"],
                       PYTHONNOUSERSITE="1")
            env.pop("PYTHONHOME", None)
            env.pop("PYTHONPATH", None)
            project = directory / "project"
            project.mkdir()
            metadata = directory / "child.json"
            (project / "probe.py").write_text(
                "import json,sys,importlib.metadata,mako,yaml\n"
                "from pathlib import Path\n"
                "Path(sys.argv[1]).write_text(json.dumps({\n"
                "'executable':sys.executable,'prefix':sys.prefix,\n"
                "'Mako':importlib.metadata.version('Mako'),\n"
                "'PyYAML':importlib.metadata.version('PyYAML')}))\n")
            (project / "meson.build").write_text(
                "project('original-python-discovery-regression', meson_version: '==1.8.3')\n"
                "child = find_program('python3', 'python', version: '>=3.8')\n"
                "run_command(child, files('probe.py'), " + "'" + metadata.as_posix().replace("'", "\\'") + "', check:true)\n")
            base = [sys.executable, "-m", "mesonbuild.mesonmain", "setup"]
            # Same discovery as Mesa: a real unprepared Python wins without the
            # native config, despite Meson itself running in the prepared venv.
            control = subprocess.run([*base, directory / "control", project], env=env,
                                     capture_output=True, text=True)
            self.assertNotEqual(control.returncode, 0, control.stdout + control.stderr)
            expected_wrong = str(binaries / ("python3.exe" if os.name == "nt" else "python3"))
            self.assertIn(expected_wrong.replace("\\", "/").lower(),
                          control.stdout.replace("\\", "/").lower())
            self.assertFalse(metadata.exists())
            native = directory / "prepared.ini"
            mesa.write_native_machine_file(native, sys.executable)
            result = subprocess.run([*base, directory / "selected", project, "--native-file", native],
                                    env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            actual = json.loads(metadata.read_text())
            self.assertEqual(Path(actual["executable"]).absolute(), Path(sys.executable).absolute())
            self.assertEqual(actual["prefix"], sys.prefix)
            self.assertEqual(actual["Mako"], mesa.PACKAGES["Mako"])
            self.assertEqual(actual["PyYAML"], mesa.PACKAGES["PyYAML"])
            fingerprint = mesa.distribution_fingerprint("Mako")
            self.assertEqual(fingerprint["version"], mesa.PACKAGES["Mako"])
            self.assertGreater(fingerprint["files"], 0)


if __name__ == "__main__":
    unittest.main()
