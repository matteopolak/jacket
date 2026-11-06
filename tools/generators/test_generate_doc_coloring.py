"""Spec admission and nonmutating generated-coloring checks."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from generators import generate_doc_coloring as generator


class ColoringGeneratorTests(unittest.TestCase):
    def fixture(self, root):
        for relative in ["crates/jacket/src/doc/coloring/spec.json", "tools/generators/templates/doc-coloring.ts"]:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((generator.ROOT / relative).read_bytes())

    def test_check_does_not_repair_stale_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            generator.generate(root=root)
            path = root / generator.OUTPUTS[0]
            generator.generate(True, root=root)
            path.write_text("stale\n")
            with self.assertRaisesRegex(ValueError, "stale"):
                generator.generate(True, root=root)
            self.assertEqual(path.read_text(), "stale\n")

    def test_generated_bytes_ignore_windows_text_translation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            original = Path.write_text

            def windows_write(path, text, encoding=None, errors=None, newline=None):
                return original(path, text, encoding=encoding or "cp1252", errors=errors,
                                newline="\r\n" if newline is None else newline)

            with mock.patch.object(Path, "write_text", windows_write):
                generator.generate(root=root)
            data = (root / generator.OUTPUTS[0]).read_bytes()
            self.assertNotIn(b"\r", data)
            self.assertEqual(data, (generator.ROOT / generator.OUTPUTS[0]).read_bytes())

    def test_check_rejects_crlf_without_repair(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            generator.generate(root=root)
            path = root / generator.OUTPUTS[0]
            data = path.read_bytes().replace(b"\n", b"\r\n")
            path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "stale"):
                generator.generate(True, root=root)
            self.assertEqual(path.read_bytes(), data)

    def test_empty_delimiters_and_unknown_operations_fail_before_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            path = root / "crates/jacket/src/doc/coloring/spec.json"
            original = json.loads(path.read_text())
            for kind in ["empty", "unknown"]:
                spec = json.loads(json.dumps(original))
                if kind == "empty":
                    spec["rules"][1]["open"] = ""
                else:
                    spec["rules"][1]["kind"] = "unexpected"
                path.write_text(json.dumps(spec))
                with self.assertRaises(ValueError):
                    generator.generate(root=root)
                self.assertFalse((root / generator.OUTPUTS[0]).exists())
