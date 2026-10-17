"""Original Meson metadata/installer regressions, not native graphics execution."""
from pathlib import Path
import json
import struct
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import window_render_mesa_windows as mesa
import window_render_mesa_install as install
from window_render_wgl_ci import probe_loaded_expected_driver


class MesaInstall(unittest.TestCase):
    def test_windows_wgl_configuration_enabled(self):
        self.assertIn("-Dplatforms=windows", mesa.MESA_OPTIONS)
        self.assertIn("-Dgallium-wgl-dll-name=libgallium_wgl", mesa.MESA_OPTIONS)

    def test_introspection_rejects_disabled_platform_and_changed_dll_name(self):
        values = {"platforms": ["windows"], "opengl": True, "gallium-drivers": ["llvmpipe"],
                  "llvm": "enabled", "shared-llvm": "disabled", "b_vscrt": "mt",
                  "gallium-wgl-dll-name": "libgallium_wgl"}
        install.validate_wgl_configuration([{"name": name, "value": value} for name, value in values.items()])
        for changed in [dict(values, platforms=[]), dict(values, **{"gallium-wgl-dll-name": "unrelated"})]:
            with self.assertRaisesRegex(ValueError, "configuration mismatch"):
                install.validate_wgl_configuration([{"name": name, "value": value} for name, value in changed.items()])

    def fixture(self, directory):
        source, build, prefix = [directory / name for name in ["source", "build", "install"]]
        targets, installed = [], {}
        for name, subdir in [("opengl32", "libgl-gdi"), ("libgallium_wgl", "wgl")]:
            defined = source / f"src/gallium/targets/{subdir}/meson.build"
            defined.parent.mkdir(parents=True)
            defined.write_text("# original metadata fixture; no Mesa implementation\n")
            output = build / f"src/gallium/targets/{subdir}/{name}.dll"
            output.parent.mkdir(parents=True)
            output.write_bytes(b"installer-copy-integrity fixture, never executed")
            destination = prefix / f"custom bin/{name.upper()}.DLL"
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(output.read_bytes())
            targets.append({"name": name, "type": "shared library", "installed": True,
                            "defined_in": str(defined), "filename": [str(output)]})
            installed[str(output)] = str(destination)
        return source, build, prefix, targets, installed

    def test_installed_mapping_handles_directory_and_case(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            args = self.fixture(Path(temporary))
            found = install.discover_wgl_install(*args)
            self.assertEqual(set(found), {"opengl32.dll", "libgallium_wgl.dll"})
            self.assertTrue(all(path.parent.name == "custom bin" for path in found.values()))

    def test_no_unrelated_target_or_changed_install_is_accepted(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            source, build, prefix, targets, installed = self.fixture(Path(temporary))
            targets[0]["defined_in"] = str(source / "unrelated/meson.build")
            with self.assertRaisesRegex(ValueError, "WGL target"):
                install.discover_wgl_install(source, build, prefix, targets, installed)
            targets[0]["defined_in"] = str(source / "src/gallium/targets/libgl-gdi/meson.build")
            Path(next(iter(installed.values()))).write_bytes(b"stale install")
            with self.assertRaisesRegex(ValueError, "integrity"):
                install.discover_wgl_install(source, build, prefix, targets, installed)

    def test_incomplete_or_escaping_install_fails(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            source, build, prefix, targets, installed = self.fixture(Path(temporary))
            output = targets[0]["filename"][0]
            destination = installed.pop(output)
            with self.assertRaisesRegex(ValueError, "install mapping"):
                install.discover_wgl_install(source, build, prefix, targets, installed)
            installed[output] = str(prefix.parent / "outside.dll")
            with self.assertRaisesRegex(ValueError, "outside"):
                install.discover_wgl_install(source, build, prefix, targets, installed)
            installed[output] = destination
            Path(destination).unlink()
            with self.assertRaisesRegex(ValueError, "installed WGL"):
                install.discover_wgl_install(source, build, prefix, targets, installed)

    def test_sdk_dependency_and_export_metadata(self):
        # Independently authored dumpbin-format records; not executable images.
        text = """    libgallium_wgl.dll
    KERNEL32.dll
    api-ms-win-core-synch-l1-2-0.dll
    1 0 00001000 wglCreateContext
    2 A 00002000 glReadPixels
"""
        dependencies, exports = install.parse_dumpbin(text)
        self.assertEqual(dependencies, {"libgallium_wgl.dll", "kernel32.dll", "api-ms-win-core-synch-l1-2-0.dll"})
        self.assertEqual(exports, {"wglCreateContext", "glReadPixels"})

    def test_driver_probe_must_identify_deployed_file(self):
        # File-identity regression only; these files are not executable drivers.
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            expected = Path(temporary) / "opengl32.dll"
            other = Path(temporary) / "other.dll"
            expected.write_bytes(b"file identity fixture")
            other.write_bytes(expected.read_bytes())
            self.assertTrue(probe_loaded_expected_driver(f"opengl32={expected}\n", expected))
            self.assertFalse(probe_loaded_expected_driver(f"opengl32={other}\n", expected))
            self.assertTrue(probe_loaded_expected_driver(f"opengl32={other}\n", expected, [other]))
            self.assertFalse(probe_loaded_expected_driver("GL_RENDERER=llvmpipe\n", expected))
            self.assertFalse(probe_loaded_expected_driver(f"opengl32={expected}\nopengl32={other}\n", expected))

    def test_x64_header_is_rejected_before_native_sdk_execution(self):
        # A COFF-header decoding input, deliberately not an executable DLL and
        # never accepted as graphics support. No native SDK/program is mocked.
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            prefix = Path(temporary) / "install"
            prefix.mkdir()
            evidence = Path(temporary) / "evidence"
            evidence.mkdir()
            header = bytearray(128)
            header[:2] = b"MZ"
            struct.pack_into("<I", header, 60, 64)
            header[64:68] = b"PE\0\0"
            struct.pack_into("<H", header, 68, 0x8664)
            (prefix / "architecture-fixture.dll").write_bytes(header)
            with self.assertRaisesRegex(ValueError, "non-ARM64"):
                install.validate_native_install(prefix, {}, Path("SDK-must-not-execute"), evidence)
            actual = json.loads((evidence / "validation.json").read_text())
            self.assertFalse(actual["validated"])
            self.assertFalse(actual["runtime_verified"])
            self.assertEqual(actual["images"]["architecture-fixture.dll"]["machine"], 0x8664)


if __name__ == "__main__":
    unittest.main()
