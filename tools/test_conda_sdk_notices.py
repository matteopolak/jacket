"""Native package metadata/notice controls; never load or execute SDK images."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

MODULE = Path(
    os.environ.get(
        "NATIVE_RUNTIME_TEST_MODULE",
        Path(__file__).with_name("developer_native_runtime.py"),
    )
)
SPEC = importlib.util.spec_from_file_location("notice_runtime", MODULE)
runtime = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runtime)


class CondaNotices(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.sdk, self.cache, self.output = [
            self.root / n for n in ("sdk", "cache", "out")
        ]
        self.image = self.sdk / "lib/libclang.13.dylib"
        self.image.parent.mkdir(parents=True)
        self.image.write_bytes(b"independent non-executable image identity")
        self.identity = dict(
            name="libclang13", version="23.1.2", build="default_h3065a3b_0"
        )
        self.metadata = dict(
            self.identity,
            files=["lib/libclang.13.dylib"],
            link=dict(source=str(self.cache)),
            sha256="0" * 64,
        )
        self.meta = self.sdk / "conda-meta/libclang13.json"
        self.meta.parent.mkdir()
        self.meta.write_text(json.dumps(self.metadata))
        self.index = self.cache / "info/index.json"
        self.index.parent.mkdir(parents=True)
        self.index.write_text(json.dumps(self.identity))
        (self.cache / "info/paths.json").write_text(
            json.dumps(
                dict(paths=[dict(_path="lib/libclang.13.dylib")], paths_version=1)
            )
        )
        self.notice = self.cache / "info/licenses/clang/LICENSE.TXT"
        self.notice.parent.mkdir(parents=True)
        self.notice.write_bytes(b"Independent package notice fixture\n")

    def stage(self):
        return runtime.stage_notices({"libclang.dylib": self.image}, self.output)

    def test_real_package_notice_is_copied_with_owner_provenance(self):
        entry = self.stage()["libclang.dylib"][0]
        self.assertEqual(
            (self.output / entry["staged_path"]).read_bytes(), self.notice.read_bytes()
        )
        self.assertEqual(
            entry["sha256"], hashlib.sha256(self.notice.read_bytes()).hexdigest()
        )
        self.assertEqual(entry["conda_package"], self.identity)
        self.assertEqual(
            entry["installed_metadata_sha256"],
            hashlib.sha256(self.meta.read_bytes()).hexdigest(),
        )
        self.assertIn("clang/LICENSE.TXT", entry["staged_path"])

    def test_wrong_cache_version_is_rejected(self):
        self.index.write_text(json.dumps(dict(self.identity, version="22.0.0")))
        with self.assertRaisesRegex(RuntimeError, "cache identity"):
            self.stage()

    def test_wrong_image_owner_is_rejected(self):
        (self.cache / "info/paths.json").write_text(
            json.dumps(dict(paths=[dict(_path="lib/other.dylib")]))
        )
        with self.assertRaisesRegex(RuntimeError, "does not own"):
            self.stage()

    def test_missing_notices_are_not_replaced_by_another_package_license(self):
        sidecar = self.notice.with_name("._" + self.notice.name)
        if sidecar.exists():
            self.assertEqual(
                sidecar.read_bytes()[:8], bytes.fromhex("0005160700020000")
            )
            sidecar.unlink()
        self.notice.unlink()
        (self.sdk / "LICENSE.TXT").write_text("Unrelated prefix notice")
        with self.assertRaisesRegex(RuntimeError, "no license files"):
            self.stage()

    def test_conflicting_staged_notice_is_not_overwritten(self):
        entry = self.stage()["libclang.dylib"][0]
        target = self.output / entry["staged_path"]
        target.write_bytes(b"Conflicting retained identity")
        with self.assertRaisesRegex(RuntimeError, "Conflicting staged"):
            self.stage()
        self.assertEqual(target.read_bytes(), b"Conflicting retained identity")

    def test_non_conda_top_level_notice_contract_is_preserved(self):
        self.meta.unlink()
        self.meta.parent.rmdir()
        notice = self.sdk / "LICENSE.TXT"
        notice.write_bytes(b"Independent conventional SDK notice\n")
        entry = self.stage()["libclang.dylib"][0]
        self.assertEqual(
            (self.output / entry["staged_path"]).read_bytes(), notice.read_bytes()
        )
        self.assertNotIn("conda_package", entry)

    def test_unrecognized_sidecar_is_not_silently_ignored(self):
        (self.meta.parent / "._invalid.json").write_bytes(b"not AppleDouble")
        with self.assertRaisesRegex(RuntimeError, "Unrecognized SDK metadata sidecar"):
            self.stage()


if __name__ == "__main__":
    unittest.main()
