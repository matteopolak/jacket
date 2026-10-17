"""Original pinned-download tests; scripted transport only, no server/driver."""
import errno
import hashlib
import http.client
import io
from pathlib import Path
import socket
import ssl
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import window_render_mesa_windows as mesa


class PinnedDownload(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(directory.cleanup)
        self.directory = Path(directory.name)
        self.archive = self.directory / "fixture.archive"
        self.payload = b"original deterministic pinned archive bytes"
        self.digest = hashlib.sha256(self.payload).hexdigest()
        self.sleep = self.enterContext(patch("time.sleep"))
        self.sleep.side_effect = lambda delay: self.assert_no_partial()

    def download(self):
        mesa.download_source("https://invalid.example/pinned", self.digest, self.archive)

    def assert_no_partial(self):
        self.assertEqual(list(self.directory.iterdir()), [self.archive] if self.archive.exists() else [])

    def test_reset_then_success_uses_timeout_and_fresh_partial(self):
        paths = []

        class Interrupted(io.BytesIO):
            def read(inner, size=-1):
                if inner.tell():
                    raise urllib.error.URLError(ConnectionResetError(errno.ECONNRESET, "reset"))
                return super().read(5)

        def open_response(url, *, timeout):
            self.assertEqual(url, "https://invalid.example/pinned")
            self.assertEqual(timeout, 120)
            partials = list(self.directory.iterdir())
            self.assertEqual(len(partials), 1)
            self.assertEqual(partials[0].stat().st_size, 0)
            paths.append(partials[0])
            return Interrupted(self.payload) if len(paths) == 1 else io.BytesIO(self.payload)

        with patch.object(mesa.urllib.request, "urlopen", side_effect=open_response) as opener:
            self.download()
        self.assertEqual(opener.call_count, 2)
        self.assertNotEqual(*paths)
        self.assertEqual(self.archive.read_bytes(), self.payload)
        self.sleep.assert_called_once_with(2)
        self.assert_no_partial()

    def test_transient_failures_retry_then_succeed(self):
        windows_reset = OSError("WinError 10054")
        windows_reset.winerror = 10054
        errors = [urllib.error.URLError(windows_reset), TimeoutError("timed out"),
                  urllib.error.URLError(TimeoutError("connect timeout")),
                  urllib.error.URLError(socket.gaierror(socket.EAI_AGAIN, "temporary DNS failure")),
                  http.client.RemoteDisconnected("remote closed before response"),
                  http.client.IncompleteRead(b"partial", 10)]
        errors += [urllib.error.HTTPError("https://invalid.example", code, "transient", {}, None)
                   for code in [408, 425, 429, 500, 502, 503, 504]]
        for error in errors:
            with self.subTest(error=repr(error)):
                with patch.object(mesa.urllib.request, "urlopen", side_effect=[error, io.BytesIO(self.payload)]) as opener:
                    self.download()
                self.assertEqual(opener.call_count, 2)
                self.assertEqual(self.archive.read_bytes(), self.payload)
                self.archive.unlink()
                self.assert_no_partial()

    def test_exhaustion_is_bounded_and_preserves_final_error(self):
        failure = urllib.error.URLError(ConnectionResetError(errno.ECONNRESET, "reset"))
        with patch.object(mesa.urllib.request, "urlopen", side_effect=failure) as opener:
            with self.assertRaises(urllib.error.URLError) as raised:
                self.download()
        self.assertIs(raised.exception, failure)
        self.assertEqual(opener.call_count, 4)
        self.assertEqual([call.args[0] for call in self.sleep.call_args_list], [2, 4, 8])
        self.assert_no_partial()

    def test_permanent_errors_do_not_retry(self):
        errors = [urllib.error.HTTPError("https://invalid.example", code, "permanent", {}, None)
                  for code in [400, 401, 403, 404, 501]]
        errors += [urllib.error.URLError(ssl.SSLCertVerificationError("certificate")),
                   ssl.SSLError("TLS protocol"), urllib.error.URLError("unknown reason"),
                   urllib.error.URLError(socket.gaierror(socket.EAI_NONAME, "unknown host")),
                   PermissionError(errno.EACCES, "denied"), OSError(errno.ENOSPC, "disk full")]
        for error in errors:
            with self.subTest(error=repr(error)):
                with patch.object(mesa.urllib.request, "urlopen", side_effect=error) as opener:
                    with self.assertRaises(type(error)):
                        self.download()
                self.assertEqual(opener.call_count, 1)
                self.assert_no_partial()
        self.sleep.assert_not_called()

    def test_corrupt_download_is_not_promoted_or_retried(self):
        with patch.object(mesa.urllib.request, "urlopen", return_value=io.BytesIO(b"corrupt")) as opener:
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                self.download()
        self.assertEqual(opener.call_count, 1)
        self.sleep.assert_not_called()
        self.assert_no_partial()

    def test_local_timeout_during_promotion_does_not_retry_network(self):
        with patch.object(mesa.urllib.request, "urlopen", return_value=io.BytesIO(self.payload)) as opener:
            with patch.object(Path, "replace", side_effect=TimeoutError("local filesystem timeout")):
                with self.assertRaisesRegex(TimeoutError, "local filesystem"):
                    self.download()
        self.assertEqual(opener.call_count, 1)
        self.sleep.assert_not_called()
        self.assert_no_partial()

    def test_source_tree_extracts_only_after_verified_retry(self):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w") as output:
            entry = tarfile.TarInfo("original/file.c")
            entry.size = 4
            output.addfile(entry, io.BytesIO(b"data"))
        payload = stream.getvalue()
        digest = hashlib.sha256(payload).hexdigest()
        source = ("https://invalid.example/pinned", digest, "original")
        with patch.dict(mesa.SOURCES, fixture=source):
            with patch.object(mesa.urllib.request, "urlopen",
                              side_effect=[TimeoutError("timeout"), io.BytesIO(payload)]) as opener:
                tree = mesa.source_tree("fixture", self.directory)
        self.assertEqual(opener.call_count, 2)
        self.assertEqual((tree / "file.c").read_bytes(), b"data")
        self.assertEqual(self.archive.read_bytes(), payload)
        self.assertEqual(list(self.directory.glob("*.partial")), [])

    def test_verified_durable_download_never_uses_network(self):
        self.archive.write_bytes(self.payload)
        with patch.object(mesa.urllib.request, "urlopen") as opener:
            self.download()
        opener.assert_not_called()
        self.assertEqual(self.archive.read_bytes(), self.payload)
        self.assert_no_partial()

    def test_corrupt_durable_download_fails_without_network_or_replacement(self):
        self.archive.write_bytes(b"corrupt durable evidence")
        with patch.object(mesa.urllib.request, "urlopen") as opener:
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                self.download()
        opener.assert_not_called()
        self.assertEqual(self.archive.read_bytes(), b"corrupt durable evidence")
        self.assert_no_partial()


if __name__ == "__main__":
    unittest.main()
