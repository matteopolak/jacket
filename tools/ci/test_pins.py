import tempfile
import unittest
from pathlib import Path

import pins


class Pins(unittest.TestCase):
    def test_the_workflows_match_the_manifest(self):
        self.assertEqual(pins.run(pins.ROOT, pins.load(), write=False), [])

    def test_sync_rewrites_only_the_pinned_text(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".github/workflows").mkdir(parents=True)
            (root / ".github/actions/a").mkdir(parents=True)
            workflow = root / ".github/workflows/w.yml"
            workflow.write_text(
                "node-version: '22'\nenv:\n  LLVM_VERSION: 1.2.3\n  llvm: LLVM-1.2.3-Linux-X64.tar.xz\n"
            )
            (root / ".github/actions/a/action.yml").write_text(
                "python-version: '3.1'\n"
            )
            manifest = {name: "9" for name in pins.RULES}
            manifest["llvm"] = "9.9.9"
            manifest["node"] = "24"
            manifest["python"] = "3.14"
            problems = pins.run(root, manifest, write=False)
            self.assertTrue(any("`node` differs" in p for p in problems))
            self.assertTrue(any("no workflow mentions `ruff`" in p for p in problems))
            pins.run(root, manifest, write=True)
            self.assertEqual(
                workflow.read_text(),
                "node-version: '24'\nenv:\n  LLVM_VERSION: 9.9.9\n  llvm: LLVM-9.9.9-Linux-X64.tar.xz\n",
            )
            self.assertEqual(
                (root / ".github/actions/a/action.yml").read_text(),
                "python-version: '3.14'\n",
            )

    def test_unknown_pins_are_reported(self):
        manifest = pins.load() | {"zig": "0.1"}
        self.assertTrue(
            any(
                "`zig` has no rule" in p
                for p in pins.run(pins.ROOT, manifest, write=False)
            )
        )


if __name__ == "__main__":
    unittest.main()
