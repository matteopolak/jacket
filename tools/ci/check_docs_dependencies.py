#!/usr/bin/env python3
"""Fail closed on the docs builder's complete npm dependency graph before installing it.

    PYTHONPATH=tools python3 tools/ci/check_docs_dependencies.py --cache /tmp/docs-dependency-age.json

Every locked registry release and the pinned npm release must be at least fourteen
 days old. Cached registry identity, tarball and SHA-512 metadata expires after three days.
"""

import argparse
import base64
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import re
from urllib.parse import quote, urlparse

from toolkit.download import fetch
from toolkit.repo import ROOT

MIN_AGE = timedelta(days=14)
CACHE_TTL = timedelta(days=3)
VERSION = r"\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?"
NAME = re.compile(r"(?:@[a-z0-9._~-]+/)?[a-z0-9._~-]+")


def stamp(value):
    if not isinstance(value, str):
        raise ValueError("registry timestamp must be a string")
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("registry timestamp must include its timezone")
    return parsed


def integrity(value):
    if not isinstance(value, str) or not value.startswith("sha512-"):
        raise ValueError("expected a SHA-512 registry integrity value")
    try:
        digest = base64.b64decode(value[7:], validate=True)
    except ValueError as error:
        raise ValueError("invalid SHA-512 registry integrity encoding") from error
    if len(digest) != 64:
        raise ValueError("invalid SHA-512 registry integrity length")


def registry_url(value):
    if not isinstance(value, str):
        raise ValueError("expected a registry tarball URL")
    url = urlparse(value)
    if (
        url.scheme != "https"
        or url.netloc != "registry.npmjs.org"
        or url.query
        or url.fragment
        or not url.path.endswith(".tgz")
    ):
        raise ValueError("only HTTPS registry.npmjs.org tarballs are accepted")


def requirements(package, lock):
    """Read every locked release, including nested and optional platform dependencies."""
    if not isinstance(package, dict) or not isinstance(lock, dict):
        raise ValueError("package and lock metadata must be objects")
    if lock.get("lockfileVersion") != 3 or not isinstance(lock.get("packages"), dict):
        raise ValueError("expected npm lockfile format 3 with a packages object")
    root = lock["packages"].get("")
    if not isinstance(root, dict) or any(
        root.get(key) != package.get(key) for key in ("name", "version")
    ):
        raise ValueError("lockfile root must match package name/version")
    if any(lock.get(key) != package.get(key) for key in ("name", "version")):
        raise ValueError("lockfile envelope must match package name/version")
    for key in ("dependencies", "devDependencies", "optionalDependencies"):
        values = package.get(key, {})
        if not isinstance(values, dict) or values != root.get(key, {}):
            raise ValueError(f"lockfile root {key} differs from package metadata")
        if any(
            not isinstance(value, str) or re.fullmatch(VERSION, value) is None
            for value in values.values()
        ):
            raise ValueError(f"{key} must pin exact registry versions")
    manager = re.fullmatch(f"npm@({VERSION})", package.get("packageManager", ""))
    if not manager:
        raise ValueError("packageManager must pin npm@<exact version>")
    found = [{"name": "npm", "version": manager[1], "integrity": None, "tarball": None}]
    for path, entry in lock["packages"].items():
        if not path:
            continue
        if (
            not isinstance(path, str)
            or any(part in ("", ".", "..") for part in path.split("/"))
            or not path.startswith("node_modules/")
            or not isinstance(entry, dict)
            or entry.get("link")
        ):
            raise ValueError(f"{path}: not a plain locked registry package")
        name = entry.get("name", path.rsplit("node_modules/", 1)[-1])
        version = entry.get("version")
        if (
            not isinstance(name, str)
            or not NAME.fullmatch(name)
            or not isinstance(version, str)
            or not re.fullmatch(VERSION, version)
        ):
            raise ValueError(f"{path}: invalid registry package identity")
        integrity(entry.get("integrity"))
        registry_url(entry.get("resolved"))
        found.append(
            {
                "name": name,
                "version": version,
                "integrity": entry["integrity"],
                "tarball": entry["resolved"],
            }
        )
    if len(found) == 1:
        raise ValueError("npm lockfile contains no external packages")
    return found


def fresh(record, now):
    """Trust an exact registry answer only within the same three-day window as Rust checks."""
    try:
        integrity(record["integrity"])
        registry_url(record["tarball"])
        stamp(record["created_at"])
        return timedelta(0) <= now - stamp(record["fetched"]) < CACHE_TTL
    except (KeyError, TypeError, ValueError, AttributeError):
        return False


def check(package, lock, lookup, now):
    """Check source pins and live/cached release facts; e.g. a thirteen-day transitive fails."""
    try:
        packages = requirements(package, lock)
    except (TypeError, ValueError) as error:
        return [str(error)]
    errors = []
    for wanted in packages:
        name, version = wanted["name"], wanted["version"]
        try:
            record = lookup(name, version)
            if record["name"] != name or record["version"] != version:
                raise ValueError("registry returned a different package identity")
            integrity(record["integrity"])
            registry_url(record["tarball"])
            if wanted["integrity"] and wanted["integrity"] != record["integrity"]:
                raise ValueError(
                    "lockfile SHA-512 integrity differs from registry dist metadata"
                )
            if wanted["tarball"] and wanted["tarball"] != record["tarball"]:
                raise ValueError("lockfile tarball differs from registry dist metadata")
            if now - stamp(record["created_at"]) < MIN_AGE:
                raise ValueError(
                    "release is younger than fourteen days or dated in the future"
                )
        except (KeyError, TypeError, ValueError, OSError) as error:
            errors.append(f"{name}@{version}: {error}")
    return errors


def refresh(name, versions, now):
    """Fetch one package history and retain only facts for the exact requested releases."""
    payload = json.loads(
        fetch(
            "https://registry.npmjs.org/" + quote(name, safe="@"),
            timeout=30,
            attempts=1,
            max_bytes=64 * 1024 * 1024,
            headers={"Accept": "application/json"},
        )
    )
    if not isinstance(payload, dict) or payload.get("name") != name:
        raise ValueError("registry returned a different package history")
    results = {}
    for version in versions:
        record = payload.get("versions", {}).get(version)
        if not isinstance(record, dict):
            raise ValueError(f"{name}@{version}: release is absent or unpublished")
        results[f"{name}@{version}"] = {
            "name": record["name"],
            "version": record["version"],
            "created_at": payload["time"][version],
            "integrity": record["dist"]["integrity"],
            "tarball": record["dist"]["tarball"],
            "fetched": now.isoformat(),
        }
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--directory", type=Path, default=ROOT / "website/docs")
    parser.add_argument(
        "--cache", type=Path, default=os.environ.get("DOCS_DEPENDENCY_AGE_CACHE")
    )
    args = parser.parse_args()
    package = json.loads((args.directory / "package.json").read_text())
    lock = json.loads((args.directory / "package-lock.json").read_text())
    try:
        packages = requirements(package, lock)
    except (TypeError, ValueError) as error:
        parser.exit(1, f"docs dependency policy: {error}\n")
    now = datetime.now(timezone.utc)
    try:
        cache = json.loads(args.cache.read_text()) if args.cache else {}
        if not isinstance(cache, dict):
            cache = {}
    except (OSError, ValueError):
        cache = {}
    pending = {}
    for entry in packages:
        name, version = entry["name"], entry["version"]
        if not fresh(cache.get(f"{name}@{version}", {}), now):
            pending.setdefault(name, set()).add(version)
    failures = {}
    with ThreadPoolExecutor(max_workers=4) as pool:
        futures = {
            pool.submit(refresh, name, versions, now): name
            for name, versions in pending.items()
        }
        for future in as_completed(futures):
            name = futures[future]
            try:
                cache.update(future.result())
            except (OSError, ValueError, KeyError, TypeError) as error:
                failures[name] = str(error)

    def lookup(name, version):
        if name in failures:
            raise ValueError(failures[name])
        record = cache.get(f"{name}@{version}")
        if not fresh(record, now):
            raise ValueError("no fresh verified registry metadata")
        return record

    errors = check(package, lock, lookup, now)
    if args.cache:
        args.cache.parent.mkdir(parents=True, exist_ok=True)
        args.cache.write_text(json.dumps(cache, sort_keys=True) + "\n")
    if errors:
        parser.exit(1, "\n".join(errors) + "\n")
    print(f"Docs dependency policy passed ({len(packages)} locked/npm releases)")


if __name__ == "__main__":
    main()
