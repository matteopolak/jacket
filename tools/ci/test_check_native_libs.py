import tempfile
import unittest
from pathlib import Path

import check_native_libs as cnl
from native import provided_libs
from toolkit.repo import ROOT

MANIFEST = {
    "libraries": {"stb_image": {}},
    "provided": {
        "embedded": {"portable_watch": "in jaic"},
        "fetched": {"wgpu_native": {"script": "fetch.py"}},
        "runtime": {},
        "packaged": {},
        "external": {"System": ["c", "kernel32"]},
    },
}


class Check(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.stdlib = Path(scratch.name)

    def bind(self, name, text):
        path = self.stdlib / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def problems(self, manifest=MANIFEST):
        return cnl.check(self.stdlib, "linux-x64", None, manifest)

    def test_every_kind_of_declaration_is_accounted_for(self):
        self.bind("A/module.jai", 'libc :: #library,system "c";\nimg :: #library,system "stb_image";\n'
                  'k :: #system_library "kernel32";\nw :: #library "native/portable_watch";\n'
                  'g :: #library,system "libwgpu_native";\n')
        self.assertEqual(self.problems(), [])

    def test_a_system_library_directive_is_scanned_too(self):
        self.bind("A/module.jai", 'x :: #system_library "unlisted";\n')
        found = [p for p in self.problems() if "`unlisted`" in p]
        self.assertEqual(len(found), 1)

    def test_an_undeclared_name_fails(self):
        self.bind("A/module.jai", 'x :: #library,system "mystery";\n')
        found = [p for p in self.problems() if "`mystery`" in p]
        self.assertIn("neither built by `libraries` nor declared in", found[0])

    def test_tests_are_not_scanned(self):
        self.bind("A/tests/probe.jai", 'x :: #library,system "mystery";\n')
        self.assertEqual([p for p in self.problems() if "mystery" in p], [])

    def test_a_declaration_nothing_binds_fails(self):
        problems = self.problems()
        self.assertTrue(any("`kernel32`" in p and "does not bind" in p for p in problems))

    def test_built_libraries_must_be_in_the_archive(self):
        self.bind("A/module.jai", 'img :: #library,system "stb_image";\n')
        native = self.stdlib / "native"
        native.mkdir()
        problems = cnl.check(self.stdlib, "linux-x64", native, MANIFEST)
        self.assertTrue(any("libstb_image.a" in p for p in problems))
        (native / "libstb_image.a").write_text("")
        problems = cnl.check(self.stdlib, "linux-x64", native, MANIFEST)
        self.assertFalse(any("libstb_image.a" in p for p in problems))


class Repository(unittest.TestCase):
    def test_the_stdlib_binds_only_declared_libraries_on_every_platform(self):
        for platform in ("macos-arm64", "macos-x64", "linux-x64", "linux-arm64", "windows-x64", "windows-arm64"):
            with self.subTest(platform=platform):
                self.assertEqual(cnl.check(ROOT / "stdlib", platform, None), [])

    def test_a_library_is_declared_once(self):
        declared = provided_libs.providers(provided_libs.load_manifest())
        manifest = provided_libs.load_manifest()
        self.assertEqual([name for name in manifest["libraries"] if name in declared], [])


if __name__ == "__main__":
    unittest.main()
