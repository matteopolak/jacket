#!/usr/bin/env python3
"""Clippy for the crates a push changes, and the stdlib tests its stdlib changes can affect.

`check` runs CI's lint (`cargo clippy --all-targets --locked -- -D warnings`) on the workspace
packages whose files differ between HEAD and the default branch, and `jacket test` on the stdlib
modules whose sources or golden outputs changed (the stdlib runtime tests are not part of
`cargo nextest`, so CI would be the first to run them), then records that HEAD's tree passed.
Only those modules' own tests run: finding every test a change can reach (`--changed`) compiles
every root with the debug compiler, minutes where the modules take seconds, and CI runs the rest. Run it before pushing: a cold build takes minutes, and git holds the SSH connection to the
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

# Run as a script (`python3 tools/dev/pre_push_clippy.py check`) or by the hook, with or without PYTHONPATH=tools.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import build_slot  # noqa: E402
from toolkit.repo import cargo_binary  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
RUSTUP = "https://rustup.rs"
ZERO = "0" * 40
# Code gated to x86-64 is not compiled, so not linted, on an arm64 Mac.
X86_TARGET = "x86_64-apple-darwin"
# The stamp's name for "the stdlib tests the change can affect passed", beside the package names.
STDLIB_TESTS = "stdlib-tests"
# The stdlib files whose module's tests `check` runs: sources and golden outputs.
STDLIB_SUFFIXES = (".jai", ".stdout", ".stderr", ".exit")
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
        # What the pushed commit changes against the default branch, as `check` lints it: the
        # remote tip of a rebased branch is an old base and would also count main's changes.
        local = parts[1]
        base = git("merge-base", local, "origin/main").strip()
        if not base:
            continue
        files.update(git("diff", "--name-only", f"{base}...{local}").split())
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


def stdlib_files(files: set[str]) -> list[str]:
    """The changed stdlib sources and golden outputs, whose tests `check` runs."""
    return sorted(f for f in files if f.startswith("stdlib/") and f.endswith(STDLIB_SUFFIXES))


def stdlib_roots(files: list[str]) -> list[str]:
    """The test roots of the modules holding `files`: the nearest folder above each, outside its
    `tests/`, that has a `tests/` folder."""
    roots = set()
    for file in files:
        parts = Path(file).parts
        if "tests" in parts:
            parts = parts[: parts.index("tests")]
        for end in range(len(parts), 1, -1):
            if (ROOT.joinpath(*parts[:end]) / "tests").is_dir():
                roots.add(Path(*parts[:end]).as_posix())
                break
    return sorted(roots)


def required(files: set[str]) -> list[str]:
    """What a passing `check` must have covered for `files`: the packages to lint, and STDLIB_TESTS."""
    return packages(files) + ([STDLIB_TESTS] if stdlib_files(files) else [])


def unlinted(lines: list[str], stamps: dict[str, set[str]]) -> list[tuple[str, list[str]]]:
    """The pushed refs whose changes have no passing `check`, as (local commit, what it must cover)."""
    missing = []
    for line in lines:
        selected = required(changed_files([line]))
        if selected and not set(selected) <= stamps.get(tree_of(line.split()[1]), set()):
            missing.append((line.split()[1], selected))
    return missing


# Files that change how every package builds or lints.
WORKSPACE_FILES = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml", "clippy.toml", "rustfmt.toml"}


def packages(files: set[str]) -> list[str]:
    """The workspace packages owning the changed Rust or `Cargo.toml` files, or all of them for a
    workspace-wide file such as `rust-toolchain.toml`; exits when cargo cannot say."""
    whole = bool(files & WORKSPACE_FILES)
    if not whole and not any(file.endswith((".rs", "Cargo.toml")) for file in files):
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
    if whole:
        return sorted(dirs.values())
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
    """The test targets of `package` holding x86-64-gated code (whole-file or per item).

    A file that is its own `[[test]]` in `Cargo.toml` is that target; every other file is a
    module of the `integration` target (`tests/main.rs`), and so is a file in a folder, unless the
    folder holds a target's modules (`native/` for `native.rs`).
    """
    crate = ROOT / "crates" / package
    own = dict(re.findall(r'\[\[test\]\]\nname = "([^"]+)"\npath = "tests/([^"]+)"', (crate / "Cargo.toml").read_text()))
    # A target's file and the folder of the same name, which holds its modules.
    owner = {}
    for name, path in own.items():
        owner[path] = name
        owner[path.removesuffix(".rs").removesuffix("/main")] = name
    targets = set()
    for f in (crate / "tests").rglob("*.rs"):
        if X86_ONLY_TEST.search(f.read_text()):
            relative = f.relative_to(crate / "tests").as_posix()
            targets.add(owner.get(relative) or owner.get(relative.split("/")[0]) or "integration")
    return sorted(targets)


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


def format_check(files: set[str]) -> int:
    """`cargo fmt --check` and `rust-spacing --check` when `files` hold Rust or the rustfmt config."""
    if not any(f.endswith(".rs") or f == "rustfmt.toml" for f in files):
        return 0
    print("pre-push: cargo fmt and rust-spacing --check", file=sys.stderr)
    for command in (
        ["cargo", "fmt", "--all", "--", "--check"],
        ["cargo", "run", "--quiet", "--locked", "-p", "rust-spacing", "--", "--check"],
    ):
        result = subprocess.run(command, cwd=ROOT)
        if result.returncode:
            print("pre-push: formatting failed; run `python3 tools/dev/dev.py fmt --lang rust`, "
                  "or push with --no-verify", file=sys.stderr)
            return result.returncode
    return 0


def dirty_rust() -> bool:
    return any(
        line[3:].endswith((".rs", "Cargo.toml")) or line[3:] in WORKSPACE_FILES
        for line in git("status", "--porcelain").splitlines()
    )


def check() -> int:
    """Lints what HEAD changes against the default branch and records HEAD's tree when it passes."""
    if os.environ.get("JACKET_HOOKS_NO_CARGO"):
        print("pre-push: JACKET_HOOKS_NO_CARGO is set; skipping clippy", file=sys.stderr)
        return 0
    head = git("rev-parse", "HEAD").strip()
    files = changed_files([f"HEAD {head} HEAD {ZERO}"])
    selected = packages(files)
    stdlib = stdlib_files(files)
    if not selected and not stdlib:
        print("pre-push: no Rust or stdlib changes to check", file=sys.stderr)
        return 0
    dirty = dirty_rust() or bool(git("status", "--porcelain", "--", "stdlib").strip())
    if dirty:
        print("pre-push: warning: the checks also see uncommitted changes, so this run is not recorded", file=sys.stderr)
    with build_slot.slot():
        code = format_check(files)
        if code == 0 and selected:
            code = clippy(selected)
        if code == 0 and stdlib:
            code = stdlib_tests(stdlib)
    if code == 0 and not dirty:
        write_stamp(stamp_file(), tree_of("HEAD"), selected + ([STDLIB_TESTS] if stdlib else []))
    return code


def stdlib_tests(files: list[str]) -> int:
    """`jacket test` on the modules holding `files`, natively, with the checkout's debug jaic and jacket."""
    roots = stdlib_roots(files)
    if not roots:
        return 0
    print(f"pre-push: jacket test {' '.join(roots)}", file=sys.stderr)
    build = subprocess.run(["cargo", "build", "--locked", "-q", "-p", "jaic-cli", "-p", "jacket"], cwd=ROOT)
    if build.returncode:
        return build.returncode
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), PATH=f"{cargo_binary('jaic').parent}{os.pathsep}{os.environ['PATH']}")
    command = [str(cargo_binary("jacket")), "test", "-q", *(arg for root in roots for arg in ("--path", root))]
    result = subprocess.run(command, cwd=ROOT, env=env)
    if result.returncode:
        print("pre-push: stdlib tests failed; fix them or push with --no-verify", file=sys.stderr)
    return result.returncode


def clippy(selected: list[str]) -> int:
    """Clippy for `selected`, as CI runs it."""
    print(f"pre-push: clippy {' '.join(selected)}", file=sys.stderr)
    command = ["cargo", "clippy", "--all-targets", "--locked", "--message-format", "short"]
    command += [arg for name in selected for arg in ("-p", name)]
    result = subprocess.run([*command, "--", "-D", "warnings"], cwd=ROOT)
    if result.returncode:
        print("pre-push: clippy failed; fix it or push with --no-verify", file=sys.stderr)
        return result.returncode
    return lint_x86_tests(selected)


def main() -> int:
    if sys.argv[1:] == ["check"]:
        return check()
    if os.environ.get("JACKET_HOOKS_NO_CARGO"):
        print("pre-push: JACKET_HOOKS_NO_CARGO is set; skipping clippy", file=sys.stderr)
        return 0
    missing = unlinted(sys.stdin.read().splitlines(), read_stamps(stamp_file()))
    for commit, selected in missing:
        print(
            f"pre-push: `check` has not passed for {commit[:10]} ({' '.join(selected)})."
            " Run `python3 tools/dev/pre_push_clippy.py check`, then push again",
            file=sys.stderr,
        )
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
