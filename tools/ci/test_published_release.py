import io
import subprocess
import tarfile
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

import published_release as pr


def done(code=0, out="", err=""):
    return subprocess.CompletedProcess([], code, out, err)


def fake_gh(
    tag="v1",
    assets="SHA256SUMS\njacket-linux-x64.tar.gz\n",
    archive_members=("jacket-linux-x64/jacket",),
    view_error=None,
):
    def run(*args):
        if args[:2] == ("release", "view") and "tagName" in args:
            return done(1, "", view_error) if view_error else done(0, tag + "\n")
        if args[:2] == ("release", "view"):
            return done(0, assets)
        if args[:2] == ("release", "download"):
            directory = Path(args[args.index("--dir") + 1])
            with tarfile.open(
                directory / args[args.index("--pattern") + 1], "w:gz"
            ) as tar:
                for name in archive_members:
                    info = tarfile.TarInfo(name)
                    tar.addfile(info, io.BytesIO(b""))
            return done()
        raise AssertionError(args)

    return run


class Check(unittest.TestCase):
    def check(self, run, platform=None):
        with tempfile.TemporaryDirectory() as scratch, redirect_stdout(io.StringIO()):
            return pr.check("o/r", platform, run=run, scratch=scratch)

    def test_no_release_yet_is_a_skip(self):
        self.assertEqual(
            self.check(fake_gh(view_error="release not found")), (False, None)
        )

    def test_other_gh_failures_fail(self):
        with self.assertRaises(SystemExit), mock.patch("sys.stderr"):
            self.check(fake_gh(view_error="HTTP 502"))

    def test_old_release_without_platform_archives_is_a_skip(self):
        self.assertEqual(
            self.check(fake_gh(assets="SHA256SUMS\nold.zip\n")), (False, None)
        )

    def test_published_release(self):
        self.assertEqual(self.check(fake_gh()), (True, None))

    def test_formula_needs_the_jacket_binary_in_the_archive(self):
        self.assertEqual(self.check(fake_gh(), "linux-x64"), (True, True))
        self.assertEqual(
            self.check(
                fake_gh(archive_members=("jacket-linux-x64/jaic",)), "linux-x64"
            ),
            (True, False),
        )

    def test_platform_names(self):
        self.assertEqual(pr.PLATFORMS["macOS-ARM64"], "macos-arm64")


if __name__ == "__main__":
    unittest.main()
