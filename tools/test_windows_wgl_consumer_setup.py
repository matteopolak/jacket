"""Original deployment/selection regressions, never native graphics evidence.

Select the accepted or patched producer tool/workflow explicitly. Binary inputs
are COFF-header decoding/copy fixtures only, never executed as drivers.
"""

import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import struct
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


class WglConsumerSetup(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        source = Path(os.environ["WGL_SETUP_TEST_SOURCE"])
        spec = importlib.util.spec_from_file_location("wgl_consumer_setup", source)
        cls.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.tool)
        cls.workflow = Path(os.environ["WGL_SETUP_TEST_WORKFLOW"]).read_text()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.driver = self.root / "curated-driver"
        self.driver.mkdir()

    def image(self, name, machine):
        header = bytearray(128)
        header[:2] = b"MZ"
        struct.pack_into("<I", header, 60, 64)
        header[64:68] = b"PE\0\0"
        struct.pack_into("<H", header, 68, machine)
        path = self.driver / name
        path.write_bytes(header)
        self.remove_fixture_metadata(path)
        return path

    def remove_fixture_metadata(self, path):
        # macOS creates AppleDouble files on assigned exFAT scratch. Only remove
        # a verified metadata sidecar of our own fixture/copy, never DLL inputs.
        if hasattr(os, "listxattr"):
            for attribute in os.listxattr(path):
                os.removexattr(path, attribute)
        metadata = path.with_name("._" + path.name)
        if metadata.exists():
            self.assertEqual(metadata.read_bytes()[:4], b"\x00\x05\x16\x07")
            metadata.unlink()

    def test_same_driver_closure_deploys_to_each_actual_consumer_for_both_cpus(self):
        for machine in [0x8664, 0xAA64]:
            with self.subTest(machine=machine):
                for name in [
                    "OPENGL32.dll",
                    "libgallium_wgl.dll",
                    "actual-transitive.dll",
                ]:
                    self.image(name, machine)
                (self.driver / "not-a-runtime.txt").write_text("never deployed")
                destinations = [
                    self.root / f"{machine}/{name}"
                    for name in ["compiler", "diagnostics", "native-exes"]
                ]
                records = self.tool.deploy_driver(self.driver, machine, destinations)
                self.assertEqual(
                    {r["name"].casefold() for r in records},
                    {"opengl32.dll", "libgallium_wgl.dll", "actual-transitive.dll"},
                )
                for directory in destinations:
                    for record in records:
                        self.remove_fixture_metadata(directory / record["name"])
                    self.assertEqual(
                        {p.name for p in directory.iterdir()},
                        {r["name"] for r in records},
                    )
                    for record in records:
                        self.assertEqual(
                            hashlib.sha256(
                                (directory / record["name"]).read_bytes()
                            ).hexdigest(),
                            record["sha256"],
                        )

    def test_wrong_cpu_rejected_before_any_destination_changes(self):
        self.image("opengl32.dll", 0x8664)
        self.image("libgallium_wgl.dll", 0xAA64)
        destination = self.root / "native-exes"
        with self.assertRaisesRegex(ValueError, "architecture mismatch"):
            self.tool.deploy_driver(self.driver, 0x8664, [destination])
        self.assertFalse(destination.exists())

    def test_missing_frontend_or_backend_rejected(self):
        for name in ["opengl32.dll", "libgallium_wgl.dll"]:
            with self.subTest(name=name):
                image = self.image(name, 0x8664)
                with self.assertRaisesRegex(ValueError, "required Mesa"):
                    self.tool.deploy_driver(
                        self.driver, 0x8664, [self.root / "native-exes"]
                    )
                image.unlink()

    def test_copy_corruption_is_a_failure(self):
        self.image("opengl32.dll", 0x8664)
        self.image("libgallium_wgl.dll", 0x8664)
        with patch.object(
            self.tool.shutil,
            "copy2",
            side_effect=lambda source, dest: dest.write_bytes(b"damaged"),
        ):
            with self.assertRaisesRegex(ValueError, "integrity"):
                self.tool.deploy_driver(
                    self.driver, 0x8664, [self.root / "native-exes"]
                )

    def test_source_directory_is_valid_destination_without_self_copy(self):
        self.image("opengl32.dll", 0x8664)
        self.image("libgallium_wgl.dll", 0x8664)
        with patch.object(
            self.tool.shutil, "copy2", side_effect=AssertionError("self copy")
        ):
            self.assertEqual(
                len(self.tool.deploy_driver(self.driver, 0x8664, [self.driver])), 2
            )

    def test_invalid_extra_dll_is_rejected_not_ignored(self):
        self.image("opengl32.dll", 0x8664)
        self.image("libgallium_wgl.dll", 0x8664)
        invalid = self.driver / "invalid.dll"
        invalid.write_bytes(b"not a driver")
        self.remove_fixture_metadata(invalid)
        with self.assertRaisesRegex(ValueError, "not a native Windows PE image"):
            self.tool.deploy_driver(self.driver, 0x8664, [self.root / "native-exes"])

    def test_real_lint_failure_status_still_fails_the_runner(self):
        # Mocked subprocess routing only; never a compiler/lint/native pass.
        image = self.image("opengl32.dll", 0x8664)
        self.image("libgallium_wgl.dll", 0x8664)
        target = self.root / "target"
        compiler, linter = target / "debug/jaic.exe", target / "debug/jailint.exe"
        compiler.parent.mkdir(parents=True)
        for path in [compiler, linter]:
            path.write_bytes(image.read_bytes())
        library, native = self.root / "stdlib", self.root / "native-libs"
        library.mkdir()
        native.mkdir()
        output, consumer = target / "wgl", self.root / "native-exes"

        def result(command, **kwargs):
            if command[0] == str(linter):
                self.assertIn("-D", command)
                self.assertIn("warnings", command)
                return SimpleNamespace(
                    returncode=1, stdout="", stderr="warning: genuine owned finding\n"
                )
            if "-exe" in command:
                return SimpleNamespace(returncode=0, stdout="", stderr="")
            if any("windows-wgl-driver" in item for item in command):
                directory = compiler.parent if "-interp" in command else output
                stdout = (
                    f"GL_RENDERER=llvmpipe\nopengl32={directory / 'opengl32.dll'}\nok\n"
                )
            else:
                stdout = "ok\n"
            return SimpleNamespace(returncode=0, stdout=stdout, stderr="")

        argv = [
            "wgl",
            "--jaic",
            str(compiler),
            "--jailint",
            str(linter),
            "--stdlib",
            str(library),
            "--native-libs",
            str(native),
            "--driver-dir",
            str(self.driver),
            "--out",
            str(output),
            "--consumer-dir",
            str(consumer),
        ]
        with (
            patch.object(self.tool, "ROOT", self.root),
            patch.object(sys, "argv", argv),
            patch.object(self.tool.platform, "system", return_value="Windows"),
            patch.object(self.tool.subprocess, "run", side_effect=result),
            patch.dict(
                os.environ,
                {"MESA_GL_VERSION_OVERRIDE": "", "MESA_GLSL_VERSION_OVERRIDE": ""},
            ),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            with self.assertRaisesRegex(
                SystemExit, "failed; retain this failure evidence"
            ):
                self.tool.main()
        report = json.loads((output / "results.json").read_text())
        self.assertFalse(report["passed"])
        self.assertEqual(report["records"][-1]["case"], "owned-WGL-lint")
        self.assertEqual(report["records"][-1]["exit"], 1)
        self.assertIn("genuine owned finding", report["records"][-1]["stderr"])
        self.assertTrue(all(record["passed"] for record in report["records"][:-1]))
        self.assertEqual((consumer / "opengl32.dll").read_bytes(), image.read_bytes())

    def test_owned_lint_inputs_are_explicit_entries_not_recursive_test_directories(
        self,
    ):
        cases = [
            "windows-wgl-driver",
            "windows-render-events",
            "windows-render-public",
            "windows-render-lifecycle",
        ]
        library = self.root / "stdlib"
        paths = self.tool.wgl_lint_paths(library, cases)
        for name in [
            "Window_Type",
            "Window_Creation",
            "Input",
            "Keymap",
            "Gamepad",
            "Gamepad/SDL_Input_Map",
            "Simp",
            "Srgb",
            "Clipboard",
        ]:
            self.assertIn(library / name / "module.jai", paths)
            self.assertNotIn(library / name, paths)
        self.assertIn(library / "internal/input/windows-devices/module.jai", paths)
        self.assertTrue(all(path.suffix == ".jai" for path in paths))
        self.assertIn(library / "internal/window/lifecycle.jai", paths)
        self.assertIn(library / "internal/rendering/window-contexts.jai", paths)
        for name in [
            "Input/tests/windows-device-decoding.jai",
            "Input/tests/signed-native-fields.jai",
            "Simp/tests/glyph-atlas.jai",
            "Simp/tests/texture-packing.jai",
        ]:
            self.assertIn(library / name, paths)
        self.assertEqual(
            {path.stem for path in paths if path.parent.name == "window-render"},
            set(cases),
        )

    def test_both_workflow_jobs_deploy_to_wheel_consumer_without_changing_checks(self):
        for job in ["native", "native-arm64"]:
            body = re.search(
                r"^  " + job + r":\n(.*?)(?=^  [\w-]+:|\Z)",
                self.workflow,
                re.MULTILINE | re.DOTALL,
            ).group(1)
            command = next(
                line
                for line in body.splitlines()
                if "python -u tools/window_render_wgl_ci.py" in line
            )
            self.assertIn("--consumer-dir native-exes", command)
            self.assertLess(
                body.index(command),
                body.index("Public renderer wheel propagation native O0"),
            )
            for opt in ["O0", "O2"]:
                self.assertIn(
                    f"tests/window-render/windows-render-events.jai -{opt} -exe native-exes/render-wheel-{opt}",
                    body,
                )
                self.assertIn(f"native-exes/render-wheel-{opt}.exe", body)
            self.assertIn("signed-wheel-windows.jai", body)


if __name__ == "__main__":
    unittest.main()
