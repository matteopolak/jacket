"""Provider order checks; these do not execute a compiler or native program."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


def jobs(name):
    parts = re.split(r"(?m)^  ([a-z][a-z0-9-]*):\n", (ROOT / name).read_text())[1:]
    return dict(zip(parts[::2], parts[1::2]))


class DeveloperProviders(unittest.TestCase):
    def test_posix_consumers_build_real_source_relative_helpers(self):
        workflow = jobs(".github/workflows/ci.yml")
        for job in ("test", "stdlib-runtime", "upstream-smoke", "sanitizers"):
            with self.subTest(job=job):
                text = workflow[job]
                for helper in ("build_developer_bindings.py", "build_developer_debug.py", "build_generator_platform.py"):
                    self.assertEqual(text.count("python3 tools/" + helper), 1)
                if job == "test":
                    prepare = text.index("python3 tools/build_portable_runtime.py")
                    execute = text.index("cargo test --workspace")
                    self.assertLess(prepare, execute)
                if job == "upstream-smoke":
                    self.assertLess(text.index("python3 tools/build_developer_debug.py"), text.index("python3 tools/fetch_upstreams.py"))

    def test_windows_native_consumers_build_actual_windows_debug_helper(self):
        workflow = jobs(".github/workflows/windows-native.yml")
        consumers = [text for text in workflow.values() if "python -u tools/build_platform_io_runtime.py" in text]
        self.assertEqual(len(consumers), 3)
        for text in consumers:
            self.assertEqual(text.count("python -u tools/build_developer_debug.py"), 1)
            self.assertLess(text.index("python -u tools/build_platform_io_runtime.py"), text.index("python -u tools/build_developer_debug.py"))
            self.assertNotIn("python -u tools/build_developer_bindings.py", text)


if __name__ == "__main__":
    unittest.main()
