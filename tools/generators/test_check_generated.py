import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import check_generated as cg
import generate_reference_docs as reference
from toolkit.repo import ROOT


class Tree(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        (self.root / "tools/generators").mkdir(parents=True)
        (self.root / "out.txt").write_text("x")

    def script(self, name, text, folder="tools/generators"):
        path = self.root / folder / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def test_a_declaring_script_is_discovered_with_its_flags(self):
        self.script("generate_a.py", 'OUTPUTS = ["out.txt"]\nFORMATTED = True\nDOWNLOADS = True\nif __name__ == "__main__":\n    pass\n')
        generators, problems = cg.discover(self.root)
        self.assertEqual(problems, [])
        self.assertEqual(generators, [cg.Generator("tools/generators/generate_a.py", ("out.txt",), True, True)])

    def test_a_script_outside_the_generators_folder_is_found_by_its_declaration(self):
        self.script("gen_b.py", 'OUTPUTS = ["out.txt"]\n', folder="tools/corpus")
        generators, problems = cg.discover(self.root)
        self.assertEqual((problems, [g.script for g in generators]), ([], ["tools/corpus/gen_b.py"]))

    def test_a_generator_without_outputs_fails(self):
        self.script("generate_c.py", 'if __name__ == "__main__":\n    pass\n')
        _, problems = cg.discover(self.root)
        self.assertEqual(len(problems), 1)
        self.assertIn("generate_c.py declares no OUTPUTS", problems[0])

    def test_empty_or_non_literal_outputs_fail(self):
        self.script("generate_d.py", "OUTPUTS = []\n")
        self.script("generate_e.py", "OUTPUTS = [ROOT / 'x']\n")
        self.script("generate_f.py", 'OUTPUTS = [1]\n')
        _, problems = cg.discover(self.root)
        self.assertEqual(len(problems), 3)

    def test_a_missing_output_or_a_shared_one_fails(self):
        self.script("generate_g.py", 'OUTPUTS = ["gone.txt", "out.txt"]\n')
        self.script("generate_h.py", 'OUTPUTS = ["out.txt"]\n')
        _, problems = cg.discover(self.root)
        self.assertTrue(any("gone.txt does not exist" in p for p in problems))
        self.assertTrue(any("declared by both" in p for p in problems))

    def test_helpers_and_tests_are_not_generators(self):
        self.script("helper.py", "def shared():\n    pass\n")
        self.script("test_generate_i.py", 'import unittest\nif __name__ == "__main__":\n    unittest.main()\n')
        generators, problems = cg.discover(self.root)
        self.assertEqual((generators, problems), ([], []))

    def test_flags_must_be_booleans(self):
        self.script("generate_j.py", 'OUTPUTS = ["out.txt"]\nFORMATTED = "yes"\n')
        _, problems = cg.discover(self.root)
        self.assertEqual(len(problems), 1)

    def test_commands_pass_the_formatter_only_to_formatted_generators_and_skip_downloads_offline(self):
        generators = [cg.Generator("a.py", ("x",), formatted=True), cg.Generator("b.py", ("y",)),
                      cg.Generator("c.py", ("z",), downloads=True)]
        self.assertEqual(cg.commands(generators, "/fmt", offline=False), [["a.py", "--formatter", "/fmt"], ["b.py"], ["c.py"]])
        self.assertEqual(cg.commands(generators, "/fmt", offline=True), [["a.py", "--formatter", "/fmt"], ["b.py"]])


class Repository(unittest.TestCase):
    def test_every_generator_declares_existing_outputs(self):
        generators, problems = cg.discover(ROOT)
        self.assertEqual(problems, [])
        self.assertGreater(len(generators), 10)


class ReferenceDocs(unittest.TestCase):
    @staticmethod
    def help(program, *args):
        if program == "jacket" and args == ("--help",):
            names = "build check run lint fmt lsp jaic jailint jailsp jaifmt new init test bench doc config clean explain version upgrade help".split()
            return "Usage: jacket <COMMAND>\n\nCommands:\n" + "".join(f"  {name}  Description\n" for name in names) + "\nOptions:\n  --help  Help\n"
        return "Usage: owned-tool [OPTIONS]\n\nOptions:\n  --help  Help\n"

    def test_prebuilt_formatter_omits_llvm_compiler_build_and_uses_current_help(self):
        with mock.patch.object(reference, "cargo") as build, mock.patch.object(reference, "help_text", side_effect=self.help), mock.patch.object(reference, "example", return_value="# Environment\n"), mock.patch.object(reference, "build_jaifmt", side_effect=AssertionError("must not compile a supplied formatter")), mock.patch.object(reference.subprocess, "run", return_value=SimpleNamespace(stdout="Usage: jaifmt [OPTIONS]\n\nOptions:\n  --declarations  Documentation layout\n")) as run:
            text = reference.tools_page(formatter=Path("/owned/jaifmt"))
        self.assertIn("--declarations", text)
        build.assert_called_once_with("build", "--locked", "-q", "-p", "jacket", "-p", "jailint", "-p", "jai-language-server")
        self.assertEqual(run.call_args.args[0], ["/owned/jaifmt", "--help"])

    def test_default_still_builds_the_compiler_and_formatter(self):
        with mock.patch.object(reference, "cargo") as build, mock.patch.object(reference, "help_text", side_effect=self.help), mock.patch.object(reference, "example", return_value="# Environment\n"), mock.patch.object(reference, "build_jaifmt", return_value=Path("/built/jaifmt")) as formatter, mock.patch.object(reference.subprocess, "run", return_value=SimpleNamespace(stdout="Usage: jaifmt [OPTIONS]\n")):
            reference.tools_page()
        self.assertIn("jaic-cli", build.call_args.args)
        formatter.assert_called_once_with()

    def test_reuse_refresh_does_not_rewrite_unchanged_lint_reference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lint = root / "docs/reference/lint-rules.md"
            tools = root / "docs/reference/tools.md"
            lint.parent.mkdir(parents=True)
            lint.write_text("exact owned lint reference\n")
            tools.write_text("old tools\n")
            formatter = root / "jaifmt"
            formatter.touch()
            before = lint.stat().st_mtime_ns
            rendered = {"docs/reference/lint-rules.md": lint.read_text(), "docs/reference/tools.md": "fresh tools\n"}
            with mock.patch.object(reference, "ROOT", root), mock.patch.object(reference, "render", return_value=rendered) as render, mock.patch("sys.argv", ["generate_reference_docs.py", "--formatter", str(formatter)]):
                self.assertEqual(reference.main(), 0)
            render.assert_called_once_with(formatter=formatter.resolve())
            self.assertEqual(lint.stat().st_mtime_ns, before)
            self.assertEqual(lint.read_text(), "exact owned lint reference\n")
            self.assertEqual(tools.read_text(), "fresh tools\n")


if __name__ == "__main__":
    unittest.main()
