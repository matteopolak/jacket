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


if __name__ == "__main__":
    unittest.main()
