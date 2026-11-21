#!/usr/bin/env python3
"""Fail closed for every registry dependency, including existing lock entries."""

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import tomllib


from toolkit.age_policy import MIN_AGE, parse_time
from toolkit import github
from toolkit.download import fetch

REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
# A git source locked to an exact commit given as `rev` in the manifest: the `?rev=` query and
# the resolved `#` fragment are the same full hash. Branches, tags and short revs fail closed.
PINNED_GITHUB = re.compile(
    r"git\+https://github\.com/([\w.-]+)/([\w.-]+?)(?:\.git)?\?rev=([0-9a-f]{40})#([0-9a-f]{40})"
)
# The sparse index is served from a CDN without the API's one-request-a-second limit, and each
# crate's file lists every version with its publish time and yank state.
INDEX = "https://index.crates.io"


def index_path(name):
    """The sparse-index path of crate `name`: `1/a`, `2/ab`, `3/a/abc`, or `ab/cd/abcd...`."""
    name = name.lower()
    if len(name) <= 2:
        return f"{len(name)}/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"


def versions(index_text):
    """`{version: {"created_at", "yanked"}}` from a crate's sparse-index file; versions without a
    publish time are left out, so they fail closed."""
    found = {}
    for line in index_text.splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        if isinstance(row.get("pubtime"), str):
            found[row["vers"]] = {"created_at": row["pubtime"], "yanked": bool(row.get("yanked"))}
    return found


def check(packages, get_version, now, get_commit=None):
    errors = []
    for package in packages:
        source = package.get("source", "")
        if not source:
            continue  # Internal path/workspace crate.
        name, version = package["name"], package["version"]
        if source != REGISTRY:
            pinned = PINNED_GITHUB.fullmatch(source)
            if not pinned or pinned[3] != pinned[4] or get_commit is None:
                errors.append(f"{name}@{version}: unverified non-crates.io source")
                continue
            # The committer date: when the commit landed upstream, not when it was authored.
            committed = parse_time(
                get_commit(pinned[1], pinned[2], pinned[4]), "GitHub"
            )
            if committed > now - MIN_AGE:
                errors.append(
                    f"{name}@{version}: git commit younger than 14 days ({committed.isoformat()})"
                )
            continue
        record = get_version(name, version)
        if record is None:
            errors.append(f"{name}@{version}: not in the crates.io index with a publish time")
            continue
        published = parse_time(record["created_at"], "registry")
        if record.get("yanked", False):
            errors.append(f"{name}@{version}: yanked")
        if published > now - MIN_AGE:
            errors.append(
                f"{name}@{version}: younger than 14 days ({published.isoformat()})"
            )
    return errors


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lockfile", type=Path, default=Path("Cargo.lock"))
    args = parser.parse_args()
    packages = tomllib.loads(args.lockfile.read_text())["package"]

    names = sorted({p["name"] for p in packages if p.get("source") == REGISTRY})
    with ThreadPoolExecutor(max_workers=8) as pool:
        files = dict(
            zip(names, pool.map(lambda n: fetch(f"{INDEX}/{index_path(n)}", timeout=30).decode(), names))
        )
    index = {name: versions(text) for name, text in files.items()}

    def get_version(name, version):
        return index[name].get(version)

    commits = {}

    def get_commit(owner, repo, sha):
        # One repository often provides several crates at the same commit.
        if (owner, repo, sha) not in commits:
            record = github.get(f"/repos/{owner}/{repo}/commits/{sha}")
            if record["sha"] != sha:
                raise ValueError(f"GitHub resolved {sha} to {record['sha']}")
            commits[owner, repo, sha] = record["commit"]["committer"]["date"]
        return commits[owner, repo, sha]

    errors = check(packages, get_version, datetime.now(timezone.utc), get_commit)
    if errors:
        raise SystemExit("\n".join(errors))
    print(
        f"Dependency age policy passed ({sum('source' in p for p in packages)} external packages)"
    )


if __name__ == "__main__":
    main()
