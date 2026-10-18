import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from toolkit.hashing import sha256_bytes, sha256_file
from toolkit.process import run_expect
from toolkit.repo import ROOT, cargo_target_dir, stdlib_env

PY = sys.executable


class Hashing(unittest.TestCase):
    def test_file_and_bytes_agree(self):
        data = b"abc" * 1_000_000
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "f"
            path.write_bytes(data)
            self.assertEqual(sha256_file(path), sha256_bytes(data))
            self.assertEqual(sha256_file(str(path)), sha256_bytes(data))

    def test_known_digest(self):
        self.assertEqual(sha256_bytes(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")


class Repo(unittest.TestCase):
    def test_root_is_the_checkout(self):
        self.assertTrue((ROOT / "tools" / "toolkit").is_dir())

    def test_cargo_target_dir(self):
        with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "/x/y"}):
            self.assertEqual(cargo_target_dir(), Path("/x/y"))
        with mock.patch.dict(os.environ):
            os.environ.pop("CARGO_TARGET_DIR", None)
            self.assertEqual(cargo_target_dir(), ROOT / "target")

    def test_stdlib_env(self):
        env = stdlib_env(EXTRA="1")
        self.assertEqual(env["JAIC_STDLIB"], str(ROOT / "stdlib"))
        self.assertEqual(env["EXTRA"], "1")


class RunExpect(unittest.TestCase):
    def test_passes(self):
        result = run_expect([PY, "-c", "print('hi')"], label="t", expected="hi\n")
        self.assertEqual(result.stdout, "hi\n")

    def test_stdout_mismatch(self):
        with self.assertRaisesRegex(RuntimeError, "^t:"):
            run_expect([PY, "-c", "print('hi')"], label="t")

    def test_unchecked_stdout(self):
        run_expect([PY, "-c", "print('hi')"], label="t", expected=None)

    def test_exit_code(self):
        run_expect([PY, "-c", "raise SystemExit(3)"], label="t", code=3)
        with self.assertRaises(RuntimeError):
            run_expect([PY, "-c", "raise SystemExit(3)"], label="t")

    def test_stderr(self):
        script = "import sys; sys.stderr.write('e')"
        with self.assertRaises(RuntimeError):
            run_expect([PY, "-c", script], label="t")
        run_expect([PY, "-c", script], label="t", stderr=None)
        run_expect([PY, "-c", script], label="t", stderr="e")

    def test_env_and_cwd(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = run_expect(
                [PY, "-c", "import os; print(os.environ['K'], os.getcwd())"],
                label="t",
                expected=None,
                env=dict(os.environ, K="v"),
                cwd=tmp,
            )
            self.assertTrue(result.stdout.startswith("v "))


if __name__ == "__main__":
    unittest.main()
