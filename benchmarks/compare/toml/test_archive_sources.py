"""Owned source archives must match their compiled build receipt without executing clients."""

import hashlib, json, subprocess, sys, tarfile, tempfile, unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("archive_sources.py")
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


class SourceArchive(unittest.TestCase):
    def fixture(self, root):
        (root / "stdlib").mkdir()
        source = root / "stdlib/example.jai"
        source.write_text("x :: 1;\n")
        compiler = root / "compiler"
        compiler.write_bytes(b"not executed")
        output = root / "build"
        output.mkdir()
        binary = output / "jai-batch"
        binary.write_bytes(b"also not executed")
        receipt = dict(
            sources={"stdlib/example.jai": sha(source)},
            compiler_sha256=sha(compiler),
            binary_sha256=sha(binary),
            command=[str(compiler)],
        )
        path = output / "build.json"
        path.write_text(json.dumps(receipt))
        return path, receipt

    def run_cli(self, root, path, *extra):
        return subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--root",
                str(root),
                "--build",
                str(path),
                *extra,
            ],
            capture_output=True,
            text=True,
        )

    def test_exact_owned_contents(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            path, _ = self.fixture(root)
            run = self.run_cli(root, path)
            self.assertEqual(run.returncode, 0, run.stderr)
            with tarfile.open(path.parent / "frozen-owned-sources.tar.gz") as archive:
                self.assertEqual(archive.getnames(), ["stdlib/example.jai"])
                self.assertEqual(
                    archive.extractfile("stdlib/example.jai").read(), b"x :: 1;\n"
                )

    def test_changed_source_or_binary_rejected(self):
        for relative in ["stdlib/example.jai", "build/jai-batch", "compiler"]:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as d:
                root = Path(d)
                path, _ = self.fixture(root)
                (root / relative).write_bytes(b"changed")
                run = self.run_cli(root, path)
                self.assertNotEqual(run.returncode, 0)
                self.assertFalse((path.parent / "frozen-owned-sources.tar.gz").exists())

    def test_escaping_receipt_path_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            path, receipt = self.fixture(root)
            receipt["sources"] = {"../outside.jai": "unused"}
            path.write_text(json.dumps(receipt))
            run = self.run_cli(root, path)
            self.assertNotEqual(run.returncode, 0)
            self.assertFalse((path.parent / "frozen-owned-sources.tar.gz").exists())

    def test_existing_archive_never_overwritten(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            path, _ = self.fixture(root)
            archive = path.parent / "frozen-owned-sources.tar.gz"
            archive.write_bytes(b"keep")
            run = self.run_cli(root, path)
            self.assertNotEqual(run.returncode, 0)
            self.assertEqual(archive.read_bytes(), b"keep")


if __name__ == "__main__":
    unittest.main()
