"""The pre-commit hook formats with this checkout's jaifmt or not at all."""
import contextlib
import io
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import dev


class HookJaifmtTests(unittest.TestCase):
    def pick(self, stamp: str | None, installed: str = "/opt/bin/jaifmt") -> str | None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "jaifmt"
            out.write_text("")
            if stamp is not None:
                dev.jaifmt_stamp(out).write_text(stamp)
            with patch.object(dev, "jaifmt_path", return_value=out), \
                    patch.object(dev, "hook_jaic", return_value=None), \
                    patch.object(dev, "hook_warn"), \
                    patch("shutil.which", return_value=installed):
                found = dev.hook_jaifmt(dev.Hook(Path(tmp)))
                return None if found is None else ("ours" if found == str(out) else found)

    def test_uses_this_checkouts_build(self):
        self.assertEqual(self.pick(str(dev.ROOT)), "ours")

    def test_never_falls_back_to_an_installed_jaifmt(self):
        self.assertIsNone(self.pick("/some/other/checkout"))


class RebuildTests(unittest.TestCase):
    """The rebuild runs jaic with this checkout's stdlib and reports why it fails."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dir = Path(self.tmp.name)
        self.out = self.dir / "jaifmt"

    def fake_jaic(self, body: str) -> Path:
        path = self.dir / "jaic"
        path.write_text("#!/bin/sh\n" + body)
        path.chmod(0o755)
        return path

    def rebuild(self, jaic: Path, **env: str):
        err = io.StringIO()
        with patch.object(dev, "jaifmt_path", return_value=self.out), \
                patch.dict(os.environ, env), contextlib.redirect_stderr(err):
            return dev.rebuild_jaifmt(jaic, quiet=True), err.getvalue()

    def test_jaic_gets_this_checkouts_stdlib(self):
        # A jaic from a deleted worktree has that worktree's stdlib path built in.
        jaic = self.fake_jaic('[ "$JAIC_STDLIB" = "%s" ] || { echo "cannot find the standard library" >&2; exit 1; }\n'
                              'echo > "$4"\n' % (dev.ROOT / "stdlib"))
        built, _ = self.rebuild(jaic, JAIC_STDLIB="/gone/stdlib")
        self.assertTrue(built)

    def test_a_failed_rebuild_shows_the_compiler_output(self):
        jaic = self.fake_jaic('echo "cannot find the standard library" >&2; exit 1\n')
        built, err = self.rebuild(jaic)
        self.assertFalse(built)
        self.assertIn("cannot find the standard library", err)

    def test_the_hook_fails_instead_of_using_a_stale_jaifmt(self):
        self.out.write_text("")
        dev.jaifmt_stamp(self.out).write_text(str(dev.ROOT))
        old = self.dir / "jaic"
        old.write_text("")
        os.utime(self.out, (1, 1))
        hook = dev.Hook(self.dir)
        err = io.StringIO()
        with patch.object(dev, "jaifmt_path", return_value=self.out), \
                patch.object(dev, "hook_jaic", return_value=old), \
                patch.object(dev, "rebuild_jaifmt", return_value=False), \
                contextlib.redirect_stderr(err):
            found = dev.hook_jaifmt(hook)
        self.assertIsNone(found)
        self.assertTrue(hook.failed)


if __name__ == "__main__":
    unittest.main()
