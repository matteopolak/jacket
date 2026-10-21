import tempfile
import unittest
from pathlib import Path
from unittest import mock

import package_bindings_helpers as helpers


class Packaging(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)

    def test_static_archive_is_named_for_the_platform_and_drops_its_object(self):
        out = self.root / "out"
        out.mkdir()

        def fake(argv, check):
            if "-c" in argv:
                Path(argv[argv.index("-o") + 1]).write_text("")
            else:
                Path(argv[-2]).write_text("archive")

        with mock.patch.object(helpers.subprocess, "run", side_effect=fake):
            linux = helpers.build_static(self.root, "linux-x64", out)
            windows = helpers.build_static(self.root, "windows-x64", out)
        self.assertEqual(linux.name, "libgenerator_platform.a")
        self.assertEqual(windows.name, "generator_platform.lib")
        self.assertEqual(sorted(p.name for p in out.iterdir()), sorted([linux.name, windows.name]))

    def test_linux_libclang_brings_the_llvm_libraries_it_names_and_not_the_systems(self):
        lib = self.root / "lib"
        lib.mkdir()
        (lib / "libclang.so.23").write_text("clang")
        (lib / "libclang.so").symlink_to("libclang.so.23")
        (lib / "libLLVM.so.23.1").write_text("llvm")
        out = self.root / "out"
        out.mkdir()
        names = {"libclang.so.23": ["libLLVM.so.23.1", "libc.so.6"], "libLLVM.so.23.1": ["libc.so.6"]}
        with (
            mock.patch.object(helpers, "needed", side_effect=lambda image: names[Path(image).resolve().name] if Path(image).resolve().name in names else names[Path(image).name]),
            mock.patch.object(helpers.subprocess, "run") as run,
        ):
            copied = helpers.copy_linux_libclang(self.root, out)
        self.assertEqual(sorted(p.name for p in copied), ["libLLVM.so.23.1", "libclang.so"])
        self.assertEqual(run.call_count, 2)  # an $ORIGIN run path on each image
        self.assertEqual((out / "libclang.so").read_text(), "clang")


if __name__ == "__main__":
    unittest.main()
