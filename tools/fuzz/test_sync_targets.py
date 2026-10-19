import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class SyncTargets(unittest.TestCase):
    def test_listings_match_the_manifest(self):
        result = subprocess.run(
            [sys.executable, str(HERE / "sync_targets.py"), "--check"], capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
