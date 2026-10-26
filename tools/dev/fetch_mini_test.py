#!/usr/bin/env python3
"""Fetch mini.test, the Neovim plugin tests' runner, at the commit pinned in .github/pins.json.

    python3 tools/dev/fetch_mini_test.py        # prints the folder: target/nvim-deps/mini.test

The tests' `minimal_init.lua` finds it there, or at $MINI_TEST_DIR. Nothing is installed globally, and
a folder already at the pinned commit is left alone.
"""
import json
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT
from repo_output import output_dir

DEST = output_dir("nvim-deps") / "mini.test"
REPOSITORY = "https://github.com/echasnovski/mini.test"


def git(*args, cwd=None):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def fetch(commit: str, dest: Path = DEST, repository: str = REPOSITORY) -> Path:
    if dest.is_dir():
        try:
            if git("rev-parse", "HEAD", cwd=dest) == commit:
                return dest
        except (subprocess.CalledProcessError, OSError):
            pass
    if dest.exists():
        subprocess.run(["rm", "-rf", str(dest)], check=True)
    dest.mkdir(parents=True)
    git("init", "-q", cwd=dest)
    git("fetch", "-q", "--depth", "1", repository, commit, cwd=dest)
    git("checkout", "-q", "FETCH_HEAD", cwd=dest)
    if git("rev-parse", "HEAD", cwd=dest) != commit:
        raise RuntimeError(f"fetched {repository} but HEAD is not {commit}")
    return dest


def main() -> int:
    pins = json.loads((ROOT / ".github" / "pins.json").read_text())
    print(fetch(pins["mini_test_commit"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
