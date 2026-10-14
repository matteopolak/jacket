import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import sdl3_native as sdl
import build_native_libs as pipeline

class BuildContract(unittest.TestCase):
    def test_six_platforms_are_explicit(self):
        self.assertEqual(len(sdl.PLATFORMS),6)
        for p in sdl.PLATFORMS:
            args=sdl.configure_options(p)
            for opt in ('AUDIO','VIDEO','EVENTS','JOYSTICK','RENDER','GPU'):
                self.assertIn(f'-DSDL_{opt}=ON',args)
            self.assertIn('-DSDL_STATIC=ON',args)
            self.assertIn('-DSDL_SHARED=ON',args)
    def test_architectures_are_not_build_host_defaults(self):
        self.assertIn('-DCMAKE_OSX_ARCHITECTURES=x86_64',sdl.configure_options('macos-x64'))
        self.assertIn('ARM64',sdl.configure_options('windows-arm64'))
        with self.assertRaises(ValueError):sdl.configure_options('linux-riscv64')
    def test_unbound_static_dependencies_fail(self):
        sdl.check_dependencies('$<LINK_ONLY:m>;$<LINK_ONLY:$<LINK_LIBRARY:WEAK_FRAMEWORK,CoreHaptics>>;$<TARGET_NAME:SDL3::Headers>;\n-lpthread','macos-arm64')
        with self.assertRaisesRegex(RuntimeError,'unbound transitive'):
            sdl.check_dependencies('$<LINK_ONLY:PkgConfig::UNEXPECTED>','linux-x64')
    def test_bad_cached_archive_is_rejected_offline(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            p=Path(d);(p/'SDL3-3.4.18.tar.gz').write_bytes(b'corrupt')
            with self.assertRaisesRegex(RuntimeError,'cache hash mismatch'):sdl.checked_archive(p,True)
    def test_offline_missing_archive_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            with self.assertRaisesRegex(RuntimeError,'offline: missing'):sdl.checked_archive(Path(d),True)
    def test_package_inventory_includes_import_license_and_metadata(self):
        manifest=json.loads(pipeline.MANIFEST.read_text())
        for p in sdl.PLATFORMS:
            files=pipeline.shipped_files(manifest,p)
            self.assertIn('licenses/sdl3/LICENSE.txt',files)
            self.assertIn('sdl3/build.json',files)
            if p.startswith('windows'):self.assertIn('SDL3-import.lib',files)

if __name__=='__main__':unittest.main()
