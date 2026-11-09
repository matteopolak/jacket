#!/usr/bin/env python3
"""Fail closed for every registry dependency, including existing lock entries."""

import argparse
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import re
import time
import tomllib
import urllib.parse


from toolkit.age_policy import MIN_AGE, load_cache, parse_time, recent, save_cache
from toolkit.download import fetch

REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
# A git source locked to an exact commit given as `rev` in the manifest: the `?rev=` query and
# the resolved `#` fragment are the same full hash. Branches, tags and short revs fail closed.
PINNED_GITHUB = re.compile(
    r"git\+https://github\.com/([\w.-]+)/([\w.-]+?)(?:\.git)?\?rev=([0-9a-f]{40})#([0-9a-f]{40})"
)
# A publish date never changes, but a release can be yanked later: a cached answer is trusted for
# this long and looked up again after that.
CACHE_TTL = timedelta(days=3)


def fresh(entry, now):
    return (
        isinstance(entry.get("created_at"), str)
        and isinstance(entry.get("yanked"), bool)
        and recent(entry.get("fetched"), now, CACHE_TTL)
    )


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
    parser.add_argument(
        "--cache",
        type=Path,
        default=os.environ.get("DEPENDENCY_AGE_CACHE") or None,
        help="JSON file of registry answers kept between runs (default: $DEPENDENCY_AGE_CACHE); "
        "only the lookups it lacks are made",
    )
    args = parser.parse_args()
    cache = load_cache(args.cache) if args.cache else {}
    cached_before = dict(cache)
    packages = tomllib.loads(args.lockfile.read_text())["package"]

    def get_version(name, version):
        entry = cache.get(f"{name}@{version}")
        if entry and fresh(entry, datetime.now(timezone.utc)):
            return entry
        url = (
            "https://crates.io/api/v1/crates/"
            + urllib.parse.quote(name, safe="")
            + "/"
            + urllib.parse.quote(version, safe="")
        )
        record = json.loads(
            fetch(
                url,
                timeout=30,
                headers={
                    "User-Agent": "jai-rs-dependency-policy/0.1 (local dependency age check)"
                },
            )
        )["version"]
        time.sleep(1)  # Respect registry API rate limits.
        cache[f"{name}@{version}"] = {
            "created_at": record["created_at"],
            "yanked": bool(record.get("yanked", False)),
            "fetched": datetime.now(timezone.utc).isoformat(),
        }
        return record

    commits = {}

    def get_commit(owner, repo, sha):
        # One repository often provides several crates at the same commit.
        if (owner, repo, sha) not in commits:
            headers = {
                "User-Agent": "jai-rs-dependency-policy/0.1",
                "Accept": "application/vnd.github+json",
            }
            if os.environ.get("GITHUB_TOKEN"):
                headers["Authorization"] = "Bearer " + os.environ["GITHUB_TOKEN"]
            url = f"https://api.github.com/repos/{owner}/{repo}/commits/{sha}"
            record = json.loads(fetch(url, timeout=30, headers=headers))
            if record["sha"] != sha:
                raise ValueError(f"GitHub resolved {sha} to {record['sha']}")
            commits[owner, repo, sha] = record["commit"]["committer"]["date"]
        return commits[owner, repo, sha]

    errors = check(packages, get_version, datetime.now(timezone.utc), get_commit)
    if args.cache and cache != cached_before:
        save_cache(args.cache, cache)
    if errors:
        raise SystemExit("\n".join(errors))
    print(
        f"Dependency age policy passed ({sum('source' in p for p in packages)} external packages)"
    )


if __name__ == "__main__":
    main()
