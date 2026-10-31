import argparse
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


from toolkit import download, llvm
from toolkit.cli import add_jaic_argument, default_jaic
from toolkit.hashing import sha256_bytes, sha256_file
from toolkit.process import run, run_expect
from toolkit.host import brew_prefix, exe_name, host_cpu, host_os
from toolkit.repo import ROOT, cargo_binary, cargo_target_dir, jaifmt_path, stdlib_env

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
            with mock.patch("toolkit.repo._metadata_target_dir", return_value=Path("/cfg/target")):
                self.assertEqual(cargo_target_dir(), Path("/cfg/target"))

    def test_jaifmt_path_follows_the_target_dir(self):
        with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "/x/y"}):
            self.assertEqual(jaifmt_path().parent, Path("/x/y"))
            self.assertTrue(jaifmt_path().name.startswith("jaifmt"))

    def test_cargo_binary(self):
        with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "/x/y"}):
            self.assertEqual(cargo_binary("jaic", "release"), Path("/x/y/release") / exe_name("jaic"))
            self.assertEqual(cargo_binary("jaic", triple="x86_64-pc-windows-msvc"),
                             Path("/x/y/x86_64-pc-windows-msvc/debug/jaic.exe"))
            self.assertEqual(cargo_binary("jaic", triple="aarch64-apple-darwin").name, "jaic")

    def test_host_names(self):
        self.assertIn(host_os(), {"macos", "linux", "windows"})
        self.assertIn(host_cpu(), {"arm64", "x64"})
        self.assertEqual(exe_name("a", "windows"), "a.exe")
        self.assertEqual(exe_name("a", "linux"), "a")

    def test_brew_prefix(self):
        brew_prefix.cache_clear()
        with mock.patch("toolkit.host.host_os", return_value="linux"):
            self.assertIsNone(brew_prefix())
        brew_prefix.cache_clear()
        with mock.patch("toolkit.host.host_os", return_value="macos"), mock.patch.dict(os.environ, {"HOMEBREW_PREFIX": "/h"}):
            self.assertEqual(brew_prefix(), Path("/h"))
        brew_prefix.cache_clear()

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


class Llvm(unittest.TestCase):
    def test_version_env_derives_the_variable_name(self):
        env = llvm.version_env("31.4.2")
        self.assertEqual(env["LLVM_MAJOR"], "31")
        self.assertEqual(env["LLVM_SYS_PREFIX_VAR"], "LLVM_SYS_314_PREFIX")

    def test_ci_environment_wins_over_the_pin(self):
        with mock.patch.dict(os.environ, {"LLVM_MAJOR": "99", "LLVM_SYS_PREFIX_VAR": "LLVM_SYS_991_PREFIX"}):
            self.assertEqual(llvm.major(), "99")
            self.assertEqual(llvm.sys_prefix_var(), "LLVM_SYS_991_PREFIX")

    def test_configured_prefix_prefers_the_sys_variable_then_llvm_prefix(self):
        env = {"LLVM_SYS_PREFIX_VAR": "LLVM_SYS_991_PREFIX", "LLVM_SYS_991_PREFIX": "/a", "LLVM_PREFIX": "/b"}
        with mock.patch.dict(os.environ, env):
            self.assertEqual(llvm.configured_prefix(), Path("/a"))
            del os.environ["LLVM_SYS_991_PREFIX"]
            self.assertEqual(llvm.configured_prefix(), Path("/b"))

    def test_tool_is_found_in_the_configured_prefix(self):
        with tempfile.TemporaryDirectory() as tmp:
            tool = Path(tmp) / "bin" / exe_name("llvm-symbolizer")
            tool.parent.mkdir()
            tool.write_text("")
            # Name a variable nothing sets, so the caller's own LLVM_SYS_*_PREFIX cannot win.
            env = {"LLVM_SYS_PREFIX_VAR": "LLVM_SYS_991_PREFIX", "LLVM_PREFIX": tmp}
            with mock.patch.dict(os.environ, env):
                os.environ.pop("LLVM_SYS_991_PREFIX", None)
                self.assertEqual(llvm.tool("llvm-symbolizer"), str(tool))


if __name__ == "__main__":
    unittest.main()
