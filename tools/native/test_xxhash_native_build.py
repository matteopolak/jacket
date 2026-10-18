"""Offline build-policy tests; original tiny inputs are not native/hash evidence."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import build_native_libs as builder


class XxhashBuildTests(unittest.TestCase):
    def setUp(self):
        temporary = builder.ROOT / "target/xxhash-build-unit"
        temporary.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=temporary)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        self.work = self.root / "work"
        self.work.mkdir()
        self.header = (
            b"#ifdef _MSC_VER\n#   include <intrin.h>\n"
            b"    /* MSVC doesn't do a good job with the mull detection. */\n"
            b"#   define XXH_mult32to64 __emulu\n#else\n"
            b"#   define XXH_mult32to64(x, y) ((U64)((x) & 0xFFFFFFFF) * (U64)((y) & 0xFFFFFFFF))\n#endif\n"
        )
        files = {
            "xxhash.c": b"/* original policy-test unit */\n",
            "xxhash.h": b"/* original policy-test declarations */\n",
            "xxh3.h": self.header,
            "LICENSE": b"original test input\n",
        }
        for name, content in files.items():
            (self.source / name).write_bytes(content)
        self.spec = {
            "revision": "v0.7.0",
            "files": {
                name: hashlib.sha256(content).hexdigest()
                for name, content in files.items()
            },
        }
        manifest = self.root / "manifest.json"
        manifest.write_text(json.dumps({"sources": {"xxhash070": self.spec}}))
        manifest_patch = patch.object(builder, "MANIFEST", manifest)
        self.addCleanup(manifest_patch.stop)
        manifest_patch.start()

    def test_target_guard_changes_only_the_build_copy(self):
        # Calling compile_units exercises the actual production selection path.
        with patch.object(builder.subprocess, "run") as command:
            objects = builder.compile_units(
                "xxhash070",
                {"units": ["xxhash.c"]},
                self.source,
                self.work,
                "windows-arm64",
            )
        unit = Path(command.call_args.args[0][-3])
        self.assertNotEqual(unit.parent, self.source)
        self.assertIn(
            b"defined(_M_IX86) || defined(_M_X64)",
            (unit.parent / "xxh3.h").read_bytes(),
        )
        self.assertEqual((self.source / "xxh3.h").read_bytes(), self.header)
        self.assertEqual(unit.read_bytes(), (self.source / "xxhash.c").read_bytes())
        self.assertEqual(objects, [self.work / "xxhash.obj"])

    def test_other_platforms_compile_the_original_source(self):
        for platform in [
            "windows-x64",
            "windows-x64-mingw",
            "windows-arm64-mingw",
            "linux-x64",
            "linux-arm64",
            "macos-x64",
            "macos-arm64",
        ]:
            with (
                self.subTest(platform=platform),
                patch.object(builder.subprocess, "run") as command,
            ):
                builder.compile_units(
                    "xxhash070",
                    {"units": ["xxhash.c"]},
                    self.source,
                    self.work,
                    platform,
                )
                self.assertEqual(
                    Path(command.call_args.args[0][-3]), self.source / "xxhash.c"
                )

    def test_corrupt_input_fails_before_compilation(self):
        (self.source / "xxhash.h").write_bytes(b"corrupt")
        with patch.object(builder.subprocess, "run") as command:
            with self.assertRaisesRegex(RuntimeError, "sha256"):
                builder.compile_units(
                    "xxhash070",
                    {"units": ["xxhash.c"]},
                    self.source,
                    self.work,
                    "windows-arm64",
                )
            command.assert_not_called()

    def test_verified_but_unrecognized_header_requires_review(self):
        changed = self.header.replace(b"#ifdef _MSC_VER", b"#if defined(_MSC_VER)")
        (self.source / "xxh3.h").write_bytes(changed)
        self.spec["files"]["xxh3.h"] = hashlib.sha256(changed).hexdigest()
        builder.MANIFEST.write_text(json.dumps({"sources": {"xxhash070": self.spec}}))
        with patch.object(builder.subprocess, "run") as command:
            with self.assertRaisesRegex(RuntimeError, "intrinsic guard"):
                builder.compile_units(
                    "xxhash070",
                    {"units": ["xxhash.c"]},
                    self.source,
                    self.work,
                    "windows-arm64",
                )
            command.assert_not_called()

    def test_changed_revision_requires_review(self):
        self.spec["revision"] = "v-next"
        builder.MANIFEST.write_text(json.dumps({"sources": {"xxhash070": self.spec}}))
        with patch.object(builder.subprocess, "run") as command:
            with self.assertRaisesRegex(RuntimeError, "revision changed"):
                builder.compile_units(
                    "xxhash070",
                    {"units": ["xxhash.c"]},
                    self.source,
                    self.work,
                    "windows-arm64",
                )
            command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
