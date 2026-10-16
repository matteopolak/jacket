"""Header configuration tests; CMake script mode only, no native compiler/run."""

import ast
import importlib.util
from sdl_test_support import source_path, scratch_root
from pathlib import Path
import subprocess
from sdl_test_support import temporary_directory
import unittest
from unittest.mock import patch


class SdlSdkHeaders(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "sdl_sdk_headers",
            source_path(
                "SDL_SDK_TEST_HELPER", "tools/native_bindings/sdl_sdk_headers.py"
            ),
        )
        cls.helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.helper)
        cls.source = source_path(
            "SDL_SDK_TEST_SOURCE", scratch_root() / "sdl-sdk-source"
        )

    def setUp(self):
        self.temp = temporary_directory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name) / "generated headers"

    def test_builder_prepares_generated_runtime_headers_once_before_compiling(self):
        tree = ast.parse(
            Path(
                source_path(
                    "SDL_SDK_TEST_BUILDER",
                    "tools/native_bindings/build_dependencies.py",
                )
            ).read_text()
        )
        calls = [
            n
            for n in ast.walk(tree)
            if isinstance(n, ast.Call)
            and isinstance(n.func, ast.Name)
            and n.func.id == "prepare_runtime_headers"
        ]
        self.assertEqual(
            len(calls), 1, "runtime probe must configure the pinned SDK headers"
        )
        main = next(
            n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "main"
        )
        preparation = next(
            n
            for n in main.body
            if isinstance(n, ast.Assign)
            and any(
                isinstance(t, ast.Name) and t.id == "runtime_sdl_headers"
                for t in n.targets
            )
        )
        self.assertIsInstance(preparation.value, ast.IfExp)
        self.assertIn(
            "'sdl_runtime_abi_probe' in selected", ast.unparse(preparation.value.test)
        )
        self.assertEqual(ast.literal_eval(preparation.value.orelse), [])
        runtime_branch = next(
            n
            for n in ast.walk(main)
            if isinstance(n, ast.If)
            and ast.unparse(n.test) == "name == 'sdl_runtime_abi_probe'"
        )
        self.assertIn("for root in runtime_sdl_headers", ast.unparse(runtime_branch))

    def test_real_pinned_template_is_generated_without_mutating_sources(self):
        template = self.source / "include/SDL2/SDL_config_unix.h.cmake"
        before = template.read_bytes()
        self.assertFalse(template.with_suffix("").exists())
        roots = self.helper.prepare_runtime_headers(self.source, self.output)
        self.assertEqual(roots, [self.output, template.parent])
        actual = (roots[0] / "SDL_config_unix.h").read_bytes()
        self.assertEqual(
            actual,
            before.replace(
                b"#cmakedefine SDL2COMPAT_DISABLE_X11",
                b"/* #undef SDL2COMPAT_DISABLE_X11 */",
            ),
        )
        self.assertIn(b"#define SDL_VIDEO_DRIVER_X11 1", actual)
        self.assertIn(b"#define SDL_VIDEO_DRIVER_WAYLAND 1", actual)
        self.assertIn(b"#define SDL_VIDEO_OPENGL 1", actual)
        self.assertEqual(template.read_bytes(), before)
        self.helper.prepare_runtime_headers(self.source, self.output)
        self.assertEqual((self.output / "SDL_config_unix.h").read_bytes(), actual)

    def test_failed_cmake_removes_stale_config_and_propagates_failure(self):
        self.output.mkdir()
        generated = self.output / "SDL_config_unix.h"
        generated.write_text("stale configuration")
        with patch.object(
            self.helper.subprocess,
            "run",
            side_effect=subprocess.CalledProcessError(1, "cmake"),
        ):
            with self.assertRaises(subprocess.CalledProcessError):
                self.helper.prepare_runtime_headers(self.source, self.output)
        self.assertFalse(generated.exists())

    def test_missing_cmake_output_is_fatal(self):
        with patch.object(self.helper.subprocess, "run"):
            with self.assertRaisesRegex(RuntimeError, "did not generate"):
                self.helper.prepare_runtime_headers(self.source, self.output)

    def test_unknown_template_contract_is_not_guessed(self):
        import shutil

        copied = Path(self.temp.name) / "source"
        shutil.copytree(self.source, copied)
        template = copied / "include/SDL2/SDL_config_unix.h.cmake"
        for addition in (
            "\n#cmakedefine FUTURE_OPTION\n",
            "\n#define VALUE @FUTURE_VALUE@\n",
            "\n#define VALUE ${FUTURE_VALUE}\n",
        ):
            with self.subTest(addition=addition):
                template.write_bytes(
                    (self.source / "include/SDL2/SDL_config_unix.h.cmake").read_bytes()
                    + addition.encode()
                )
                with self.assertRaisesRegex(RuntimeError, "Unreviewed"):
                    self.helper.prepare_runtime_headers(copied, self.output)
        template.write_bytes(
            (self.source / "include/SDL2/SDL_config_unix.h.cmake").read_bytes()
        )
        cmake = copied / "CMakeLists.txt"
        cmake.write_text(
            cmake.read_text().replace(
                '"Enable X11 support" ON', '"Enable X11 support" OFF'
            )
        )
        with self.assertRaisesRegex(RuntimeError, "X11 default changed"):
            self.helper.prepare_runtime_headers(copied, self.output)

    def test_missing_template_is_fatal(self):
        with self.assertRaises(FileNotFoundError):
            self.helper.prepare_runtime_headers(Path(self.temp.name), self.output)


if __name__ == "__main__":
    unittest.main()
