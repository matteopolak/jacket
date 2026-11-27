import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import test_times

XML = """<?xml version="1.0"?><testsuites><testsuite name="m">
<testcase name="a" time="1.5"/><testcase name="b&lt;c&gt;" time="0.25"/><testcase name="s" time="9"><skipped/></testcase>
</testsuite></testsuites>"""


class TestTimes(unittest.TestCase):
    def test_times_add_up_across_files_and_skips_are_left_out(self):
        with tempfile.TemporaryDirectory() as tmp:
            for mode in ("native", "interp"):
                Path(tmp, mode + ".xml").write_text(XML)
            totals = test_times.totals(sorted(Path(tmp).glob("*.xml")))
        self.assertEqual(totals, {"a": 3.0, "b<c>": 0.5})


if __name__ == "__main__":
    unittest.main()
