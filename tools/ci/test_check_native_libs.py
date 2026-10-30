import tempfile
import unittest
from pathlib import Path

import check_native_libs as cnl
from native import provided_libs
from toolkit.repo import ROOT

MANIFEST = {"libraries": {"stb_image": {}}}


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
        self.bind("A/module.jai", 'libc :: #library,system "c"; @native(external)\nimg :: #library,system "stb_image";\n'
                  'k :: #system_library "kernel32"; @native(external)\nw :: #library "native/portable_watch"; @native(embedded)\n'
                  'g :: #library,system "libwgpu_native"; @native(fetched)\n'
                  'r :: #library "crash-native/platform_io_windows_crash"; @native(runtime,windows)\n'
                  'p :: #library "generator_platform"; @native(packaged)\n'
                  'd :: #library "check-native/developer_tools_check"; @native(developer)\n')
        self.assertEqual(self.problems(), [])

    def test_a_system_library_directive_is_scanned_too(self):
        self.bind("A/module.jai", 'x :: #system_library "unlisted";\n')
        found = [p for p in self.problems() if "`unlisted`" in p]
        self.assertEqual(len(found), 1)

    def test_foreign_library_directives_are_scanned(self):
        self.bind("A/module.jai", 'x :: #foreign_library "unlisted";\ny :: #foreign_system_library "other";\n')
        self.assertEqual(len([p for p in self.problems() if "`unlisted`" in p or "`other`" in p]), 2)

    def test_a_local_folder_does_not_hide_the_name(self):
        self.bind("A/module.jai", 'x :: #library "../internal/developer-tools/clang-bindings/clang";\n')
        self.assertTrue(any("`clang`" in p for p in self.problems()))

    def test_an_undeclared_name_fails(self):
        self.bind("A/module.jai", 'x :: #library,system "mystery";\n')
        found = [p for p in self.problems() if "`mystery`" in p]
        self.assertIn("neither built by `libraries` nor declared by", found[0])

    def test_one_note_per_name_is_enough(self):
        self.bind("A/module.jai", 'x :: #library,system "c"; @native(external)\n')
        self.bind("B/module.jai", 'y :: #library,system "c";\n')
        self.assertEqual(self.problems(), [])

    def test_notes_that_disagree_fail(self):
        self.bind("A/module.jai", 'x :: #library,system "c"; @native(external)\ny :: #library,system "c"; @native(embedded)\n')
        self.assertTrue(any("`c` is declared embedded here" in p for p in self.problems()))

    def test_a_bad_note_fails(self):
        self.bind("A/module.jai", 'x :: #library,system "c"; @native(nowhere)\n')
        self.assertTrue(any("must be one of" in p for p in self.problems()))
        self.bind("A/module.jai", 'x :: #library,system "c"; @native(external,beos)\n')
        self.assertTrue(any("OS names" in p for p in self.problems()))

    def test_a_fetched_library_needs_a_recipe(self):
        self.bind("A/module.jai", 'x :: #library,system "mystery"; @native(fetched)\n')
        self.assertTrue(any("has no recipe" in p for p in self.problems()))

    def test_a_built_library_has_no_note(self):
        self.bind("A/module.jai", 'img :: #library,system "stb_image"; @native(external)\n')
        self.assertTrue(any("built by tools/native/native-libs.json and also declared" in p for p in self.problems()))

    def test_comments_do_not_bind(self):
        self.bind("A/module.jai", '// x :: #library,system "mystery";\n')
        self.assertEqual(self.problems(), [])

    def test_tests_are_not_scanned(self):
        self.bind("A/tests/probe.jai", 'x :: #library,system "mystery";\n')
        self.assertEqual([p for p in self.problems() if "mystery" in p], [])

    def test_built_libraries_must_be_in_the_archive(self):
        self.bind("A/module.jai", 'img :: #library,system "stb_image";\n')
        native = self.stdlib / "native"
        native.mkdir()
        problems = cnl.check(self.stdlib, "linux-x64", native, MANIFEST)
        self.assertTrue(any("libstb_image.a" in p for p in problems))
        (native / "libstb_image.a").write_text("")
        problems = cnl.check(self.stdlib, "linux-x64", native, MANIFEST)
        self.assertFalse(any("libstb_image.a" in p for p in problems))

    def test_a_packaged_library_must_be_in_the_archive_as_static_and_shared(self):
        self.bind("A/module.jai", 'p :: #library "generator_platform"; @native(packaged)\n')
        native = self.stdlib / "native"
        native.mkdir()
        problems = cnl.check(self.stdlib, "linux-x64", native, MANIFEST)
        self.assertTrue(any("libgenerator_platform.a" in p for p in problems))
        self.assertTrue(any("libgenerator_platform.so" in p for p in problems))


class Repository(unittest.TestCase):
    def test_the_stdlib_binds_only_declared_libraries_on_every_platform(self):
        for platform in ("macos-arm64", "macos-x64", "linux-x64", "linux-arm64", "windows-x64", "windows-arm64"):
            with self.subTest(platform=platform):
                self.assertEqual(cnl.check(ROOT / "stdlib", platform, None), [])

    def test_the_notes_parse_and_agree(self):
        self.assertEqual(provided_libs.providers(ROOT / "stdlib")[1], [])

    def test_every_recipe_is_used(self):
        declared = provided_libs.providers(ROOT / "stdlib")[0]
        self.assertEqual(sorted(set(provided_libs.RECIPES) - set(declared)), [])


if __name__ == "__main__":
    unittest.main()
