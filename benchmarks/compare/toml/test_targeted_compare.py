"""Causal recording regressions using the actual decoded-string Run shape."""

import ast
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path.cwd()
sys.path[:0] = [str(ROOT / "tools"), str(ROOT / "benchmarks/compare/toml")]
import measure
from bench.measure import Run

SCRIPT = Path(
    os.environ.get(
        "TOML_FOLLOWUP_CONTROLLER", str(Path(__file__).with_name("targeted-compare.py"))
    )
)
tree = ast.parse(SCRIPT.read_text())
function = next(
    n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "record_sample"
)
namespace = {"json": json}
exec(
    compile(ast.Module(body=[function], type_ignores=[]), str(SCRIPT), "exec"),
    namespace,
)
record_sample = namespace["record_sample"]


class Recording(unittest.TestCase):
    def check_row(self, stdout, returncode):
        run = Run(
            wall=0.1,
            cpu=0.08,
            rss_mib=2,
            returncode=returncode,
            stdout=stdout,
            stderr="diagnostic\u0090",
        )
        job = measure.Job(
            "candidate-read", ["fake", "own-read", "file", "1"], None, signature=(0, 0)
        )
        rows = []
        with tempfile.TemporaryDirectory() as d:
            journal = Path(d) / "rows.jsonl"
            row = record_sample(
                run,
                job,
                1,
                dict(phase="warm", round=0),
                journal,
                rows,
                measure.apply_metrics,
            )
            stored = [json.loads(x) for x in journal.read_text().splitlines()]
            self.assertEqual(stored, [row])
            self.assertEqual(rows, [row])
            self.assertEqual(row["stdout_text"], stdout)
            self.assertEqual(row["stderr_text"], "diagnostic\u0090")
            return row

    def test_malformed_metrics_are_journaled(self):
        self.assertIn("failure", self.check_row("{bad-json", 0))

    def test_failed_child_is_journaled(self):
        self.assertEqual(self.check_row("", 70)["failure"]["exit"], 70)

    def test_valid_string_metrics_are_journaled(self):
        self.assertEqual(
            self.check_row(
                '{"iterations":1,"nanoseconds":100,"output_bytes":0,"checksum":0}', 0
            )["ns_per_iteration"],
            100,
        )


if __name__ == "__main__":
    unittest.main()
