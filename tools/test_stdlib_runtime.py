import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from stdlib_coverage import code_lines, declarations
from stdlib_runtime import load_skips, skip_reason


class SkipListTests(unittest.TestCase):
    def skips(self, text):
        with TemporaryDirectory() as d:
            path = Path(d) / "skips.txt"
            path.write_text(text)
            return load_skips(path)

    def test_patterns_modes_and_comments(self):
        skips = self.skips(
            "# header\n\n"
            "a linux-*,macos-arm64 native,wasm-native no libfoo there  # trailing comment\n"
            "b * * never\n"
        )
        self.assertEqual(skip_reason(skips, "a", "linux-arm64", "native"), "no libfoo there")
        self.assertEqual(skip_reason(skips, "a", "macos-arm64", "wasm-native"), "no libfoo there")
        self.assertIsNone(skip_reason(skips, "a", "macos-x64", "native"))
        self.assertIsNone(skip_reason(skips, "a", "linux-x64", "interp"))
        self.assertEqual(skip_reason(skips, "b", "windows-x64-mingw", "wasm-interp"), "never")
        self.assertIsNone(skip_reason(skips, "c", "linux-x64", "interp"))

    def test_malformed_lines_are_rejected(self):
        with self.assertRaises(SystemExit):
            self.skips("a * native\n")
        with self.assertRaises(SystemExit):
            self.skips("a * compile no such mode\n")


class CoverageParserTests(unittest.TestCase):
    def test_comments_and_strings_are_blanked(self):
        lines = code_lines('x :: "a { b"; // c :: () {}\n/* d :: () {} */ e :: 1;\n#string END\nf :: () {}\nEND\ng :: 2;')
        self.assertEqual(lines[0].strip(), 'x :: "";')
        self.assertEqual(lines[1].strip(), "e :: 1;")
        self.assertEqual(lines[3], "")
        self.assertEqual(lines[5], "g :: 2;")

    def test_public_procedures_follow_scopes_loads_and_ifs(self):
        with TemporaryDirectory() as d:
            root = Path(d)
            (root / "part.jai").write_text("loaded :: () {}\n")
            (root / "module.jai").write_text(
                '#load "part.jai";\n'
                "public :: (x: int) -> int { return x; }\n"
                "#if OS == .LINUX {\n    in_if :: () {\n        nested :: () {}\n    }\n}\n"
                "foreign :: () #foreign libc;\n"
                "header :: (a: int,\n    b: int) {\n}\n"
                "macro :: () #expand {}\n"
                "Type :: #type () -> int;\n"
                "#scope_file\nhidden :: () {}\n"
            )
            found = {p["name"]: p for p in declarations(root / "module.jai", set())}
            self.assertEqual(sorted(found), ["header", "in_if", "loaded", "macro", "public"])
            self.assertTrue(found["macro"]["macro"])
            self.assertEqual(found["header"]["line"], 9)


if __name__ == "__main__":
    unittest.main()
