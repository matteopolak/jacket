"""Package/supervisor controls only; no Windows DLL is executed."""

import os
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import build_native_libs as builder
import stdlib_runtime as runtime


class DynamicPackage(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_matching_import_archive_survives_the_linker_temporary_directory(self):
        obj = self.root / "unit.obj"
        definitions = self.root / "unit.def"
        with (
            patch.object(builder, "compile_units", return_value=[obj]),
            patch.object(builder, "windows_exports", return_value=definitions),
            patch.object(builder.subprocess, "run") as run,
        ):
            builder.build("unit", {}, self.root, self.root, "windows-x64")
        commands = [call.args[0] for call in run.call_args_list]
        self.assertIn("/out:" + str(self.root / "unit.lib"), commands[0])
        self.assertIn(
            "-Wl,/IMPLIB:" + str(self.root / "dynamic-imports/unit.lib"),
            commands[1],
        )
        self.assertTrue((self.root / "dynamic-imports").is_dir())

    def test_package_inventory_requires_both_static_and_dynamic_identities(self):
        manifest = {
            "libraries": {"unit": {"source": "unit"}},
            "sources": {"unit": {}},
        }
        self.assertCountEqual(
            builder.shipped_files(manifest, "windows-x64"),
            ["unit.lib", "unit.dll", "dynamic-imports/unit.lib"],
        )
        self.assertCountEqual(
            builder.shipped_files(manifest, "linux-x64"),
            ["libunit.a", "libunit.so"],
        )

    def test_dll_deployment_retains_bytes_and_rejects_conflicting_providers(self):
        first, second, output = [self.root / name for name in ("a", "b", "out")]
        for directory in (first, second, output):
            directory.mkdir()
        (first / "unit.dll").write_bytes(b"independent package identity A")
        (second / "unit.dll").write_bytes(b"independent package identity A")
        runtime.deploy_windows_dlls([first, second], output)
        self.assertEqual(
            (output / "unit.dll").read_bytes(), (first / "unit.dll").read_bytes()
        )
        (second / "unit.dll").write_bytes(b"independent conflicting identity B")
        with self.assertRaisesRegex(ValueError, "conflicting Windows DLL"):
            runtime.deploy_windows_dlls([first, second], output)
        self.assertEqual(
            (output / "unit.dll").read_bytes(), (first / "unit.dll").read_bytes()
        )

    def test_static_only_cache_is_incomplete_and_real_import_folder_has_priority(self):
        manifest = self.root / "manifest.json"
        manifest.write_text(
            json.dumps(
                {
                    "libraries": {"unit": {"source": "unit"}},
                    "sources": {"unit": {}},
                }
            )
        )
        (self.root / "unit.lib").write_bytes(b"independent static cache control")
        (self.root / "unit.dll").write_bytes(b"independent DLL identity control")
        with (
            patch.object(builder, "MANIFEST", manifest),
            patch.object(builder, "output_dir", return_value=self.root),
        ):
            self.assertEqual(builder.missing("windows-x64"), ["unit"])
            imports = self.root / "dynamic-imports"
            imports.mkdir()
            (imports / "unit.lib").write_bytes(b"independent import cache control")
            self.assertEqual(builder.missing("windows-x64"), [])
        self.assertEqual(
            runtime.windows_library_dirs([self.root]), [imports, self.root]
        )

    def test_existing_app_local_provider_is_never_overwritten(self):
        source, output = self.root / "source", self.root / "out"
        source.mkdir()
        output.mkdir()
        (source / "opengl32.dll").write_bytes(b"different package provider")
        (output / "opengl32.dll").write_bytes(b"caller-owned Mesa identity")
        with self.assertRaisesRegex(ValueError, "conflicting Windows DLL destination"):
            runtime.deploy_windows_dlls([source], output)
        self.assertEqual(
            (output / "opengl32.dll").read_bytes(), b"caller-owned Mesa identity"
        )


if __name__ == "__main__":
    unittest.main()
