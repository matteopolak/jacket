import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from stdlib_coverage import code_lines, declarations
from stdlib_runtime import contract_result
from stdlib_test_catalog import validated_entries, unavailable_reason


class CatalogueTests(unittest.TestCase):
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
