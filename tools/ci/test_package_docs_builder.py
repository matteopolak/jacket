"""Check authored archive fixtures and actual Git checkout bytes without npm or Astro."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import package_docs_builder as builder


def source(root):
    root.mkdir()
    package = {"name": "owned-docs-builder", "version": "fixture", "private": True}
    (root / "package.json").write_text(json.dumps(package))
    (root / "package-lock.json").write_text(json.dumps({
        "name": package["name"], "version": package["version"], "lockfileVersion": 3,
        "packages": {"": package},
    }))
    for name in ("astro.config.mjs", "tsconfig.json", "tsconfig.worker.json", ".node-version"):
        (root / name).write_text("inert fixture\n")
    for name in ("src/pages/index.astro", "src/generated/doc-site.d.ts", "schema/doc-site.schema.json",
                 "scripts/build.mjs", "scripts/check-site.mjs", "scripts/generate-types.mjs", "public/fonts/owned.woff2"):
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"inert fixture\n")
    return root


class DocsBuilderPackageTests(unittest.TestCase):
    def test_source_closure_is_complete_relocatable_and_reproducible(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = source(base / "source")
            for name in ("node_modules/private.txt", ".astro/cache.json", "dist/index.html", "export/site.json",
                         "tests/private.json", "src/tests/private.json", "src/.env", ".env", "scripts/check-examples.mjs"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("must not ship")
            one = builder.package(root, base / "one")
            two = builder.package(root, base / "two")
            self.assertEqual(one, two)
            actual = {p.relative_to(base / "one").as_posix() for p in (base / "one").rglob("*") if p.is_file()}
            self.assertIn("tsconfig.worker.json", actual)
            self.assertIn("public/fonts/owned.woff2", actual)
            self.assertIn("src/generated/doc-site.d.ts", actual)
            self.assertIn("schema/doc-site.schema.json", actual)
            self.assertIn("scripts/build.mjs", actual)
            self.assertIn("scripts/check-site.mjs", actual)
            self.assertIn("scripts/generate-types.mjs", actual)
            self.assertNotIn("scripts/check-examples.mjs", actual)
            self.assertFalse(any("private" in name or ".env" in name for name in actual))
            self.assertEqual(actual, {entry["path"] for entry in one["files"]} | {"builder-manifest.json"})
            self.assertEqual(one["schema"], "jacket-docs-builder")
            self.assertEqual(one["schemaVersion"], 1)
            for entry in one["files"]:
                data = (base / "one" / entry["path"]).read_bytes()
                self.assertEqual(entry["sha256"], hashlib.sha256(data).hexdigest())
                self.assertEqual(entry["bytes"], len(data))
            self.assertNotIn(str(base), json.dumps(one))

    def test_manifest_writer_preserves_utf8_lf_on_windows(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = source(base / "source")
            original = Path.write_text

            def windows_write(path, text, encoding=None, errors=None, newline=None):
                return original(path, text, encoding=encoding or "cp1252", errors=errors,
                                newline="\r\n" if newline is None else newline)

            with mock.patch.object(Path, "write_text", windows_write):
                manifest = builder.package(root, base / "output")
            expected = (json.dumps(manifest, indent=2) + "\n").encode("utf-8")
            self.assertEqual((base / "output/builder-manifest.json").read_bytes(), expected)

    def test_actual_source_closure_and_spec_keep_git_bytes_with_autocrlf(self):
        # Use a separate Git index and no commits. Git blobs, not the host checkout's
        # newline convention, are the authority for every actual packaged input.
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        _, files = builder.inventory(builder.ROOT / "website/docs")
        paths = ["website/docs/" + name for name in files]
        paths.append("crates/jacket/src/doc/coloring/spec.json")

        def git(directory, *args, data=None):
            return subprocess.run(["git", "-C", str(directory), *args], input=data,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  env=env, check=True).stdout

        authority = {name: git(builder.ROOT, "cat-file", "blob", "HEAD:" + name) for name in paths}
        authority["website/docs/public/future-binary.dat"] = b"binary\x00line\nnext\r\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            git(root, "init", "--quiet")
            git(root, "config", "core.autocrlf", "true")
            entries = {".gitattributes": (builder.ROOT / ".gitattributes").read_bytes(), **authority}
            for name, data in entries.items():
                digest = git(root, "hash-object", "-w", "--stdin", data=data).decode().strip()
                git(root, "update-index", "--add", "--cacheinfo", "100644", digest, name)
            git(root, "checkout-index", "--all", "--force")
            for name, data in authority.items():
                self.assertEqual((root / name).read_bytes(), data, name)

    def test_missing_inputs_and_lock_mismatch_leave_destination_absent(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = source(base / "source")
            for required in ("package-lock.json", "schema/doc-site.schema.json", "src/generated/doc-site.d.ts"):
                path = root / required
                data = path.read_bytes()
                path.unlink()
                with self.assertRaisesRegex(ValueError, "missing"):
                    builder.package(root, base / "output")
                self.assertFalse((base / "output").exists())
                path.write_bytes(data)
            lock = json.loads((root / "package-lock.json").read_text())
            lock["version"] = "mismatch"
            (root / "package-lock.json").write_text(json.dumps(lock))
            with self.assertRaisesRegex(ValueError, "lock"):
                builder.package(root, base / "output")
            self.assertFalse((base / "output").exists())

    def test_dependency_edits_require_a_matching_frozen_lockfile(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = source(base / "source")
            metadata = json.loads((root / "package.json").read_text())
            metadata["dependencies"] = {"owned-package": "exact-pin"}
            (root / "package.json").write_text(json.dumps(metadata))
            with self.assertRaisesRegex(ValueError, "lock"):
                builder.package(root, base / "output")
            self.assertFalse((base / "output").exists())

    def test_symlinks_and_existing_destination_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = source(base / "source")
            outside = base / "private.txt"
            outside.write_text("must not ship")
            link = root / "public" / "linked.txt"
            try:
                link.symlink_to(outside)
            except OSError:
                self.skipTest("symlinks unavailable on this host")
            with self.assertRaisesRegex(ValueError, "symlink"):
                builder.package(root, base / "output")
            self.assertFalse((base / "output").exists())
            link.unlink()
            (base / "output").mkdir()
            (base / "output" / "unrelated.txt").write_text("preserve")
            with self.assertRaisesRegex(ValueError, "exists"):
                builder.package(root, base / "output")
            self.assertEqual((base / "output" / "unrelated.txt").read_text(), "preserve")


if __name__ == "__main__":
    unittest.main()
