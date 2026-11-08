"""Check that the JPEG case list is derived from the pinned files, not stored per case."""

import json
from pathlib import Path
import tempfile
import unittest

import inventory
from toolkit.hashing import sha256_bytes, sha256_file

HERE = Path(__file__).resolve().parent


def sample(files):
    return {
        "case_count": 3,
        "expectations": {"codec-corpus/jpeg-conformance/valid/odd.jpg": "reject"},
        "unsupported_process": ["codec-corpus/jpeg-conformance/valid/odd.jpg"],
        "baseline_reference": {
            "codec-corpus/jpeg-conformance/invalid/bad.jpg": {
                "exit": 2,
                "warnings": None,
                "stderr": "x\n",
            }
        },
        "expectation_provenance": {},
        "sets": [
            {
                "name": "codec-corpus",
                "repository": "https://github.com/example/corpus",
                "revision": "abc",
                "files": files,
            }
        ],
    }


FILES = {
    "README.md": "0" * 64,
    "jpeg-conformance/valid/ok.jpg": "1" * 64,
    "jpeg-conformance/valid/odd.jpg": "2" * 64,
    "jpeg-conformance/invalid/bad.jpg": "3" * 64,
    "jpeg-conformance/other/skipped.jpg": "4" * 64,
}


class InventoryTests(unittest.TestCase):
    def test_cases_take_class_and_expectation_from_the_path(self):
        cases = {c["id"]: c for c in inventory.cases(sample(FILES))}
        self.assertEqual(len(cases), 3)
        ok = cases["codec-corpus/jpeg-conformance/valid/ok.jpg"]
        self.assertEqual((ok["class"], ok["expectation"]), ("valid", "accept"))
        self.assertEqual(ok["baseline_reference"], inventory.CLEAN_BASELINE)
        self.assertFalse(ok["unsupported_process"])

    def test_listed_exceptions_override_the_defaults(self):
        cases = {c["id"]: c for c in inventory.cases(sample(FILES))}
        odd = cases["codec-corpus/jpeg-conformance/valid/odd.jpg"]
        self.assertEqual(odd["expectation"], "reject")
        self.assertTrue(odd["unsupported_process"])
        bad = cases["codec-corpus/jpeg-conformance/invalid/bad.jpg"]
        self.assertEqual(bad["baseline_reference"]["exit"], 2)

    def test_urls_pin_the_revision_and_escape_spaces(self):
        pins = inventory.sources(sample({"a b.jpg": "0" * 64}))
        self.assertEqual(
            pins["codec-corpus/a b.jpg"]["url"],
            "https://raw.githubusercontent.com/example/corpus/abc/a%20b.jpg",
        )

    def test_corpus_check_reports_changed_missing_and_extra_files(self):
        data = b"jpeg"
        files = {
            "jpeg-conformance/valid/ok.jpg": sha256_bytes(data),
            "jpeg-conformance/valid/odd.jpg": sha256_bytes(data),
            "jpeg-conformance/invalid/bad.jpg": sha256_bytes(b"other"),
        }
        manifest = sample(files)
        with tempfile.TemporaryDirectory() as directory:
            corpus = Path(directory)
            for name in files:
                target = corpus / "codec-corpus" / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            (corpus / "codec-corpus/jpeg-conformance/valid/extra.jpg").write_bytes(data)
            errors = inventory.check_corpus(manifest, corpus, sha256_file)
        self.assertEqual(
            errors,
            [
                "missing or changed: codec-corpus/jpeg-conformance/invalid/bad.jpg",
                "unpinned JPEG: codec-corpus/jpeg-conformance/valid/extra.jpg",
            ],
        )

    def test_checked_in_pins_define_the_full_inventory(self):
        manifest = json.loads((HERE / "manifest.json").read_text())
        ids = {c["id"] for c in inventory.cases(manifest)}
        self.assertEqual(len(ids), manifest["case_count"])
        self.assertEqual(len(inventory.sources(manifest)), 296)
        self.assertTrue(set(manifest["above_production_cap_cases"]) <= ids)


if __name__ == "__main__":
    unittest.main()
