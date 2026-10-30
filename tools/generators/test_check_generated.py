import tempfile
import unittest
from pathlib import Path

import check_generated as cg
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


if __name__ == "__main__":
    unittest.main()
