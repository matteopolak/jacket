"""CI capture-retention policy only; never launches the HTTP workload."""

import json
from pathlib import Path
import tempfile
import unittest

from ci_test_scratch import assigned_scratch

WORKFLOW = (
    Path(__file__).resolve().parents[1] / ".github/workflows/http-conformance.yml"
)
STEP = "      - name: Retain the selected Linux observer capture"


class ObserverArtifacts(unittest.TestCase):
    def upload(self):
        return WORKFLOW.read_text().split(STEP, 1)[1]

    def paths(self):
        block = (
            self.upload()
            .split("          path: |\n", 1)[1]
            .split("          retention-days:", 1)[0]
        )
        return [line.strip() for line in block.splitlines() if line.strip()]

    def test_failed_original_step_still_uploads_without_waiving_failure(self):
        text = WORKFLOW.read_text()
        self.assertGreater(
            text.index(STEP), text.index("- name: Io and Http tests under load")
        )
        self.assertIn("always() && !cancelled() && runner.os == 'Linux'", self.upload())
        self.assertNotIn("continue-on-error", text)
        self.assertNotIn("outcome", self.upload())
        self.assertIn("--copies 12 --observe-copy backends:o2:1:0", text)
        self.assertIn('--rounds 3 "${observe[@]}"', text)

    def test_only_three_explicit_artifacts_no_recursive_or_source_upload(self):
        self.assertEqual(
            self.paths(),
            [
                "${{ runner.temp }}/io-observed-copy/receipt.json",
                "${{ runner.temp }}/io-observed-copy/program.stdout",
                "${{ runner.temp }}/io-observed-copy/program.stderr",
            ],
        )
        self.assertTrue(all("*" not in path for path in self.paths()))
        self.assertIn("retention-days: 7", self.upload())

    def test_collecting_receipt_is_retained_and_sibling_sentinel_is_excluded(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as directory:
            root = Path(directory)
            (root / "receipt.json").write_text(json.dumps({"status": "collecting"}))
            (root / "synthetic-env-source-auth-sentinel.txt").write_text(
                "test marker only"
            )
            selected = [
                root / Path(path).name
                for path in self.paths()
                if (root / Path(path).name).is_file()
            ]
            self.assertEqual(selected, [root / "receipt.json"])
            self.assertEqual(
                json.loads(selected[0].read_text())["status"], "collecting"
            )
            self.assertIn("if-no-files-found: warn", self.upload())


if __name__ == "__main__":
    unittest.main()
