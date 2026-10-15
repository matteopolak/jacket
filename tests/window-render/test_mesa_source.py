"""Original source-cache integrity regressions; no driver execution is simulated."""
import hashlib
import io
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import window_render_mesa_windows as mesa


class SourceIntegrity(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name)

    def archive(self, member):
        archive = self.path / "fixture.archive"
        with tarfile.open(archive, "w") as output:
            entry = tarfile.TarInfo(member)
            entry.size = 4
            output.addfile(entry, io.BytesIO(b"data"))
        return hashlib.sha256(archive.read_bytes()).hexdigest()

    def test_digest_failure_does_not_extract(self):
        self.archive("source/file.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", "0" * 64, "source")):
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                mesa.source_tree("fixture", self.path)
        self.assertFalse((self.path / "source").exists())

    def test_verified_source_rejects_traversal(self):
        digest = self.archive("../escaped.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", digest, "source")):
            with self.assertRaises(tarfile.OutsideDestinationError):
                mesa.source_tree("fixture", self.path)
        self.assertFalse((self.path.parent / "escaped.c").exists())

    def test_modified_source_is_not_reused(self):
        digest = self.archive("source/file.c")
        with patch.dict(mesa.SOURCES, fixture=("https://invalid.example", digest, "source")):
            tree = mesa.source_tree("fixture", self.path)
            (tree / "file.c").write_bytes(b"modified")
            with self.assertRaisesRegex(ValueError, "stale/modified"):
                mesa.source_tree("fixture", self.path)

    def cache(self):
        configuration = {"source": "pinned", "sdk": "20348", "crt": "MT"}
        entry = self.path / mesa.build_cache_key(configuration)
        tree = entry / "llvm"
        tree.mkdir(parents=True)
        (tree / "library.lib").write_bytes(b"native library integrity fixture")
        mesa.seal_llvm_cache(entry, configuration)
        return entry, configuration

    def test_completed_cache_requires_exact_configuration(self):
        entry, configuration = self.cache()
        self.assertTrue(mesa.verify_llvm_cache(entry, configuration))
        with self.assertRaisesRegex(ValueError, "configuration"):
            mesa.verify_llvm_cache(entry, dict(configuration, crt="MD"))
        self.assertNotEqual(mesa.build_cache_key(configuration),
                            mesa.build_cache_key(dict(configuration, sdk="changed")))

    def test_cache_rejects_modified_missing_and_extra_files(self):
        entry, configuration = self.cache()
        library = entry / "llvm/library.lib"
        original = library.read_bytes()
        library.write_bytes(b"corruption")
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)
        library.unlink()
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)
        library.write_bytes(original)
        (entry / "llvm/unrecorded.dll").write_bytes(b"unexpected")
        with self.assertRaisesRegex(ValueError, "integrity"):
            mesa.verify_llvm_cache(entry, configuration)

    def test_partial_cache_does_not_count_as_complete(self):
        entry = self.path / "partial"
        (entry / "llvm").mkdir(parents=True)
        (entry / "llvm/unfinished.lib").write_bytes(b"partial")
        self.assertFalse(mesa.verify_llvm_cache(entry, {"source": "pinned"}))

    def native_environment(self):
        # SDK layout/content-integrity fixtures only, not a runnable toolchain.
        vs = self.path / "Program Files/Microsoft Visual Studio/18/Enterprise"
        vc = vs / "VC/Tools/MSVC/14.50"
        sdk = self.path / "Windows Kits/10"
        version = "10.0.26100.0"
        includes = [vc / "include", *[sdk / f"Include/{version}/{name}" for name in ["um", "shared", "ucrt"]]]
        libs = [vc / "lib/arm64", *[sdk / f"Lib/{version}/{name}/arm64" for name in ["um", "ucrt"]]]
        for directory, names in zip(includes + libs, [
                ["vcruntime.h", "vector"], ["Windows.h"], ["SDKDDKVer.h"], ["stdio.h"],
                ["libcmt.lib", "libcpmt.lib", "libvcruntime.lib"],
                ["kernel32.lib", "user32.lib", "gdi32.lib", "ws2_32.lib"], ["libucrt.lib"]]):
            directory.mkdir(parents=True)
            for name in names:
                (directory / name).write_bytes(b"native SDK integrity fixture")
        (vc / "bin").mkdir()
        tool = vc / "bin/tool.exe"
        tool.write_bytes(b"tool identity fixture")
        env = {"WINDOWSSDKDIR": str(sdk), "VCTOOLSINSTALLDIR": str(vc),
               "VSINSTALLDIR": str(vs), "VSCMD_ARG_TGT_ARCH": "arm64",
               "INCLUDE": os.pathsep.join(map(str, includes)), "LIB": os.pathsep.join(map(str, libs)),
               "WINDOWSSDKVERSION": version + "\\", "VCTOOLSVERSION": "14.50"}
        return env, tool, includes[0] / "vector", libs[0] / "libcmt.lib"

    def test_native_cache_key_tracks_sdk_tools_and_flags(self):
        env, tool, header, library = self.native_environment()
        with patch.object(mesa.shutil, "which", return_value=str(tool)):
            baseline = mesa.llvm_cache_configuration(env)
            key = mesa.build_cache_key(baseline)
            with patch.object(mesa, "MESA_OPTIONS", ["-Dplatforms=changed-for-driver-only"]):
                self.assertEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(env)))
            self.assertEqual(baseline["toolchain"]["VCToolsVersion"], "14.50")
            self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(dict(env, CL="/Od"))))
            with self.assertRaisesRegex(ValueError, "required"):
                mesa.llvm_cache_configuration(dict(env, WINDOWSSDKVERSION="changed"))
            for path in [header, library, tool]:
                original = path.read_bytes()
                path.write_bytes(original + b"changed")
                self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(env)))
                path.write_bytes(original)
            (tool.parent / "frontend.dll").write_bytes(b"changed compiler backend")
            self.assertNotEqual(key, mesa.build_cache_key(mesa.llvm_cache_configuration(env)))

    def test_optional_vs2026_search_entries_and_later_creation(self):
        env, tool, _, _ = self.native_environment()
        optional = {name: Path(env["VSINSTALLDIR"]) / f"VC/Auxiliary/VS/{subdir}"
                    for name, subdir in [("INCLUDE", "include"), ("LIB", "lib/arm64")]}
        for name, path in optional.items():
            env[name] = str(path) + os.pathsep + env[name]
        with patch.object(mesa.shutil, "which", return_value=str(tool)):
            baseline = mesa.llvm_cache_configuration(env)
            for name in optional:
                self.assertEqual(baseline["sdk_inputs"][name][0]["state"], "missing")
            self.assertEqual(baseline, mesa.llvm_cache_configuration(env))
            key = mesa.build_cache_key(baseline)
            for name, path in optional.items():
                path.mkdir(parents=True)
                created = mesa.llvm_cache_configuration(env)
                self.assertEqual(created["sdk_inputs"][name][0]["state"], "directory")
                self.assertNotEqual(key, mesa.build_cache_key(created))
                (path / "optional-native-input").write_bytes(b"new SDK input")
                self.assertNotEqual(mesa.build_cache_key(created),
                                    mesa.build_cache_key(mesa.llvm_cache_configuration(env)))

    def test_missing_required_headers_libraries_search_and_compiler_fail(self):
        env, tool, header, library = self.native_environment()
        with patch.object(mesa.shutil, "which", return_value=str(tool)):
            for required in [header, library, Path(env["WINDOWSSDKDIR"]) / "Lib/10.0.26100.0/um/arm64/kernel32.lib"]:
                original = required.read_bytes()
                required.unlink()
                with self.assertRaisesRegex(ValueError, "required"):
                    mesa.llvm_cache_configuration(env)
                required.write_bytes(original)
            with self.assertRaisesRegex(ValueError, "required"):
                mesa.llvm_cache_configuration(dict(env, LIB=str(self.path / "missing-only")))
            architecture = Path(env["WINDOWSSDKDIR"]) / "Lib/10.0.26100.0/um/arm64"
            architecture.rename(architecture.with_name("saved-arm64"))
            with self.assertRaisesRegex(ValueError, "required"):
                mesa.llvm_cache_configuration(env)
            architecture.with_name("saved-arm64").rename(architecture)
            with self.assertRaisesRegex(ValueError, "required"):
                mesa.llvm_cache_configuration(dict(env, VSCMD_ARG_TGT_ARCH="x64"))
            tool.unlink()
            with self.assertRaisesRegex(ValueError, "missing native build tool"):
                mesa.llvm_cache_configuration(env)
        with patch.object(mesa.shutil, "which", return_value=None):
            with self.assertRaisesRegex(ValueError, "missing native build tool"):
                mesa.llvm_cache_configuration(env)

    def test_optional_real_paths_are_hashed_and_wrong_library_shadows_fail(self):
        env, tool, _, _ = self.native_environment()
        optional = Path(env["VSINSTALLDIR"]) / "VC/Auxiliary/VS/lib/arm64"
        optional.mkdir(parents=True)
        env["LIB"] = str(optional) + os.pathsep + env["LIB"]
        with patch.object(mesa.shutil, "which", return_value=str(tool)):
            baseline = mesa.llvm_cache_configuration(env)
            (optional / "kernel32.lib").write_bytes(b"unexpected library shadow")
            with self.assertRaisesRegex(ValueError, "shadowed"):
                mesa.llvm_cache_configuration(env)
            (optional / "kernel32.lib").unlink()
            (optional / "auxiliary.lib").write_bytes(b"optional library")
            self.assertNotEqual(mesa.build_cache_key(baseline),
                                mesa.build_cache_key(mesa.llvm_cache_configuration(env)))
            # Existing unknown directories are still rejected, not silently
            # omitted from the native SDK content fingerprint.
            unknown = self.path / "unselected"
            unknown.mkdir()
            with self.assertRaisesRegex(ValueError, "selected native SDK directory"):
                mesa.llvm_cache_configuration(dict(env, INCLUDE=str(unknown) + os.pathsep + env["INCLUDE"]))


if __name__ == "__main__":
    unittest.main()
