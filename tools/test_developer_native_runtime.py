import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import developer_native_runtime as runtime


class RuntimeNamesTest(unittest.TestCase):
    def test_follows_the_spelling_the_helper_records(self):
        recorded = [
            "@rpath/libclang.13.dylib",
            "/usr/lib/libSystem.B.dylib",
            "@rpath/libclang.13.dylib",
        ]
        with mock.patch.object(runtime, "dependencies", return_value=recorded):
            self.assertEqual(runtime.runtime_names(Path("helper")), ["libclang.13.dylib"])


if __name__ == "__main__":
    unittest.main()
