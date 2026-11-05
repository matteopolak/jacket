"""The durable reproduction input must regenerate every reported workload byte-for-byte."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


class FrozenWorkloads(unittest.TestCase):
    def test_complete_input_and_family_counts(self):
        plan = json.loads((HERE / "frozen-followup-plan.json").read_text())
        with tempfile.TemporaryDirectory() as directory:
            command = [
                sys.executable,
                str(ROOT / "tools/stdlib/toml/workloads.py"),
                "--output",
                directory,
                "--repository-config",
                str(HERE / "fixtures/repository-config.toml"),
            ]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            paths = list(Path(directory).rglob("*.toml"))
            self.assertEqual(len(paths), 18)
            self.assertEqual(len({p.name for p in paths}), 18)
            actual = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
            self.assertEqual(actual, plan["input_sha256"])
        for family, count in [
            ("read", 9),
            ("lookup", 9),
            ("write", 9),
            ("formatter", 4),
        ]:
            self.assertEqual(
                sum(key.endswith("/" + family) for key in plan["iterations"]), count
            )
        self.assertTrue(
            all(
                type(count) is int and count > 0
                for count in plan["iterations"].values()
            )
        )


if __name__ == "__main__":
    unittest.main()
