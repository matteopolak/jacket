#!/usr/bin/env python3
"""Clippy for the crates a push changes. Called by .githooks/pre-push with git's ref lines on stdin.

Runs CI's lint (`cargo clippy --all-targets --locked -- -D warnings`) on just the workspace packages
whose files differ between each pushed commit and what the remote already has. Clippy reads the
working tree, so uncommitted Rust changes are linted too (a warning says so).
"""

import json
import platform
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ZERO = "0" * 40
# Code gated to x86-64 is not compiled, so not linted, on an arm64 Mac.
X86_TARGET = "x86_64-apple-darwin"
X86_ONLY_TEST = re.compile(r'#!?\[cfg\(target_arch = "x86_64"\)\]')


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout


def changed_files(lines: list[str]) -> set[str]:
    files: set[str] = set()
    for line in lines:
        parts = line.split()
        if len(parts) != 4 or parts[1] == ZERO:
            continue  # a deleted ref pushes nothing
        local, remote = parts[1], parts[3]
        if remote == ZERO or not git("cat-file", "-t", remote).strip():
            # A new ref, or one this clone has not fetched: compare against the default branch.
            remote = git("merge-base", local, "origin/main").strip()
            if not remote:
                continue
        files.update(git("diff", "--name-only", f"{remote}...{local}").split())
    return files


def packages(files: set[str]) -> list[str]:
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if metadata.returncode != 0:
        return []
    dirs = {
        Path(p["manifest_path"]).parent.relative_to(ROOT).as_posix(): p["name"]
        for p in json.loads(metadata.stdout)["packages"]
        if Path(p["manifest_path"]).is_relative_to(ROOT)
    }
    found = set()
    for file in files:
        if not (file.endswith(".rs") or file.endswith("Cargo.toml")):
            continue
        # The deepest package directory holding the file.
        owner = max((d for d in dirs if d != "." and file.startswith(d + "/")), key=len, default=None)
        if owner:
            found.add(dirs[owner])
    return sorted(found)


def x86_only_tests(package: str) -> list[str]:
    """The integration tests of `package` with x86-64-gated code (whole-file or per item)."""
    tests = ROOT / "crates" / package / "tests"
    return sorted(f.stem for f in tests.glob("*.rs") if X86_ONLY_TEST.search(f.read_text()))


def lint_x86_tests(selected: list[str]) -> int:
    """Clippy the x86-64-only tests on an arm64 Mac, where the host lint never compiles them."""
    if platform.system() != "Darwin" or platform.machine() != "arm64" or "jaic" not in selected:
        return 0
    tests = x86_only_tests("jaic")
    if not tests:
        return 0
    installed = subprocess.run(["rustup", "target", "list", "--installed"], capture_output=True, text=True)
    if X86_TARGET not in installed.stdout.split():
        print(f"pre-push: skipping the x86-64 test lint: `rustup target add {X86_TARGET}` enables it", file=sys.stderr)
        return 0
    print(f"pre-push: clippy jaic x86-64 tests ({' '.join(tests)})", file=sys.stderr)
    command = ["cargo", "clippy", "--locked", "--message-format", "short", "-p", "jaic", "--target", X86_TARGET]
    command += [arg for name in tests for arg in ("--test", name)]
    result = subprocess.run([*command, "--", "-D", "warnings"], cwd=ROOT)
    if result.returncode:
        print("pre-push: x86-64 clippy failed; fix it or push with --no-verify", file=sys.stderr)
    return result.returncode


def main() -> int:
    selected = packages(changed_files(sys.stdin.read().splitlines()))
    if not selected:
        return 0
    if any(f.endswith(".rs") for f in git("status", "--porcelain").split()):
        print("pre-push: warning: clippy also sees uncommitted Rust changes", file=sys.stderr)
    print(f"pre-push: clippy {' '.join(selected)}", file=sys.stderr)
    command = ["cargo", "clippy", "--all-targets", "--locked", "--message-format", "short"]
    command += [arg for name in selected for arg in ("-p", name)]
    result = subprocess.run([*command, "--", "-D", "warnings"], cwd=ROOT)
    if result.returncode:
        print("pre-push: clippy failed; fix it or push with --no-verify", file=sys.stderr)
        return result.returncode
    return lint_x86_tests(selected)


if __name__ == "__main__":
    sys.exit(main())
