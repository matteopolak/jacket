"""Pure caller-font runner policy; no compiler, SDK or font replacement."""

import ast
import os
from pathlib import Path
import tempfile
import unittest

SOURCE = Path(__file__).with_name("verify.py")


class FontArguments(unittest.TestCase):
    def setUp(self):
        tree = ast.parse(SOURCE.read_text())
        node = next(
            n
            for n in tree.body
            if isinstance(n, ast.FunctionDef) and n.name == "fixture_arguments"
        )
        namespace = {"Path": Path}
        exec(
            compile(ast.Module(body=[node], type_ignores=[]), str(SOURCE), "exec"),
            namespace,
        )
        self.arguments = namespace["fixture_arguments"]

    def test_unrelated_fixture_does_not_require_font(self):
        self.assertEqual(self.arguments("lz4", {}), [])

    def test_missing_nonexistent_or_directory_font_fails(self):
        with tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"]) as directory:
            for value in (None, str(Path(directory) / "absent.ttf"), directory):
                with (
                    self.subTest(value=value),
                    self.assertRaisesRegex(ValueError, "JAIC_TEST_FONT_PATH"),
                ):
                    self.arguments(
                        "freetype",
                        {} if value is None else {"JAIC_TEST_FONT_PATH": value},
                    )

    def test_exact_caller_path_is_passed_to_both_modes(self):
        with tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"]) as directory:
            font = Path(directory) / "caller 字 font.ttf"
            font.write_bytes(b"input policy marker only; never executed as a font")
            self.assertEqual(
                self.arguments("freetype", {"JAIC_TEST_FONT_PATH": str(font)}),
                [str(font.resolve())],
            )


if __name__ == "__main__":
    unittest.main()
