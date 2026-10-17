"""Linux clipboard provider closure in every native POSIX consumer job."""

from pathlib import Path
import re
import unittest


class ClipboardProviders(unittest.TestCase):
    def test_every_consumer_builds_and_searches_the_real_helper(self):
        workflow = (
            Path(__file__).resolve().parents[1] / ".github/workflows/ci.yml"
        ).read_text()
        parts = re.split(r"(?m)^  ([a-z][a-z0-9-]*):\n", workflow)[1:]
        jobs = dict(zip(parts[::2], parts[1::2]))
        for name in ("test", "stdlib-runtime", "upstream-smoke", "sanitizers"):
            with self.subTest(job=name):
                text = jobs[name]
                builder = "python3 tools/build_portable_clipboard.py --output target/clipboard-native"
                self.assertEqual(text.count(builder), 1)
                self.assertIn("$PWD/target/clipboard-native", text)
                # The helper is additional to the full third-party SDK, never a
                # nonempty search variable that accidentally suppresses its build.
                self.assertIn("python3 tools/build_native_libs.py", text)
                self.assertIn("build_native_libs.output_dir()", text)
                self.assertIn("runner.os == 'Linux'", text)


if __name__ == "__main__":
    unittest.main()
