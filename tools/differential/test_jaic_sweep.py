import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from differential import jaic_sweep


class ExampleCaseTests(unittest.TestCase):
    def test_every_example_case_reads_and_only_run_cases_expect_output(self):
        found = list(jaic_sweep.cases("examples"))
        self.assertTrue(found)
        for cid, _path, mode, expect, _args in found:
            self.assertEqual(expect is not None, mode == "run", cid)
            if mode == "run":
                self.assertIn("contains", expect, cid)


class SetNameTests(unittest.TestCase):
    def test_an_unknown_set_name_lists_the_sets(self):
        with self.assertRaisesRegex(SystemExit, "unknown set `modules`.*corpus.*upstream"):
            list(jaic_sweep.cases("modules"))


class NativeLibsTests(unittest.TestCase):
    def test_a_windows_host_builds_and_uses_the_native_libraries(self):
        # mbedTLS (Http) and the other linked libraries are built on Windows too, or native
        # builds of the examples cannot link.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            helper = root / "stdlib/internal/developer-tools/debug-native/developer_tools_debug.dll"
            helper.parent.mkdir(parents=True)
            helper.touch()
            built = []

            def build(plat):
                built.append(plat)
                os.environ["JAIC_NATIVE_LIBS"] = f"libs-{plat}"

            with (
                mock.patch.dict(os.environ, {}, clear=False),
                mock.patch.object(jaic_sweep.sys, "platform", "win32"),
                mock.patch.object(jaic_sweep, "ROOT", root),
                mock.patch("repo_output.host_platform", return_value="windows-x64"),
                mock.patch("repo_output.shared_output_dir", return_value=root / "none"),
                mock.patch("stdlib.stdlib_runtime.use_native_libs", side_effect=build),
            ):
                os.environ.pop("JAIC_NATIVE_LIBS", None)
                jaic_sweep.use_native_libs()
                self.assertEqual(built, ["windows-x64"])
                self.assertEqual(os.environ["JAIC_NATIVE_LIBS"], "libs-windows-x64")


class SetupCommandTests(unittest.TestCase):
    def test_jacket_in_a_setup_command_is_the_binary_next_to_jaic_and_ignores_the_users_git_config(self):
        with tempfile.TemporaryDirectory() as tmp:
            jaic = Path(tmp) / "jaic"
            jacket = Path(tmp) / "jacket"
            jaic.write_text("")
            jacket.write_text("")
            command, env = jaic_sweep.setup_command(["{jacket}", "migrate", "--apply", "."], str(jaic), {"A": "1"})
            self.assertEqual(command, [str(jacket), "migrate", "--apply", "."])
            self.assertEqual(env["A"], "1")
            self.assertEqual(env["GIT_CONFIG_GLOBAL"], os.devnull)
            self.assertEqual(env["GIT_CONFIG_SYSTEM"], os.devnull)
            # Any other command is left as written.
            self.assertEqual(jaic_sweep.setup_command(["make"], str(jaic), {"A": "1"}), (["make"], {"A": "1"}))
            jacket.unlink()
            with self.assertRaisesRegex(SystemExit, "build jacket"):
                jaic_sweep.setup_command(["{jacket}"], str(jaic), {})


if __name__ == "__main__":
    unittest.main()
