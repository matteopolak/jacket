#!/usr/bin/env python3
"""Clippy for the crates a push changes. Called by .githooks/pre-push with git's ref lines on stdin.

Runs CI's lint (`cargo clippy --all-targets --locked -- -D warnings`) on just the workspace packages
whose files differ between each pushed commit and what the remote already has. Clippy reads the
working tree, so uncommitted Rust changes are linted too (a warning says so).
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ZERO = "0" * 40


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


if __name__ == "__main__":
    sys.exit(main())
