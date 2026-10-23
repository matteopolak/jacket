"""`stamped` in ci_native_deps.sh rebuilds exactly when the inputs, stamp or output change."""

import re
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("ci_native_deps.sh")


class Stamps(unittest.TestCase):
    def setUp(self):
        text = SCRIPT.read_text()
        self.function = re.search(r"^stamped\(\) \{.*?^\}\n", text, re.S | re.M).group(0)

    def test_rebuilds_only_when_something_changed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "in").mkdir()
            (root / "in/f").write_text("a")
            (root / "out").mkdir()
            script = f"""set -euo pipefail
os=Linux
{self.function}
git init -q
git add in
stamped x out in -- echo BUILD
stamped x out in -- echo BUILD
echo b > in/f
git add in
stamped x out in -- echo BUILD
rmdir out
stamped x out in -- echo BUILD
"""
            result = subprocess.run(
                ["bash", "-c", script], cwd=root, text=True, capture_output=True
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            lines = result.stdout.splitlines()
            self.assertEqual(lines.count("BUILD"), 3, result.stdout)
            self.assertEqual(sum("unchanged inputs" in line for line in lines), 1)


if __name__ == "__main__":
    unittest.main()
