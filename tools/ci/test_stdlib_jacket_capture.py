"""Run the real CI shell driver against a controlled jacket and display wrapper."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from toolkit.repo import ROOT


@unittest.skipUnless(os.name == "posix", "the stdlib CI driver is a Unix shell script")
class DisplayCapture(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        (self.root / "tools/portable").mkdir(parents=True)
        (self.root / "tools/ci").symlink_to(ROOT / "tools/ci", target_is_directory=True)
        (self.root / "tools/portable/build_portable.py").write_text("# No fixtures needed by the controlled driver.\n")
        (self.bin / "python3").symlink_to(sys.executable)
        self.tool(self.bin / "uname", 'print("Linux")')
        self.tool(self.bin / "rustc", 'print("rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)")')
        self.tool(self.bin / "xvfb-run", """import os, subprocess, sys
assert sys.argv[1:4] == ["-a", "-s", "-screen 0 1280x1024x24"]
redirect = subprocess.STDOUT if os.environ["MERGE_CLIENT_STDERR"] == "true" else None
raise SystemExit(subprocess.run(sys.argv[4:], stderr=redirect, close_fds=False).returncode)
""")
        jacket = self.root / "target/release/jacket"
        jacket.parent.mkdir(parents=True)
        self.tool(jacket, r"""import json, os, sys
from pathlib import Path
args = sys.argv[1:]
assert args[:1] == ["test"]
if "--changed" not in args:
    assert "--needs" in args and args[args.index("--needs") + 1] == "display"
with Path(os.environ["COMMAND_LOG"]).open("a") as out:
    out.write(json.dumps(args) + "\n")
print("jacket: root controlled-root (library)", file=sys.stderr)
case = {"name": "Module::one", "kind": "test"}
if os.environ.get("NARROW_WASM") and "--changed" in args and "wasm" not in args:
    raise SystemExit(0)
if "--list" in args:
    print(json.dumps({"type": "list", **case}))
else:
    print(json.dumps({"type": "run", "event": "started", "tests": 1}))
    print(json.dumps({"type": "test", "event": "passed", **case, "subtests": []}))
    print(json.dumps({"type": "run", "event": "finished", "passed": 1, "failed": 0}))
    if os.environ.get("FAIL_INTERP") and "--interp" in args:
        raise SystemExit(23)
""")
        self.env = {**os.environ, "PATH": str(self.bin) + os.pathsep + os.environ["PATH"],
                    "GITHUB_SHA": "a" * 40, "GITHUB_ACTIONS": "true",
                    "JACKET_TEST_PARTITION": "hash:1/4", "JACKET_TEST_CHANGED": "", "WASM": "false",
                    "COMMAND_LOG": str(self.root / "commands.jsonl")}

    def tool(self, path, body):
        path.write_text(f"#!{sys.executable}\n" + body + "\n")
        path.chmod(0o755)

    def invoke(self, merge, **extra):
        return subprocess.run(["bash", str(ROOT / "tools/ci/stdlib_jacket_test.sh")],
                              cwd=self.root, env={**self.env, "MERGE_CLIENT_STDERR": str(merge).lower(), **extra},
                              capture_output=True, text=True)

    def test_default_driver_keeps_original_three_mode_invocations(self):
        result = self.invoke(True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("jacket: root controlled-root", result.stdout + result.stderr)
        commands = [json.loads(line) for line in (self.root / "commands.jsonl").read_text().splitlines()]
        self.assertEqual(len(commands), 3)
        self.assertTrue(all("--list" not in args and "--format" not in args for args in commands))
        self.assertEqual(sum("--interp" in args for args in commands), 1)
        self.assertEqual(sum(args[-2:] == ["--", "-O2"] for args in commands), 1)
        self.assertEqual(sum("--partition" in args for args in commands), 3)

    def test_empty_host_selection_does_not_hide_affected_wasm_cases(self):
        result = self.invoke(True, WASM="true", NARROW_WASM="true",
                             JACKET_TEST_CHANGED='["stdlib/Module/module.jai"]')
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = [json.loads(line) for line in (self.root / "commands.jsonl").read_text().splitlines()]
        executions = [args for args in commands if "--list" not in args]
        self.assertEqual(len(executions), 2, "a native empty selection must not skip WASM modes")
        self.assertTrue(all("wasm" in args and "--exact" in args for args in executions))
        self.assertEqual(sum("--changed" in args for args in commands), 5)

    def test_failed_mode_is_not_hidden_and_later_modes_still_run(self):
        result = self.invoke(True, FAIL_INTERP="true")
        self.assertNotEqual(result.returncode, 0)
        commands = [json.loads(line) for line in (self.root / "commands.jsonl").read_text().splitlines()]
        self.assertEqual(len(commands), 3)


if __name__ == "__main__":
    unittest.main()
