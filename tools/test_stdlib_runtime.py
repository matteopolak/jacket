import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

from stdlib_coverage import code_lines, declarations
from stdlib_runtime import contract_result, run_mode, program_arguments, runtime_prefix
from stdlib_test_catalog import validated_entries, unavailable_reason


class CatalogueTests(unittest.TestCase):
    def test_appledouble_metadata_is_not_an_independent_program(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "tests/stdlib-contract").mkdir(parents=True)
            (root / "tests/stdlib-contract/manifest.json").write_text("[]")
            regression = root / "tests/compiler-regressions"
            regression.mkdir()
            (regression / "actual.jai").write_text("main :: () {}")
            (regression / "._actual.jai").write_bytes(b"\x00\x05\x16\x07")
            module = root / "stdlib/Real/tests"
            module.mkdir(parents=True)
            (module / "actual.jai").write_text("main :: () {}")
            (module / "._actual.jai").write_bytes(b"\x00\x05\x16\x07")
            self.assertEqual(
                {c["id"] for c in validated_entries(root)},
                {"compiler/actual", "Real/tests/actual"},
            )


    def test_cocoa_unit_receives_its_actual_default_configuration(self):
        case = next(c for c in validated_entries() if c['id'] == 'Clipboard/tests/cocoa-operations')
        self.assertEqual(program_arguments(case, '.', {}), ['enabled'])
        self.assertEqual(case['required_os'], 'macos')

    def test_mac_clipboard_requires_a_restoration_guard(self):
        with self.assertRaises(ValueError):
            runtime_prefix({"clipboard_guard": True}, {}, "darwin")
        self.assertEqual(runtime_prefix({}, {}, "darwin"), [])

    def test_native_platform_requirement_is_not_a_success(self):
        case = {"family": "modules", "required_os": "windows"}
        self.assertIsNotNone(unavailable_reason(case, "native", "macos-arm64"))
        self.assertIsNone(unavailable_reason(case, "native", "windows-arm64"))
        self.assertIsNotNone(unavailable_reason(case, "wasm-interp", "windows-arm64"))
        case["required_os"] = ("linux", "macos")
        self.assertIsNone(unavailable_reason(case, "native", "macos-arm64"))
        self.assertIsNotNone(unavailable_reason(case, "native", "windows-x64"))

    def test_prepared_inputs_are_real_and_isolated(self):
        with TemporaryDirectory() as temporary:
            directory = program_arguments({"id": "resource", "input": "scratch-directory"}, temporary, {})[0]
            self.assertTrue(Path(directory).is_dir())
            file = program_arguments({"id": "io", "input": "scratch-file"}, temporary, {})[0]
            self.assertTrue(Path(file).parent.is_dir())
            self.assertFalse(Path(file).exists())
            self.assertNotEqual(Path(file).parent, Path(directory))
            with self.assertRaises(ValueError):
                program_arguments({"id": "glyph", "input": "font"}, temporary, {})

    def test_reviewed_and_independent_families_are_discovered(self):
        cases = validated_entries()
        self.assertEqual({c["family"] for c in cases}, {"contracts", "compiler", "extensions", "native", "graphics", "modules"})
        self.assertTrue(any(c["id"] == "contracts/hash-private-seed-is-not-exported" for c in cases))
        self.assertTrue(any(c["id"] == "compiler/array-bounds-check-opt-out" for c in cases))
        self.assertFalse(any("tests/stdlib/" in str(c["path"]) for c in cases))

    def test_precise_negative_and_positive_output_are_required(self):
        case = {"mode": "check-fail", "diagnostic_contains": "has no exported member `HASH_INIT`"}
        self.assertFalse(contract_result(case, 0, case["diagnostic_contains"]))
        self.assertFalse(contract_result(case, 1, "unrelated syntax error"))
        self.assertTrue(contract_result(case, 1, "error: " + case["diagnostic_contains"]))
        self.assertFalse(contract_result({"stdout": "ok\n"}, 0, "wrong\n"))
        self.assertIsNone(unavailable_reason({"family": "native"}, "native"))
        self.assertIsNotNone(unavailable_reason({"family": "native"}, "wasm-native"))

    def test_expected_diagnostics_are_separate_from_program_stdout(self):
        case = {"stdout": "ok\n", "stderr": "expected diagnostic\n"}
        self.assertTrue(contract_result(case, 0, "ok\n", "expected diagnostic\n"))
        self.assertFalse(contract_result(case, 0, "ok\n", "unrelated failure\n"))
        self.assertFalse(contract_result(case, 0, "wrong\n", "expected diagnostic\n"))

    def test_missing_main_does_not_hide_other_compile_errors(self):
        args = SimpleNamespace(jaic="own-jaic", opt="O0", timeout=5, coverage=None, dlls=None)
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "program.jai"
            with patch("stdlib_runtime.run_command", side_effect=[
                (1, "", "program has no `main` procedure; unknown identifier genuine_error"),
                (1, "", "unknown identifier genuine_error"),
            ]):
                passed, detail = run_mode(args, "program", path, "native", temporary)
                self.assertIs(passed, False, detail)

    def test_timeout_is_not_reclassified_as_compile_time_only(self):
        args = SimpleNamespace(jaic="own-jaic", opt="O0", timeout=5, coverage=None, dlls=None)
        with TemporaryDirectory() as temporary:
            with patch("stdlib_runtime.run_command", return_value=(None, "", "program has no `main` procedure")):
                passed, detail = run_mode(args, "program", Path(temporary) / "program.jai", "native", temporary)
                self.assertIs(passed, False, detail)


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
