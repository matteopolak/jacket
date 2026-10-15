"""Metadata-only audit checkpoint regressions; never reads corpus source."""
import json
from pathlib import Path
import tempfile
import unittest

from ide_corpus_audit import read_progress


class Checkpoints(unittest.TestCase):
    def read(self, *records, suffix=""):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "audit.jsonl"
            path.write_text("".join(json.dumps(record) + "\n" for record in records) + suffix)
            return read_progress(path)

    def header(self):
        return dict(kind="header", redacted=True, build_id="test-source", stdlib_id="test-dependency",
                    expected_inputs=3, selection=None)

    def test_killed_append_preserves_completed_files_without_claiming_acceptance(self):
        result = self.read(self.header(), dict(kind="plan", redacted=True, fingerprint="test-plan"),
                           dict(kind="file", redacted=True, file=dict(input_id="f000000", status="audited")),
                           dict(kind="start", redacted=True, input_id="f000001"), suffix='{"kind":')
        self.assertFalse(result["accepted"])
        self.assertTrue(result["partial"])
        self.assertEqual(result["completed_files"], 1)
        self.assertEqual(result["remaining_inputs"], 2)
        self.assertEqual(result["cursor"]["input_id"], "f000001")
        self.assertEqual(result["plan"], "test-plan")

    def test_unexecuted_record_is_not_completed_query_coverage(self):
        result = self.read(self.header(), dict(kind="file", redacted=True,
                                              file=dict(input_id="f000000", status="unexecuted")))
        self.assertEqual(result["remaining_inputs"], 3)
        self.assertEqual(result["unexecuted_records"], 1)

    def test_unredacted_progress_rejected(self):
        with self.assertRaises(ValueError):
            self.read(self.header(), dict(kind="file", redacted=False))

    def test_missing_header_is_not_evidence(self):
        self.assertIsNone(self.read(dict(kind="start", redacted=True, input_id="f000000")))

    def test_nonfinal_corruption_rejected(self):
        with self.assertRaises(ValueError):
            self.read(self.header(), suffix='broken\n{"kind":"start","redacted":true}\n')


if __name__ == "__main__":
    unittest.main()
