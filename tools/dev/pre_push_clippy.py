#!/usr/bin/env python3
"""Clippy for the crates a push changes.

`check` runs CI's lint (`cargo clippy --all-targets --locked -- -D warnings`) on the workspace
packages whose files differ between HEAD and the default branch, then records that HEAD's tree
passed. Run it before pushing: a cold build takes minutes, and git holds the SSH connection to the
remote while its pre-push hook runs, so a slow lint there gets the connection closed.

With no argument this is the pre-push hook, called by .githooks/pre-push with git's ref lines on
stdin. It never runs cargo: it fails when a pushed commit changes Rust and its tree has no record
of a passing `check` covering those packages.
"""

import json
import os
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUSTUP = "https://rustup.rs"
ZERO = "0" * 40
# Code gated to x86-64 is not compiled, so not linted, on an arm64 Mac.
X86_TARGET = "x86_64-apple-darwin"
KEPT_STAMPS = 20
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


def stamp_file() -> Path:
    path = Path(git("rev-parse", "--git-path", "jacket-clippy-ok").strip())
    return path if path.is_absolute() else ROOT / path


def read_stamps(path: Path) -> dict[str, set[str]]:
    """Trees that passed clippy, each with the packages linted: lines of `<tree> <package>...`."""
    stamps: dict[str, set[str]] = {}
    try:
        lines = path.read_text().splitlines()
    except OSError:
        return stamps
    for line in lines:
        tree, *names = line.split()
        stamps[tree] = set(names)
    return stamps


def write_stamp(path: Path, tree: str, selected: list[str]) -> None:
    """Records `tree` as linted, keeping the most recent trees only."""
    stamps = read_stamps(path)
    stamps.pop(tree, None)
    stamps[tree] = set(selected)
    recent = list(stamps.items())[-KEPT_STAMPS:]
    path.write_text("".join(f"{t} {' '.join(sorted(n))}\n" for t, n in recent))


def tree_of(commit: str) -> str:
    return git("rev-parse", f"{commit}^{{tree}}").strip()


def unlinted(lines: list[str], stamps: dict[str, set[str]]) -> list[tuple[str, list[str]]]:
    """The pushed refs whose changed packages have no passing `check`, as (local commit, packages)."""
    missing = []
    for line in lines:
        selected = packages(changed_files([line]))
        if selected and not set(selected) <= stamps.get(tree_of(line.split()[1]), set()):
            missing.append((line.split()[1], selected))
    return missing


def packages(files: set[str]) -> list[str]:
    """The workspace packages owning the changed Rust or `Cargo.toml` files; exits when cargo cannot say."""
    if not any(file.endswith((".rs", "Cargo.toml")) for file in files):
        return []
    if shutil.which("cargo") is None:
        raise SystemExit(
            f"pre-push: cargo not found: install Rust ({RUSTUP}), or skip with JAI_HOOK_SKIP=clippy or git push --no-verify, or set JACKET_HOOKS_NO_CARGO=1 in a job with only a prebuilt toolchain"
        )
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if metadata.returncode != 0:
        raise SystemExit(
            f"pre-push: cargo metadata failed; skip with JAI_HOOK_SKIP=clippy or git push --no-verify\n{metadata.stderr}"
        )
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


def dirty_rust() -> bool:
    return any(
        line[3:].endswith((".rs", "Cargo.toml", "Cargo.lock")) for line in git("status", "--porcelain").splitlines()
    )


def check() -> int:
    """Lints what HEAD changes against the default branch and records HEAD's tree when it passes."""
    if os.environ.get("JACKET_HOOKS_NO_CARGO"):
        print("pre-push: JACKET_HOOKS_NO_CARGO is set; skipping clippy", file=sys.stderr)
        return 0
    head = git("rev-parse", "HEAD").strip()
    selected = packages(changed_files([f"HEAD {head} HEAD {ZERO}"]))
    if not selected:
        print("pre-push: no Rust changes to lint", file=sys.stderr)
        return 0
    dirty = dirty_rust()
    if dirty:
        print("pre-push: warning: clippy also sees uncommitted Rust changes, so this run is not recorded", file=sys.stderr)
    print(f"pre-push: clippy {' '.join(selected)}", file=sys.stderr)
    command = ["cargo", "clippy", "--all-targets", "--locked", "--message-format", "short"]
    command += [arg for name in selected for arg in ("-p", name)]
    result = subprocess.run([*command, "--", "-D", "warnings"], cwd=ROOT)
    if result.returncode:
        print("pre-push: clippy failed; fix it or push with --no-verify", file=sys.stderr)
        return result.returncode
    code = lint_x86_tests(selected)
    if code == 0 and not dirty:
        write_stamp(stamp_file(), tree_of("HEAD"), selected)
    return code


def main() -> int:
    if sys.argv[1:] == ["check"]:
        return check()
    if os.environ.get("JACKET_HOOKS_NO_CARGO"):
        print("pre-push: JACKET_HOOKS_NO_CARGO is set; skipping clippy", file=sys.stderr)
        return 0
    missing = unlinted(sys.stdin.read().splitlines(), read_stamps(stamp_file()))
    for commit, selected in missing:
        print(
            f"pre-push: clippy has not passed for {commit[:10]} ({' '.join(selected)})."
            " Run `python3 tools/dev/pre_push_clippy.py check`, then push again",
            file=sys.stderr,
        )
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
