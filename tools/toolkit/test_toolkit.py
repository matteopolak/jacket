import argparse
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

# tools/, for folder-qualified imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit import download
from toolkit.cli import add_jaic_argument, default_jaic
from toolkit.hashing import sha256_bytes, sha256_file
from toolkit.process import run, run_expect
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



class PinnedDownloadTests(unittest.TestCase):
    DATA = b"pinned bytes"

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        patcher = mock.patch.dict(os.environ, {"XDG_CACHE_HOME": self.tmp.name})
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_downloads_once_then_reads_the_cache(self):
        with mock.patch.object(download, "fetch", return_value=self.DATA) as fetch:
            first = download.pinned("https://example.invalid/f", sha256_bytes(self.DATA), "f.bin")
            second = download.pinned("https://example.invalid/f", sha256_bytes(self.DATA), "f.bin")
        self.assertEqual(fetch.call_count, 1)
        self.assertEqual(first, second)
        self.assertEqual(first.read_bytes(), self.DATA)

    def test_wrong_digest_exits_and_drops_the_file(self):
        with mock.patch.object(download, "fetch", return_value=self.DATA):
            with self.assertRaises(SystemExit):
                download.pinned("https://example.invalid/f", "0" * 64, "f.bin")
        self.assertFalse((download.cache_dir() / "f.bin").exists())

    def test_rate_limit_is_retried(self):
        import urllib.error

        busy = urllib.error.HTTPError("https://example.invalid/f", 429, "busy", {"Retry-After": "1"}, None)
        response = mock.MagicMock()
        response.__enter__.return_value.read.return_value = self.DATA
        with mock.patch("urllib.request.urlopen", side_effect=[busy, response]), mock.patch("time.sleep") as sleep:
            path = download.pinned("https://example.invalid/f", sha256_bytes(self.DATA), "f.bin")
        sleep.assert_called_once_with(1)
        self.assertEqual(path.read_bytes(), self.DATA)


class Run(unittest.TestCase):
    def test_captures_output(self):
        result = run([PY, "-c", "print(1)"], capture=True)
        self.assertEqual(result.stdout, "1\n")

    def test_raises_on_failure_unless_unchecked(self):
        with self.assertRaises(subprocess.CalledProcessError):
            run([PY, "-c", "raise SystemExit(3)"])
        self.assertEqual(run([PY, "-c", "raise SystemExit(3)"], check=False).returncode, 3)


class JaicArgument(unittest.TestCase):
    def test_default_and_override(self):
        parser = argparse.ArgumentParser()
        add_jaic_argument(parser)
        self.assertEqual(parser.parse_args([]).jaic, default_jaic())
        self.assertEqual(parser.parse_args(["--jaic", "x/jaic"]).jaic, Path("x/jaic"))

    def test_required(self):
        parser = argparse.ArgumentParser()
        add_jaic_argument(parser, required=True)
        with mock.patch("sys.stderr"), self.assertRaises(SystemExit):
            parser.parse_args([])


class Fetch(unittest.TestCase):
    def test_server_errors_are_retried_and_not_found_is_not(self):
        import urllib.error

        broken = urllib.error.HTTPError("https://example.invalid/f", 503, "down", {}, None)
        response = mock.MagicMock()
        response.__enter__.return_value.read.return_value = b"ok"
        with mock.patch("urllib.request.urlopen", side_effect=[broken, response]), mock.patch("time.sleep") as sleep:
            self.assertEqual(download.fetch("https://example.invalid/f"), b"ok")
        sleep.assert_called_once_with(5)
        missing = urllib.error.HTTPError("https://example.invalid/f", 404, "no", {}, None)
        with mock.patch("urllib.request.urlopen", side_effect=missing), mock.patch("time.sleep") as sleep:
            with self.assertRaises(urllib.error.HTTPError):
                download.fetch("https://example.invalid/f")
        sleep.assert_not_called()


if __name__ == "__main__":
    unittest.main()
