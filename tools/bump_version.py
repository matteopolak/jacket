#!/usr/bin/env python3
"""Set the compiler version everywhere it is written by hand: `just bump-version X.Y.Z`.

    python3 tools/bump_version.py 0.20.0          # edit, then check both lock files with cargo
    python3 tools/bump_version.py 0.20.0 --no-verify
    python3 tools/bump_version.py --check         # only fail when the copies disagree

Updates the root Cargo.toml `[workspace.package] version`, the path packages of Cargo.lock and
fuzz/Cargo.lock that carry that version, and renames CHANGELOG.md's `## [Unreleased]` heading to
`## [X.Y.Z] - <today>` (or adds that header when there is no such section and none for X.Y.Z yet).
The generated copies (jaifmt/version.jai, editors/nvim/lua/jai/version.lua) are rewritten by
tools/generators/generate_version_files.py, which this runs last. editors/vscode/package.json is left
alone: the release workflow stamps its version (docs/tools/releases.md).
"""
import argparse
import datetime
import re
import subprocess
import sys
from generators import generate_version_files
from toolkit.repo import ROOT
from toolkit.version import WORKSPACE_VERSION, workspace_version

SEMVER = re.compile(r"\d+\.\d+\.\d+")
LOCKS = ("Cargo.lock", "fuzz/Cargo.lock")
MANIFESTS = ("Cargo.toml", "fuzz/Cargo.toml")
# A lock entry for a path package has no `source =` line right after its version.
PATH_PACKAGE = re.compile(r'(\[\[package\]\]\nname = "[^"]+"\nversion = ")([^"]+)("\n)(?!source)')


def read(root, rel):
    return (root / rel).read_text()


def lock_versions(text):
    return [m.group(2) for m in PATH_PACKAGE.finditer(text)]


def disagreements(root):
    """Where a hand-written copy of the version differs from Cargo.toml."""
    version = workspace_version(root)
    problems = []
    for rel in generate_version_files.stale(root):
        problems.append(f"{rel} does not match Cargo.toml's {version}; run tools/generators/generate_version_files.py")
    for lock in LOCKS:
        if (root / lock).exists():
            stale = sorted({v for v in lock_versions(read(root, lock)) if v not in (version, "0.0.0")})
            if stale:
                problems.append(f"{lock} has workspace packages at {', '.join(stale)}, Cargo.toml says {version}")
    return problems


def bump_cargo_toml(text, new):
    return WORKSPACE_VERSION.sub(lambda m: m.group(1) + new + m.group(3), text, count=1)


def bump_lock(text, old, new):
    return PATH_PACKAGE.sub(lambda m: m.group(1) + (new if m.group(2) == old else m.group(2)) + m.group(3), text)


def bump_changelog(text, new, today):
    if re.search(rf"^## \[{re.escape(new)}\]", text, re.M):
        return text
    unreleased = re.search(r"^## \[Unreleased\][^\n]*$", text, re.M)
    if unreleased:
        return text[:unreleased.start()] + f"## [{new}] - {today}" + text[unreleased.end():]
    first = re.search(r"^## \[", text, re.M)
    header = f"## [{new}] - {today}\n\n"
    if first is None:
        return text.rstrip("\n") + "\n\n" + header.rstrip("\n") + "\n"
    return text[:first.start()] + header + text[first.start():]


def bump(root, new, today=None):
    """Edit the files under `root`; returns the previous version."""
    if not SEMVER.fullmatch(new):
        raise SystemExit(f"version must be X.Y.Z, got {new}")
    today = today or datetime.date.today().isoformat()
    old = workspace_version(root)
    if old == new:
        raise SystemExit(f"already at {new}")
    edits = {"Cargo.toml": bump_cargo_toml(read(root, "Cargo.toml"), new)}
    for lock in LOCKS:
        if (root / lock).exists():
            edits[lock] = bump_lock(read(root, lock), old, new)
    edits["CHANGELOG.md"] = bump_changelog(read(root, "CHANGELOG.md"), new, today)
    for rel, text in edits.items():
        (root / rel).write_text(text)
    generate_version_files.write(root)
    return old


def verify(root):
    """`cargo metadata --locked` fails when a lock file does not match its manifest."""
    for manifest in MANIFESTS:
        command = ["cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--manifest-path", str(root / manifest)]
        result = subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
        if result.returncode:
            sys.stderr.write(result.stderr)
            raise SystemExit(f"{manifest}: its lock file does not match after the bump")
        print(f"cargo metadata --locked: {manifest} ok")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("version", nargs="?", help="the new X.Y.Z")
    parser.add_argument("--check", action="store_true", help="only check that the copies agree")
    parser.add_argument("--no-verify", action="store_true", help="skip cargo metadata --locked on both lock files")
    args = parser.parse_args()
    if args.check or not args.version:
        problems = disagreements(ROOT)
        if problems:
            raise SystemExit("\n".join(problems))
        print(f"Version copies agree at {workspace_version(ROOT)}")
        return
    old = bump(ROOT, args.version)
    print(f"{old} -> {args.version}")
    if not args.no_verify:
        verify(ROOT)
    print("Now fill in the CHANGELOG.md section (docs/tools/releases.md), commit, tag and push the tag.")


if __name__ == "__main__":
    main()
