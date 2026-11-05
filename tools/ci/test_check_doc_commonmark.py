"""Conformance accounting is fail-closed before any compiler or browser work."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import check_doc_commonmark as check


class InventoryTests(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        cases = [{"example": 1, "markdown": "# x\n", "html": "<h1>x</h1>\n", "section": "Headings"},
                 {"example": 2, "markdown": "hi\n", "html": "<p>hi</p>\n", "section": "Text"}]
        data = json.dumps(cases).encode()
        (root / "spec.json").write_bytes(data)
        manifest = {"version": "0.31.2", "count": 2, "sha256": hashlib.sha256(data).hexdigest(),
                    "source": "https://spec.commonmark.org/0.31.2/spec.json", "license": "CC-BY-SA-4.0"}
        (root / "manifest.json").write_text(json.dumps(manifest))
        return root, cases

    def test_pinned_complete_inventory_and_exact_ledger(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, cases = self.fixture(temporary)
            self.assertEqual(check.load_cases(root), cases)
            check.validate_ledger(cases, [{"id": 1, "status": "pass"}, {"id": 2, "status": "pass"}])
            for ledger in ([{"id": 1, "status": "pass"}], [{"id": 1, "status": "pass"}] * 2,
                           [{"id": 1, "status": "pass"}, {"id": 2, "status": "skip"}],
                           [{"id": 1, "status": "pass"}, {"id": 3, "status": "pass"}]):
                with self.assertRaises(ValueError):
                    check.validate_ledger(cases, ledger)

    def test_modified_corpus_or_discontinuous_ids_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, cases = self.fixture(temporary)
            (root / "spec.json").write_text("[]")
            with self.assertRaisesRegex(ValueError, "digest"):
                check.load_cases(root)
            cases[1]["example"] = 3
            data = json.dumps(cases).encode()
            (root / "spec.json").write_bytes(data)
            manifest = json.loads((root / "manifest.json").read_text())
            manifest["sha256"] = hashlib.sha256(data).hexdigest()
            (root / "manifest.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "contiguous"):
                check.load_cases(root)


if __name__ == "__main__":
    unittest.main()
