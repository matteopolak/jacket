import argparse
import io
import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock


import json
import tarfile
import zipfile

from toolkit import download, llvm
from toolkit.lsp import FrameDecoder, FrameError, encode
from toolkit.cli import add_jaic_argument, default_jaic
from toolkit.hashing import sha256_bytes, sha256_file
from toolkit.process import output, run, run_expect
from toolkit.host import brew_prefix, exe_name, host_cpu, host_os, require_tool
from toolkit.repo import ROOT, cargo_binary, cargo_target_dir, git_files, jaifmt_path, stdlib_env

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

    def test_git_files(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            (root / "a.rs").write_text("")
            (root / "b.py").write_text("")
            subprocess.run(["git", "add", "a.rs"], cwd=root, check=True)
            self.assertEqual(git_files(root=root), ["a.rs"])
            self.assertEqual(git_files("*.py", root=root), [])
            self.assertEqual(sorted(git_files(root=root, untracked=True)), ["a.rs", "b.py"])
            self.assertEqual(git_files("*.py", root=root, untracked=True), ["b.py"])


class RunTree(unittest.TestCase):
    def test_timeout_kills_descendants(self):
        # The child starts a grandchild that would write the marker after 3 s, then waits.
        with tempfile.TemporaryDirectory() as tmp:
            marker = Path(tmp) / "marker"
            grandchild = f"import time, pathlib; time.sleep(3); pathlib.Path({str(marker)!r}).write_text('x')"
            script = f"import subprocess, sys, time; subprocess.Popen([sys.executable, '-c', {grandchild!r}]); time.sleep(30)"
            with self.assertRaises(subprocess.TimeoutExpired):
                run([PY, "-c", script], echo=False, timeout=1, capture=True)
            time.sleep(3.5)
            self.assertFalse(marker.exists())


class Output(unittest.TestCase):
    def test_strips_stdout(self):
        self.assertEqual(output(PY, "-c", "print(' hi ')"), "hi")

    def test_missing_program_is_empty(self):
        self.assertEqual(output("/nonexistent/program"), "")

    def test_timeout_is_empty(self):
        self.assertEqual(output(PY, "-c", "import time; time.sleep(5)", timeout=0.2), "")


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

    def test_pinned_to_copies_the_cached_file(self):
        target = Path(self.tmp.name) / "out.bin"
        with mock.patch.object(download, "fetch", return_value=self.DATA):
            download.pinned_to("https://example.invalid/f", sha256_bytes(self.DATA), target)
        self.assertEqual(target.read_bytes(), self.DATA)

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


class GitHubTokenTests(unittest.TestCase):
    def request(self, url, token):
        env = {"GITHUB_TOKEN": token} if token else {}
        with mock.patch.dict(os.environ, env, clear=False):
            if not token:
                os.environ.pop("GITHUB_TOKEN", None)
            return download.request(url)

    def test_github_requests_carry_the_job_token(self):
        for url in (
            "https://github.com/o/r/archive/abc.tar.gz",
            "https://codeload.github.com/o/r/tar.gz/abc",
            "https://api.github.com/repos/o/r",
        ):
            self.assertEqual(self.request(url, "t0k").get_header("Authorization"), "Bearer t0k", url)

    def test_other_hosts_and_no_token_send_nothing(self):
        self.assertIsNone(self.request("https://example.com/f", "t0k").get_header("Authorization"))
        self.assertIsNone(self.request("https://github.com/o/r", "").get_header("Authorization"))
        # A look-alike host is not GitHub.
        self.assertIsNone(self.request("https://github.com.evil.test/f", "t0k").get_header("Authorization"))

    def test_redirect_off_github_drops_the_token(self):
        # Release assets redirect to signed storage URLs, which refuse an extra Authorization.
        original = self.request("https://github.com/o/r/releases/download/v1/a.tgz", "t0k")
        handler = download.GitHubRedirect()
        for target, kept in (
            ("https://codeload.github.com/o/r/tar.gz/abc", True),
            ("https://objects.githubusercontent.com/x?sig=1", False),
        ):
            moved = handler.redirect_request(original, None, 302, "Found", {}, target)
            self.assertEqual(moved.get_header("Authorization") is not None, kept, target)


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
            with self.assertRaisesRegex(urllib.error.HTTPError, "404: no for https://example.invalid/f"):
                download.fetch("https://example.invalid/f")
        sleep.assert_not_called()


class StreamedResponse(io.BytesIO):
    """What `urlopen` returns, for a body that is streamed: a file with headers and context management."""

    def __init__(self, body: bytes, announced: int | None = None):
        super().__init__(body)
        self.headers = {} if announced is None else {"Content-Length": str(announced)}


class FetchTo(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / "deep" / "archive.bin"

    def test_streams_to_the_path_and_leaves_no_partial_file(self):
        with mock.patch("urllib.request.urlopen", return_value=StreamedResponse(b"abc", 3)):
            download.fetch_to("https://example.invalid/f", self.path)
        self.assertEqual(self.path.read_bytes(), b"abc")
        self.assertEqual(os.listdir(self.path.parent), ["archive.bin"])

    def test_transient_failures_are_retried(self):
        import urllib.error

        broken = urllib.error.HTTPError("https://example.invalid/f", 503, "down", {}, None)
        truncated = StreamedResponse(b"ab", 3)
        with (
            mock.patch("urllib.request.urlopen", side_effect=[broken, truncated, StreamedResponse(b"abc", 3)]),
            mock.patch("time.sleep") as sleep,
        ):
            download.fetch_to("https://example.invalid/f", self.path)
        self.assertEqual(sleep.call_count, 2)
        self.assertEqual(self.path.read_bytes(), b"abc")

    def test_two_downloads_of_one_path_use_different_partial_files(self):
        names = []
        real = shutil.copyfileobj

        def spy(source, out, *rest):
            names.append(out.name)
            return real(source, out, *rest)

        with (
            mock.patch("urllib.request.urlopen", side_effect=lambda *a, **k: StreamedResponse(b"abc", 3)),
            mock.patch("shutil.copyfileobj", spy),
        ):
            download.fetch_to("https://example.invalid/f", self.path)
            download.fetch_to("https://example.invalid/f", self.path)
        self.assertEqual(len(set(names)), 2)
        self.assertTrue(all(name.endswith(".part") for name in names))

    def test_a_failure_leaves_nothing_behind(self):
        import urllib.error

        missing = urllib.error.HTTPError("https://example.invalid/f", 404, "no", {}, None)
        with mock.patch("urllib.request.urlopen", side_effect=missing), mock.patch("time.sleep") as sleep:
            with self.assertRaisesRegex(urllib.error.HTTPError, "404: no for https://example.invalid/f"):
                download.fetch_to("https://example.invalid/f", self.path)
        sleep.assert_not_called()
        self.assertEqual(os.listdir(self.path.parent), [])
        with mock.patch("urllib.request.urlopen", return_value=StreamedResponse(b"ab", 3)), mock.patch("time.sleep"):
            with self.assertRaises(Exception):
                download.fetch_to("https://example.invalid/f", self.path, attempts=2)
        self.assertEqual(os.listdir(self.path.parent), [])


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


class LspFraming(unittest.TestCase):
    def test_encode_counts_bytes_not_characters(self):
        header, _, body = encode({'a': 'é'}).partition(b'\r\n\r\n')
        self.assertEqual(header, b'Content-Length: %d' % len(body))
        self.assertEqual(json.loads(body), {'a': 'é'})

    def test_decoder_handles_split_and_joined_chunks(self):
        data = encode({'id': 1}) + encode({'id': 2, 'x': 'ü'})
        decoder = FrameDecoder()
        out = []
        for i in range(0, len(data), 3):
            out += decoder.push(data[i:i + 3])
        self.assertEqual([m['id'] for m in out], [1, 2])
        self.assertEqual([m['id'] for m in FrameDecoder().push(data)], [1, 2])

    def test_decoder_ignores_other_headers_and_rejects_bad_ones(self):
        body = b'{"id":7}'
        frame = b'content-type: x\r\nCONTENT-LENGTH: %d\r\n\r\n' % len(body) + body
        self.assertEqual(FrameDecoder().push(frame), [{'id': 7}])
        with self.assertRaises(FrameError):
            FrameDecoder().push(b'Foo: 1\r\n\r\n{}')
        with self.assertRaises(FrameError):
            FrameDecoder().push(b'Content-Length: x\r\n\r\n{}')



class PinnedArchive(unittest.TestCase):
    def archive(self, kind="tar"):
        buffer = io.BytesIO()
        files = {"pkg-1/include/a.h": b"header", "pkg-1/.hidden": b"dot"}
        if kind == "zip":
            with zipfile.ZipFile(buffer, "w") as pack:
                for name, data in files.items():
                    pack.writestr(name, data)
        else:
            with tarfile.open(fileobj=buffer, mode="w:gz") as pack:
                for name, data in files.items():
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    pack.addfile(info, io.BytesIO(data))
        return buffer.getvalue()

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dest = Path(self.tmp.name) / "out" / "pkg"

    def test_extracts_the_root_into_the_destination(self):
        for kind, suffix in (("tar", "tar.gz"), ("zip", "zip")):
            data = self.archive(kind)
            with mock.patch.object(download, "fetch", return_value=data):
                result = download.pinned_archive(
                    f"https://example.invalid/pkg.{suffix}", sha256_bytes(data), "pkg-1", self.dest
                )
            self.assertEqual(result, self.dest)
            self.assertEqual((self.dest / "include/a.h").read_bytes(), b"header")
            self.assertEqual((self.dest / ".hidden").read_bytes(), b"dot")

    def test_wrong_digest_exits_before_extraction(self):
        with mock.patch.object(download, "fetch", return_value=self.archive()):
            with self.assertRaises(SystemExit):
                download.pinned_archive("https://example.invalid/p.tar.gz", "0" * 64, "pkg-1", self.dest)
        self.assertFalse(self.dest.exists())

    def test_a_missing_root_is_an_error(self):
        data = self.archive()
        with mock.patch.object(download, "fetch", return_value=data):
            with self.assertRaises(SystemExit):
                download.pinned_archive("https://example.invalid/p.tar.gz", sha256_bytes(data), "other", self.dest)

    def test_cache_file_is_reused_and_verified(self):
        data = self.archive()
        cache = Path(self.tmp.name) / "cache.tar.gz"
        with mock.patch.object(download, "fetch", return_value=data) as fetch:
            download.pinned_archive("https://example.invalid/p.tar.gz", sha256_bytes(data), "pkg-1", self.dest, cache=cache)
            download.pinned_archive("https://example.invalid/p.tar.gz", sha256_bytes(data), "pkg-1", self.dest, cache=cache)
        self.assertEqual(fetch.call_count, 1)
        cache.write_bytes(b"corrupt")
        with self.assertRaises(SystemExit):
            download.pinned_archive("https://example.invalid/p.tar.gz", sha256_bytes(data), "pkg-1", self.dest, cache=cache)


class RequireTool(unittest.TestCase):
    def test_a_missing_tool_names_itself_and_the_hint(self):
        with mock.patch("shutil.which", return_value=None):
            with self.assertRaises(SystemExit) as raised:
                require_tool("x86_64-w64-mingw32-gcc", "install MinGW-w64")
        self.assertIn("x86_64-w64-mingw32-gcc", str(raised.exception))
        self.assertIn("install MinGW-w64", str(raised.exception))

    def test_a_found_tool_is_returned(self):
        with mock.patch("shutil.which", return_value="/bin/cc"):
            self.assertEqual(require_tool("cc", "hint"), "/bin/cc")


class LlvmRequire(unittest.TestCase):
    def test_a_missing_tool_names_itself_and_the_fix(self):
        with mock.patch.object(llvm, "tool", return_value=None):
            with self.assertRaises(SystemExit) as raised:
                llvm.require("llvm-ar")
        self.assertIn("llvm-ar", str(raised.exception))
        self.assertIn("LLVM_PREFIX", str(raised.exception))

    def test_a_found_tool_is_returned(self):
        with mock.patch.object(llvm, "tool", return_value="/x/llvm-ar"):
            self.assertEqual(llvm.require("llvm-ar"), "/x/llvm-ar")


if __name__ == "__main__":
    unittest.main()
