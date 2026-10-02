#!/usr/bin/env python3
"""Fail closed for every registry dependency, including existing lock entries."""
import argparse
from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import time
import tomllib
import urllib.parse
import urllib.request


def check(packages, get_version, now):
    errors = []
    for package in packages:
        source = package.get("source", "")
        if not source:
            continue  # Internal path/workspace crate.
        name, version = package["name"], package["version"]
        if source != "registry+https://github.com/rust-lang/crates.io-index":
            errors.append(f"{name}@{version}: unverified non-crates.io source")
            continue
        record = get_version(name, version)
        published = datetime.fromisoformat(record["created_at"].replace("Z", "+00:00"))
        if published.tzinfo is None:
            raise ValueError("registry returned a timestamp without a timezone")
        if record.get("yanked", False):
            errors.append(f"{name}@{version}: yanked")
        if published > now - timedelta(days=14):
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
    errors = check(packages, get_version, datetime.now(timezone.utc))
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Dependency age policy passed ({sum('source' in p for p in packages)} external packages)")


if __name__ == "__main__":
    main()
