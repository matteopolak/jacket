"""The generated-file gate compares Rust authority without rewriting checked-in source."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from generators import generate_doc_site as generator


class DocSiteGeneratorTests(unittest.TestCase):
    def fixture(self, root):
        for name in generator.OUTPUTS:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("checked source\n")
        for name in ("ajv", "json-schema-to-typescript"):
            path = root / "website/docs/node_modules" / name / "package.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("{}")

    def test_check_compares_authority_and_asks_node_to_check_without_source_mutation(
        self,
    ):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            before = {name: (root / name).read_bytes() for name in generator.OUTPUTS}
            calls = []

            def run(command, **_):
                calls.append(command)
                if command[0] == "cargo":
                    Path(command[-1]).write_text("checked source\n")
                return subprocess.CompletedProcess(command, 0)

            generator.generate(True, root=root, run=run)
            self.assertIn("--check", calls[-1])
            self.assertEqual(
                before, {name: (root / name).read_bytes() for name in generator.OUTPUTS}
            )

    def test_stale_schema_stops_before_node_and_missing_setup_is_actionable(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            calls = []

            def run(command, **_):
                calls.append(command)
                if command[0] == "cargo":
                    Path(command[-1]).write_text("different Rust schema\n")
                return subprocess.CompletedProcess(command, 0)

            with self.assertRaisesRegex(ValueError, "stale"):
                generator.generate(True, root=root, run=run)
            self.assertEqual(len(calls), 1)
            self.assertEqual(
                (root / generator.OUTPUTS[0]).read_text(), "checked source\n"
            )
            (root / "website/docs/node_modules/ajv/package.json").unlink()
            with self.assertRaisesRegex(ValueError, "npm ci"):
                generator.generate(True, root=root, run=run)


if __name__ == "__main__":
    unittest.main()
