"""Closure isolation checks use authored files; actual CLI coverage is a separate smoke."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_installed_docs as check


class InstalledDocsIsolation(unittest.TestCase):
    def test_explicit_dependency_reuse_excludes_caches_and_keeps_internal_links(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "installed"
            (source / "astro").mkdir(parents=True)
            (source / "astro/package.json").write_text('{"name":"astro"}')
            (source / ".vite").mkdir()
            (source / ".vite/private-cache").write_text("not part of the closure")
            (source / ".bin").mkdir()
            try:
                (source / ".bin/astro").symlink_to("../astro/package.json")
            except (OSError, NotImplementedError):
                pass
            destination = root / "relocated"
            check.stage_dependencies(source, destination)
            self.assertFalse((destination / ".vite").exists())
            self.assertEqual((destination / "astro/package.json").read_bytes(), (source / "astro/package.json").read_bytes())
            if (source / ".bin/astro").is_symlink():
                self.assertEqual((destination / ".bin/astro").resolve(), (destination / "astro/package.json").resolve())

    def test_dependency_links_cannot_smuggle_checkout_or_external_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "installed"
            (source / "astro").mkdir(parents=True)
            (source / "astro/package.json").write_text('{}')
            outside = root / "outside"
            outside.write_text("not installed")
            try:
                (source / "escape").symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("platform does not permit creating a symbolic link")
            with self.assertRaisesRegex(ValueError, "escapes installed closure"):
                check.stage_dependencies(source, root / "relocated")
            self.assertFalse((root / "relocated").exists())

    def test_environment_removes_checkout_overrides_and_uses_a_file_uri(self):
        with tempfile.TemporaryDirectory(prefix="installed docs ") as temporary:
            root = Path(temporary).resolve()
            with mock.patch.dict(os.environ, {"JACKET_DOCS_BUILDER": "checkout", "JACKET_DOC_EXPORT": "old", "JACKET_DOC_BASE": "/old/", "NODE_OPTIONS": "--require=unrelated", "NODE_PATH": "checkout-dependencies", "PATH": "prior", "JAIC_STDLIB": "owned-stdlib"}):
                environment = check.installed_environment(root / "bin", root / "record node.mjs")
            self.assertNotIn("JACKET_DOCS_BUILDER", environment)
            self.assertNotIn("JACKET_DOC_EXPORT", environment)
            self.assertNotIn("JACKET_DOC_BASE", environment)
            self.assertNotIn("NODE_PATH", environment)
            self.assertEqual(environment["NODE_OPTIONS"], "--import=" + (root / "record node.mjs").as_uri())
            self.assertIn("%20", environment["NODE_OPTIONS"])
            self.assertEqual(environment["PATH"], str(root / "bin") + os.pathsep + "prior")
            self.assertEqual(environment["JAIC_STDLIB"], "owned-stdlib")


if __name__ == "__main__":
    unittest.main()
