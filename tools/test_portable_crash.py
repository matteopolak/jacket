"""The startup harness must not replace the production compiler facades."""

import contextlib
import io
import os
from pathlib import Path
import signal
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import check_portable_crash as harness


class ProductionStartup(unittest.TestCase):
    def test_supplied_library_remains_unchanged_when_sibling_sources_differ(self):
        with TemporaryDirectory() as temporary:
            parent = Path(temporary)
            root = parent / "integration"
            compiler = root / "target/debug/jaic"
            compiler.parent.mkdir(parents=True)
            compiler.write_bytes(b"unit test path; subprocess is mocked")
            library = root / "stdlib"
            for module in ["Compiler", "internal/compiler-api"]:
                own = library / module / "module.jai"
                own.parent.mkdir(parents=True)
                own.write_text("owned :: 17;\n")
                peer = parent / "compiler/stdlib" / module / "module.jai"
                peer.parent.mkdir(parents=True)
                peer.write_text("different :: 42;\n")
            before = {
                str(p.relative_to(library)): p.read_bytes()
                for p in library.rglob("*.jai")
            }
            # Native outcomes are mocked only for the nonmutation property. The
            # integration runner separately executes real ON/OFF crash programs.
            outcomes = [
                subprocess.CompletedProcess([], 0, "", ""),
                subprocess.CompletedProcess([], -signal.SIGSEGV, "", ""),
                subprocess.CompletedProcess([], 0, "", ""),
                subprocess.CompletedProcess(
                    [], -signal.SIGSEGV, "", "crash signal 11\nnative stack:\n"
                ),
            ]
            with (
                patch.dict(os.environ, {"CARGO_TARGET_DIR": str(root / "target")}),
                patch.object(harness, "ROOT", root),
                patch(
                    "sys.argv",
                    ["check", "--jaic", str(compiler), "--stdlib", str(library)],
                ),
                patch.object(harness.subprocess, "run", side_effect=outcomes) as run,
                contextlib.redirect_stdout(io.StringIO()),
            ):
                harness.main()
            self.assertEqual(run.call_count, 4)
            self.assertEqual(
                before,
                {
                    str(p.relative_to(library)): p.read_bytes()
                    for p in library.rglob("*.jai")
                },
            )


if __name__ == "__main__":
    unittest.main()
