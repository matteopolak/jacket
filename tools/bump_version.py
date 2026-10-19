#!/usr/bin/env python3
"""Set the compiler version everywhere it is written by hand: `just bump-version X.Y.Z`.

    python3 tools/bump_version.py 0.20.0          # edit, then check both lock files with cargo
    python3 tools/bump_version.py 0.20.0 --no-verify
    python3 tools/bump_version.py --check         # only fail when the copies disagree

Updates the root Cargo.toml `[workspace.package] version`, the path packages of Cargo.lock and
fuzz/Cargo.lock that carry that version, editors/nvim/lua/jai/version.lua, jaifmt/version.jai, and renames
CHANGELOG.md's `## [Unreleased]` heading to `## [X.Y.Z] - <today>` (or adds that header when there is
no such section and none for X.Y.Z yet). editors/vscode/package.json is left
alone: the release workflow stamps its version (docs/tools/releases.md).
"""
import argparse
import datetime
import re
import subprocess
import sys
from toolkit.repo import ROOT

SEMVER = re.compile(r"\d+\.\d+\.\d+")
LOCKS = ("Cargo.lock", "fuzz/Cargo.lock")
MANIFESTS = ("Cargo.toml", "fuzz/Cargo.toml")
VERSION_LUA = "editors/nvim/lua/jai/version.lua"
VERSION_JAIFMT = "jaifmt/version.jai"
WORKSPACE_VERSION = re.compile(r'(\[workspace\.package\][^\[]*?^version = ")([^"]+)(")', re.M | re.S)
LUA_VERSION = re.compile(r"^return '([^']*)'$", re.M)
JAIFMT_VERSION = re.compile(r'^JAIFMT_VERSION :: "([^"]*)";$', re.M)
# A lock entry for a path package has no `source =` line right after its version.
PATH_PACKAGE = re.compile(r'(\[\[package\]\]\nname = "[^"]+"\nversion = ")([^"]+)("\n)(?!source)')


def read(root, rel):
    return (root / rel).read_text()


def workspace_version(root):
    match = WORKSPACE_VERSION.search(read(root, "Cargo.toml"))
    if not match:
        raise SystemExit("Cargo.toml has no [workspace.package] version")
    return match.group(2)


def lock_versions(text):
    return [m.group(2) for m in PATH_PACKAGE.finditer(text)]


def disagreements(root):
    """Where a hand-written copy of the version differs from Cargo.toml."""
    version = workspace_version(root)
    problems = []
    lua = LUA_VERSION.search(read(root, VERSION_LUA))
    if not lua or lua.group(1) != version:
        problems.append(f"{VERSION_LUA} says {lua.group(1) if lua else 'nothing'}, Cargo.toml says {version}")
    fmt = JAIFMT_VERSION.search(read(root, VERSION_JAIFMT))
    if not fmt or fmt.group(1) != version:
        problems.append(f"{VERSION_JAIFMT} says {fmt.group(1) if fmt else 'nothing'}, Cargo.toml says {version}")
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
    lua = read(root, VERSION_LUA)
    if not LUA_VERSION.search(lua):
        raise SystemExit(f"{VERSION_LUA} has no `return '<version>'` line")
    edits[VERSION_LUA] = LUA_VERSION.sub(f"return '{new}'", lua, count=1)
    fmt = read(root, VERSION_JAIFMT)
    if not JAIFMT_VERSION.search(fmt):
        raise SystemExit(f'{VERSION_JAIFMT} has no `JAIFMT_VERSION :: "<version>";` line')
    edits[VERSION_JAIFMT] = JAIFMT_VERSION.sub(f'JAIFMT_VERSION :: "{new}";', fmt, count=1)
    edits["CHANGELOG.md"] = bump_changelog(read(root, "CHANGELOG.md"), new, today)
    for rel, text in edits.items():
        (root / rel).write_text(text)
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
