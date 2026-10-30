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
                "node-version: '22'\nenv:\n  MESA_VERSION: 1.2.3\n"
            )
            (root / ".github/actions/a/action.yml").write_text(
                "python-version: '3.1'\n"
            )
            manifest = {name: "9" for name in pins.RULES}
            manifest["mesa"] = "9.9.9"
            manifest["node"] = "24"
            manifest["python"] = "3.14"
            problems = pins.run(root, manifest, write=False)
            self.assertTrue(any("`node` differs" in p for p in problems))
            self.assertTrue(any("no workflow mentions `ruff`" in p for p in problems))
            pins.run(root, manifest, write=True)
            self.assertEqual(
                workflow.read_text(),
                "node-version: '24'\nenv:\n  MESA_VERSION: 9.9.9\n",
            )
            self.assertEqual(
                (root / ".github/actions/a/action.yml").read_text(),
                "python-version: '3.14'\n",
            )

    def test_llvm_forms_derive_from_the_one_pin(self):
        self.assertEqual(
            pins.llvm_env({"llvm": "31.4.1"}),
            {
                "LLVM_VERSION": "31.4.1",
                "LLVM_MAJOR": "31",
                "LLVM_SYS_PREFIX_VAR": "LLVM_SYS_314_PREFIX",
            },
        )

    def test_the_full_llvm_version_is_spelled_out_in_pins_json_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".github").mkdir()
            (root / ".github/pins.json").write_text('{"llvm": "31.4.1"}')
            (root / "tools/ci").mkdir(parents=True)
            (root / "tools/ci/a.sh").write_text("install llvm 31.4.1\n")
            (root / "tools/ci/b.sh").write_text("version 31.4.10 and 131.4.1\n")
            (root / "docs").mkdir()
            (root / "docs/c.md").write_text("LLVM 31 (31.4)\n")
            self.assertEqual(
                pins.literal_llvm_versions(root, "31.4.1"), ["tools/ci/a.sh"]
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
