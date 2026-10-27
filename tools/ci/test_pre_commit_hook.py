"""Exercise .githooks/pre-commit (`dev.py precommit`) on a throwaway repository, with stand-in jaifmt and jailint."""

import os
import shutil
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from toolkit.repo import ROOT

REPO = ROOT


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
    return subprocess.run(
        args, cwd=cwd, env=env, text=True, capture_output=True, check=check
    )


class PreCommitHookTests(unittest.TestCase):
    def setUp(self):
        self.tmp = TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        sh(["git", "init", "-q"], self.repo)
        for key, value in (
            ("user.name", "t"),
            ("user.email", "t@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", ".githooks"),
        ):
            sh(["git", "config", key, value], self.repo)
        (self.repo / ".githooks").mkdir()
        shutil.copy(REPO / ".githooks/pre-commit", self.repo / ".githooks/pre-commit")
        (self.repo / "tools/dev").mkdir(parents=True)
        for script in ("dev.py", "rust_item_spacing.py", "staged-files.sh"):
            shutil.copy(REPO / "tools/dev" / script, self.repo / "tools/dev" / script)
        shutil.copytree(
            REPO / "tools/toolkit",
            self.repo / "tools/toolkit",
            ignore=shutil.ignore_patterns("__pycache__", "test_*"),
        )
        (self.repo / "target").mkdir()
        self.env = dict(os.environ, JAI_HOOK_NO_BUILD="1", JAI_HOOK_SKIP="rustfmt")
        self.env.pop("CARGO_TARGET_DIR", None)
        # The copy of tools/ in the throwaway repository is the one dev.py imports from, so ROOT is that repository.
        self.env["PYTHONPATH"] = str(self.repo / "tools")
        # Keep any real jaifmt/jailint on the developer's PATH out of the tests.
        # A directory that holds one of them (Homebrew's bin) also holds tools cargo needs, such as
        # the `sccache` a rustc-wrapper names, so its other entries stay reachable through links.
        kept, dropped = [], []
        for p in self.env["PATH"].split(os.pathsep):
            has_ours = (Path(p) / "jaifmt").exists() or (Path(p) / "jailint").exists()
            (dropped if has_ours else kept).append(p)
        others = Path(self.tmp.name) / "path-others"
        others.mkdir()
        for p in dropped:
            for entry in Path(p).iterdir() if Path(p).is_dir() else ():
                link = others / entry.name
                if entry.name not in ("jaifmt", "jailint") and not link.exists():
                    link.symlink_to(entry)
        self.env["PATH"] = os.pathsep.join([*kept, str(others)])

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
        return sh(
            ["git", "commit", "-q", "-m", "x", *extra], self.repo, self.env, check=False
        )

    def test_clean_commit_passes_quietly(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "main :: () {}\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")

    def test_tools_are_found_in_cargo_target_dir(self):
        custom = Path(self.tmp.name) / "elsewhere"
        for rel in ("jaifmt", "debug/jailint"):
            path = custom / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(FAKE_TOOL)
            path.chmod(0o755)
        self.env["CARGO_TARGET_DIR"] = str(custom)
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertNotIn("not found", result.stderr)
        self.assertIn("a.jai: BAD", result.stderr)

    def test_a_jaifmt_built_in_another_checkout_is_not_used(self):
        self.tool("target/jaifmt")
        self.write("target/jaifmt.checkout", "/some/deleted/worktree")
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        # The stand-in would reject the file, so it was not run; the hook said why.
        self.assertIn("built from another checkout", result.stderr)
        self.assertNotIn("a.jai: BAD", result.stderr)

    def test_a_jaifmt_built_in_this_checkout_is_used(self):
        self.tool("target/jaifmt")
        self.write("target/jaifmt.checkout", str(self.repo.resolve()))
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertNotIn("another checkout", result.stderr)
        self.assertIn("a.jai: BAD", result.stderr)

    def test_jailint_comes_from_a_cargo_build_of_this_checkout(self):
        # A binary already in the target directory (a deleted worktree's, newer than every source
        # here) passes everything; the one cargo links for this checkout rejects BAD.
        self.tool("target/debug/jailint")
        (self.repo / "target/debug/jailint").write_text("#!/bin/sh\nexit 0\n")
        fake = Path(self.tmp.name) / "bin"
        fake.mkdir(exist_ok=True)
        cargo = fake / "cargo"
        cargo.write_text(
            "#!/bin/sh\n"
            'case "$*" in *"-p jailint"*)\n'
            f"  cat > {self.repo}/target/debug/jailint <<'EOF'\n{FAKE_TOOL}EOF\n"
            f"  chmod +x {self.repo}/target/debug/jailint ;;\n"
            "esac\n"
            "exit 0\n"
        )
        cargo.chmod(0o755)
        (self.repo / "target/debug/jailint").write_text("#!/bin/sh\nexit 0\n")
        self.env["PATH"] = os.pathsep.join([str(fake), self.env["PATH"]])
        self.env.pop("JAI_HOOK_NO_BUILD")
        self.env["JAI_HOOK_SKIP"] = "rustfmt jaifmt"
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("a.jai: BAD", result.stderr)

    def test_install_sets_a_relative_hooks_path_over_an_absolute_one(self):
        other = Path(self.tmp.name) / "main-checkout" / ".githooks"
        other.mkdir(parents=True)
        sh(["git", "config", "core.hooksPath", str(other)], self.repo)
        env = {k: v for k, v in self.env.items() if k != "CI"}
        result = sh([sys.executable, "tools/dev/dev.py", "hooks"], self.repo, env)
        self.assertEqual(result.returncode, 0, result.stderr)
        value = sh(["git", "config", "--local", "core.hooksPath"], self.repo).stdout.strip()
        self.assertEqual(value, ".githooks")

    def test_install_drops_a_worktree_level_hooks_path(self):
        sh(["git", "config", "extensions.worktreeConfig", "true"], self.repo)
        sh(["git", "config", "--worktree", "core.hooksPath", "/elsewhere"], self.repo)
        env = {k: v for k, v in self.env.items() if k != "CI"}
        sh([sys.executable, "tools/dev/dev.py", "hooks"], self.repo, env)
        value = sh(["git", "config", "core.hooksPath"], self.repo).stdout.strip()
        self.assertEqual(value, ".githooks")

    def test_missing_python_skips_the_checks(self):
        self.tool("target/jaifmt")
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        # A PATH of symlinks to git and the shell tools only, without python3.
        bin_dir = Path(self.tmp.name) / "nopython"
        bin_dir.mkdir()
        for name in ("git", "sh", "dirname", "env", "cat", "rm", "mkdir", "grep", "sed", "uname"):
            found = shutil.which(name)
            if found:
                (bin_dir / name).symlink_to(found)
        self.env["PATH"] = str(bin_dir)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("python3 not found", result.stderr)

    def test_only_matching_staged_files_reach_each_tool(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        # Staged files no tool takes, and a Jai-looking file that is not staged, are not checked.
        self.write("old.jai", "BAD\n")
        self.write("notes.txt", "BAD\n")
        self.write("scripts/x.rs", "BAD\n")  # Rust outside crates/ is not the workspace
        sh(["git", "add", "notes.txt", "scripts"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_skip_names_turn_off_only_those_checks(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "BAD\n")
        sh(["git", "add", "a.jai"], self.repo)
        env = dict(self.env, JAI_HOOK_SKIP="rustfmt jaifmt")
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertNotIn("jaifmt <files>", result.stderr)
        self.assertIn("jailint --fix <files>", result.stderr)
        env = dict(self.env, JAI_HOOK_SKIP="rustfmt jaifmt jailint")
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_verbose_says_what_runs(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "main :: () {}\n")
        sh(["git", "add", "a.jai"], self.repo)
        env = dict(self.env, JAI_HOOK_VERBOSE="1")
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("pre-commit: jaifmt --check", result.stderr)
        self.assertIn("pre-commit: jailint -D warnings", result.stderr)

    def test_a_partly_staged_jai_file_is_warned_about(self):
        self.tool("target/jaifmt")
        self.tool("target/debug/jailint")
        self.write("a.jai", "main :: () {}\n")
        sh(["git", "add", "a.jai"], self.repo)
        self.write("a.jai", "main :: () { x := 1; }\n")
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("partially staged", result.stderr)

    def test_repo_checks_take_names(self):
        self.write("tools/ci/check_source_style.py", "import sys\nprint('style bad')\nsys.exit(1)\n")
        self.write("tools/ci/check_repo_paths.py", "print('paths fine')\n")
        self.write("tools/ci/check_ci_sources.py", "print('sources fine')\n")
        result = sh([sys.executable, "tools/dev/dev.py", "repo-checks"], self.repo, self.env, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn("style bad", result.stderr)
        result = sh([sys.executable, "tools/dev/dev.py", "repo-checks", "paths", "ci-sources"], self.repo, self.env, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)

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
        self.assertIn("python3 tools/dev/rust_item_spacing.py", result.stderr)

    def test_rustfmt_checks_staged_rust(self):
        if shutil.which("rustfmt") is None:
            self.skipTest("rustfmt is not installed")
        env = dict(self.env, JAI_HOOK_SKIP="spacing")
        self.write("crates/x/src/lib.rs", "fn   a( ) { }\n")
        sh(["git", "add", "crates"], self.repo)
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("cargo fmt --all", result.stderr)

    def test_repo_checks_run_for_any_staged_file(self):
        self.write(
            "tools/ci/check_source_style.py",
            "import sys\nprint('a.rs: too long')\nsys.exit(1)\n",
        )
        self.write("notes.txt", "x\n")
        sh(["git", "add", "notes.txt"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("a.rs: too long", result.stderr)
        self.assertIn("python3 tools/ci/check_source_style.py", result.stderr)
        env = dict(self.env, JAI_HOOK_SKIP="rustfmt repo")
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)

    def fake_shellcheck(self):
        """Put a stand-in shellcheck first on PATH, so the tests do not need the real one."""
        bin_dir = Path(self.tmp.name) / "bin"
        bin_dir.mkdir(exist_ok=True)
        path = bin_dir / "shellcheck"
        path.write_text(FAKE_TOOL)
        path.chmod(0o755)
        self.env["PATH"] = os.pathsep.join([str(bin_dir), self.env["PATH"]])

    def test_shellcheck_checks_staged_shell_files(self):
        self.fake_shellcheck()
        self.write("run.sh", "#!/bin/sh\ntrue\n")
        self.write(
            "tool", "#!/usr/bin/env bash\nBAD\n"
        )  # no extension: found by its shebang
        self.write("notes.txt", "BAD\n")  # not a shell file
        sh(["git", "add", "run.sh", "tool", "notes.txt"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("tool: BAD", result.stderr)
        self.assertNotIn("notes.txt", result.stderr)
        self.assertIn("shellcheck <files>", result.stderr)

    def test_shellcheck_checks_the_staged_content(self):
        self.fake_shellcheck()
        self.write("run.sh", "#!/bin/sh\nBAD\n")
        sh(["git", "add", "run.sh"], self.repo)
        self.write(
            "run.sh", "#!/bin/sh\ntrue\n"
        )  # fixed in the working tree, not staged
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)

    def test_shellcheck_passes_clean_files_and_can_be_skipped(self):
        self.fake_shellcheck()
        self.write("run.sh", "#!/bin/sh\nBAD\n")
        sh(["git", "add", "run.sh"], self.repo)
        env = dict(self.env, JAI_HOOK_SKIP="rustfmt shellcheck")
        result = sh(["git", "commit", "-q", "-m", "x"], self.repo, env, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_missing_shellcheck_warns_and_does_not_block(self):
        # A PATH without the directory of any real shellcheck, keeping git and the shell tools.
        dirs = [
            p
            for p in self.env["PATH"].split(os.pathsep)
            if not (Path(p) / "shellcheck").exists()
        ]
        if shutil.which("git", path=os.pathsep.join(dirs)) is None:
            self.skipTest("git lives next to shellcheck")
        self.env["PATH"] = os.pathsep.join(dirs)
        self.write("run.sh", "#!/bin/sh\nBAD\n")
        sh(["git", "add", "run.sh"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("shellcheck not found", result.stderr)

    def test_stale_lock_file_fails(self):
        if shutil.which("cargo") is None:
            self.skipTest("cargo is not installed")
        self.write(
            "Cargo.toml", '[package]\nname = "x"\nversion = "0.1.0"\nedition = "2024"\n'
        )
        self.write("src/lib.rs", "")
        sh(["cargo", "generate-lockfile", "--offline"], self.repo)
        self.write(
            "Cargo.toml", '[package]\nname = "x"\nversion = "0.2.0"\nedition = "2024"\n'
        )
        sh(["git", "add", "Cargo.toml", "Cargo.lock", "src"], self.repo)
        result = self.commit()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("./Cargo.lock is out of date", result.stderr)


if __name__ == "__main__":
    unittest.main()
