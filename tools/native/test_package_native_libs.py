import unittest
from pathlib import Path

import package_native_libs as pkg


def scripts(platform):
    package = Path("/p/jacket-" + platform)
    return [command[1].rsplit("/", 1)[-1] for command in pkg.commands(platform, package, Path("/llvm"))]


class Commands(unittest.TestCase):
    def test_windows_builds_the_crash_runtime_first_and_checks_last(self):
        self.assertEqual(scripts("windows-x64"), [
            "build_platform_io_runtime.py", "fetch_wgpu_native.py", "build_native_libs.py",
            "package_bindings_helpers.py", "check_native_libs.py"])

    def test_other_platforms_have_no_runtime_step(self):
        for platform in ("macos-arm64", "macos-x64", "linux-x64", "linux-arm64"):
            self.assertEqual(scripts(platform), [
                "fetch_wgpu_native.py", "build_native_libs.py", "package_bindings_helpers.py", "check_native_libs.py"])

    def test_placeholders_are_filled(self):
        commands = pkg.commands("windows-arm64", Path("/p/jacket-windows-arm64"), Path("/llvm"))
        runtime = commands[0]
        self.assertEqual(runtime[2:], ["--stdlib", "/p/jacket-windows-arm64/stdlib", "--cpu", "arm64", "--static-library"])
        helpers = next(command for command in commands if command[1].endswith("package_bindings_helpers.py"))
        self.assertEqual(helpers[2:], ["--platform", "windows-arm64", "--llvm-prefix", "/llvm",
                                       "--out", "/p/jacket-windows-arm64/native-libs/windows-arm64"])
        self.assertTrue(all("{" not in part for command in commands for part in command))

    def test_declared_scripts_exist(self):
        for command in pkg.commands("linux-x64", Path("/p/jacket-linux-x64"), Path("/llvm")):
            self.assertTrue((pkg.ROOT / command[1]).is_file(), command[1])


if __name__ == "__main__":
    unittest.main()
