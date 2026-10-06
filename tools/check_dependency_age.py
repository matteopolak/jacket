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
import urllib.request

REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
# A git source locked to an exact commit given as `rev` in the manifest: the `?rev=` query and
# the resolved `#` fragment are the same full hash. Branches, tags and short revs fail closed.
PINNED_GITHUB = re.compile(r"git\+https://github\.com/([\w.-]+)/([\w.-]+?)(?:\.git)?\?rev=([0-9a-f]{40})#([0-9a-f]{40})")
MIN_AGE = timedelta(days=14)


def parse_time(text, what):
    stamp = datetime.fromisoformat(text.replace("Z", "+00:00"))
    if stamp.tzinfo is None:
        raise ValueError(f"{what} returned a timestamp without a timezone")
    return stamp


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
            committed = parse_time(get_commit(pinned[1], pinned[2], pinned[4]), "GitHub")
            if committed > now - MIN_AGE:
                errors.append(f"{name}@{version}: git commit younger than 14 days ({committed.isoformat()})")
            continue
        record = get_version(name, version)
        published = parse_time(record["created_at"], "registry")
        if record.get("yanked", False):
            errors.append(f"{name}@{version}: yanked")
        if published > now - MIN_AGE:
            errors.append(f"{name}@{version}: younger than 14 days ({published.isoformat()})")
    return errors


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lockfile", type=Path, default=Path("Cargo.lock"))
    args = parser.parse_args()
    packages = tomllib.loads(args.lockfile.read_text())["package"]
    def get_version(name, version):
        url = "https://crates.io/api/v1/crates/" + urllib.parse.quote(name, safe="") + "/" + urllib.parse.quote(version, safe="")
        request = urllib.request.Request(url, headers={"User-Agent": "jai-rs-dependency-policy/0.1 (local dependency age check)"})
        with urllib.request.urlopen(request, timeout=30) as response:
            record = json.load(response)["version"]
        time.sleep(1)  # Respect registry API rate limits.
        return record
    commits = {}
    def get_commit(owner, repo, sha):
        # One repository often provides several crates at the same commit.
        if (owner, repo, sha) not in commits:
            headers = {"User-Agent": "jai-rs-dependency-policy/0.1", "Accept": "application/vnd.github+json"}
            if os.environ.get("GITHUB_TOKEN"):
                headers["Authorization"] = "Bearer " + os.environ["GITHUB_TOKEN"]
            url = f"https://api.github.com/repos/{owner}/{repo}/commits/{sha}"
            with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=30) as response:
                record = json.load(response)
            if record["sha"] != sha:
                raise ValueError(f"GitHub resolved {sha} to {record['sha']}")
            commits[owner, repo, sha] = record["commit"]["committer"]["date"]
        return commits[owner, repo, sha]
    errors = check(packages, get_version, datetime.now(timezone.utc), get_commit)
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Dependency age policy passed ({sum('source' in p for p in packages)} external packages)")


if __name__ == "__main__":
    main()
