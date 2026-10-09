"""Exercise .githooks/pre-commit on a throwaway repository, with stand-in jaifmt and jailint."""
import os
import shutil
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

REPO = Path(__file__).resolve().parents[1]

# Stand-ins: fail when a file mentions BAD. jaifmt reads the staged copy it is handed (the cwd is
# an export of the index), jailint reads the working tree.
FAKE_TOOL = """#!/bin/sh
for f in "$@"; do
    case "$f" in -*|warnings) continue ;; esac
    if grep -q BAD "$f"; then echo "$f: BAD"; exit 1; fi
done
exit 0
"""


def sh(args, cwd, env=None, check=True):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, check=check)


class PreCommitHookTests(unittest.TestCase):
    def setUp(self):
        self.tmp = TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        sh(["git", "init", "-q"], self.repo)
        for key, value in (("user.name", "t"), ("user.email", "t@example.com"),
                           ("commit.gpgsign", "false"), ("core.hooksPath", ".githooks")):
            sh(["git", "config", key, value], self.repo)
        (self.repo / ".githooks").mkdir()
        shutil.copy(REPO / ".githooks/pre-commit", self.repo / ".githooks/pre-commit")
        (self.repo / "tools").mkdir()
        for script in ("rust_item_spacing.py", "staged-files.sh"):
            shutil.copy(REPO / "tools" / script, self.repo / "tools" / script)
        (self.repo / "target").mkdir()
        self.env = dict(os.environ, JAI_HOOK_NO_BUILD="1", JAI_HOOK_SKIP="rustfmt")
        # Keep any real jaifmt/jailint on the developer's PATH out of the tests.
        self.env["PATH"] = os.pathsep.join(
            p for p in self.env["PATH"].split(os.pathsep)
            if not (Path(p) / "jaifmt").exists() and not (Path(p) / "jailint").exists()
        )

    def tool(self, rel):
        path = self.repo / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(FAKE_TOOL)
        path.chmod(0o755)

    def write(self, rel, text):
        path = self.repo / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, *extra):
        return sh(["git", "commit", "-q", "-m", "x", *extra], self.repo, self.env, check=False)

    def test_clean_commit_passes_quietly(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "main :: () {}\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")

    def test_failures_name_the_fix_commands(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1)
        self.assertIn("jaifmt <files>", result.stderr)
        self.assertIn("jailint --fix <files>", result.stderr)
        self.assertEqual(self.commit("--no-verify").returncode, 0)

    def test_checks_the_staged_content(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        self.write("a.jai", "main :: () {}\n")  # fixed in the working tree, not staged
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("a.jai", result.stderr)

    def test_deleted_files_are_skipped(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "main :: () {}\n")
        sh(["git", "add", "a.jai"], self.repo)
        self.assertEqual(self.commit().returncode, 0)
        self.write("a.jai", "BAD\n")
        (self.repo / "a.jai").unlink()
        sh(["git", "add", "-A"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_missing_tools_warn_and_do_not_block(self):
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("jaifmt not found", result.stderr)
        self.assertIn("jailint not found", result.stderr)

    def test_item_spacing_is_checked(self):
        self.write("crates/x/src/lib.rs", "fn a() {\n    1;\n}\nfn b() {\n    2;\n}\n")
        sh(["git", "add", "crates"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("needs item spacing: crates/x/src/lib.rs", result.stderr)
        self.assertIn("python3 tools/rust_item_spacing.py", result.stderr)

    def test_rustfmt_checks_staged_rust(self):
        if shutil.which("rustfmt") is None:
            self.skipTest("rustfmt is not installed")
        env = dict(self.env, JAI_HOOK_SKIP="spacing")
        self.write("crates/x/src/lib.rs", "fn   a( ) { }\n")
        sh(["git", "add", "crates"], self.repo)
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("cargo fmt --all", result.stderr)


if __name__ == "__main__":
    unittest.main()
