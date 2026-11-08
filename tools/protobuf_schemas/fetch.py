#!/usr/bin/env python3
"""Fetch the pinned real-world `.proto` sets used to test `Extensions/Protobuf/Schema`.

Each source repository is downloaded as a tarball of one pinned commit and only its `.proto`
files are extracted into a cache folder outside the repository (never vendored). The fetched
files are untrusted data: they are written as plain files and nothing in them is ever run.

A set is pinned by the commit of each source and by `tree_sha256`, the digest of the sorted
`path sha256` lines of the extracted files. A tarball's own bytes are not stable across GitHub
re-archiving, the extracted files are.

    python3 tools/protobuf_schemas/fetch.py                  # every set
    python3 tools/protobuf_schemas/fetch.py onnx k8s         # some sets
    python3 tools/protobuf_schemas/fetch.py --print-hashes   # show digests instead of failing (re-pinning)

The cache is `--cache DIR`, `$PROTOBUF_SCHEMAS_CACHE`, or `protobuf-schemas` in the repo's output folder (`target/`, see `tools/repo_output.py`).
"""
from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import os
import re
import shutil
import sys
import tarfile
import time
import urllib.error
import urllib.request
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from repo_output import output_dir  # noqa: E402
PINS = HERE / "schemas.json"
DEFAULT_CACHE = str(output_dir("protobuf-schemas"))
USER_AGENT = "jacket-tools"
ATTEMPTS = 5
IMPORT = re.compile(r'^\s*import\s+(?:public\s+|weak\s+)?"([^"]+)"', re.M)


def cache_root(argument: str | None) -> Path:
    return Path(argument or os.environ.get("PROTOBUF_SCHEMAS_CACHE") or DEFAULT_CACHE)


def load_pins() -> dict:
    return json.loads(PINS.read_text())


def download(url: str, target: Path) -> None:
    """Stream `url` to `target`, retrying rate limits, 5xx answers and dropped connections."""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(1, ATTEMPTS + 1):
        try:
            print(f"downloading {url}", file=sys.stderr)
            with urllib.request.urlopen(request, timeout=120) as response, open(target, "wb") as out:
                shutil.copyfileobj(response, out, 1 << 20)
            return
        except urllib.error.HTTPError as error:
            if error.code not in (408, 429, 500, 502, 503, 504) or attempt == ATTEMPTS:
                raise SystemExit(f"{url}: HTTP {error.code}")
        except (urllib.error.URLError, OSError) as error:
            if attempt == ATTEMPTS:
                raise SystemExit(f"{url}: {error}")
        time.sleep(2 ** attempt)


def selected(mount: dict, relative: str) -> str | None:
    """The destination path of `relative` (a path inside the source) under `mount`, or None."""
    source = mount["src"]
    if not relative.startswith(source) or not relative.endswith(".proto"):
        return None
    rest = relative[len(source):]
    if mount.get("flat") and "/" in rest:
        return None
    if not fnmatch.fnmatch(PurePosixPath(rest).name, mount.get("match", "*.proto")):
        return None
    if any(part in relative for part in mount.get("exclude", [])):
        return None
    return mount["dst"] + rest


def extract(source: dict, mounts: list[tuple[dict, Path]], scratch: Path) -> None:
    tarball = scratch / "source.tar.gz"
    download(f"https://codeload.github.com/{source['repo']}/tar.gz/{source['commit']}", tarball)
    try:
        with tarfile.open(tarball, "r|gz") as archive:
            for member in archive:
                if not member.isfile():  # symlinks and devices are never extracted
                    continue
                relative = member.name.split("/", 1)[1] if "/" in member.name else ""
                data = None  # a stream cannot seek back: read once for every mount that wants it
                for mount, root in mounts:
                    destination = selected(mount, relative)
                    if destination is None:
                        continue
                    path = root / destination
                    if root.resolve() not in path.resolve().parents:
                        raise SystemExit(f"{member.name}: escapes the cache")
                    path.parent.mkdir(parents=True, exist_ok=True)
                    if data is None:
                        data = archive.extractfile(member).read()
                    path.write_bytes(data)
    finally:
        tarball.unlink(missing_ok=True)


def tree_digest(root: Path) -> tuple[str, int, int]:
    """Digest, file count and line count of the `.proto` files below `root`."""
    lines = []
    count = 0
    total = 0
    for path in sorted(root.rglob("*.proto")):
        data = path.read_bytes()
        lines.append(f"{path.relative_to(root).as_posix()} {hashlib.sha256(data).hexdigest()}\n")
        count += 1
        total += data.count(b"\n")
    return hashlib.sha256("".join(lines).encode()).hexdigest(), count, total


def missing_imports(root: Path) -> dict[str, list[str]]:
    """Imports no file in the set provides (other than the built-in `google/protobuf/*`)."""
    missing: dict[str, list[str]] = {}
    for path in root.rglob("*.proto"):
        for name in IMPORT.findall(path.read_text(errors="replace")):
            if name.startswith("google/protobuf/") or (root / name).exists():
                continue
            missing.setdefault(name, []).append(path.relative_to(root).as_posix())
    return missing


def fetch_sets(names: list[str], cache: Path, print_hashes: bool, force: bool) -> int:
    pins = load_pins()
    sets = pins["sets"]
    names = names or list(sets)
    unknown = [name for name in names if name not in sets]
    if unknown:
        raise SystemExit(f"unknown set(s) {unknown}; known: {list(sets)}")
    cache.mkdir(parents=True, exist_ok=True)
    status = 0
    todo: dict[str, list[tuple[dict, Path]]] = {}
    for name in names:
        root = cache / name
        if root.exists() and not force:
            continue
        shutil.rmtree(root, ignore_errors=True)
        shutil.rmtree(cache / (name + ".partial"), ignore_errors=True)
        for mount in sets[name]["mounts"]:
            todo.setdefault(mount["source"], []).append((mount, cache / (name + ".partial")))
    scratch = cache / ".download"
    scratch.mkdir(exist_ok=True)
    for source_name, mounts in todo.items():
        extract(pins["sources"][source_name], mounts, scratch)
    for name in names:
        partial = cache / (name + ".partial")
        if partial.exists():
            partial.replace(cache / name)
        digest, count, total = tree_digest(cache / name)
        want = sets[name].get("tree_sha256", "")
        gaps = missing_imports(cache / name)
        flag = "ok" if digest == want else "DIGEST MISMATCH"
        print(f"{name}: {count} files, {total} lines, tree_sha256 {digest} [{flag}]")
        if gaps:
            print(f"  unresolved imports: {json.dumps(gaps)[:600]}")
        if digest != want and not print_hashes:
            print(f"  pinned {want or '(none)'}", file=sys.stderr)
            status = 1
    return status


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("sets", nargs="*")
    parser.add_argument("--cache", help="cache folder (default: $PROTOBUF_SCHEMAS_CACHE or %s)" % DEFAULT_CACHE)
    parser.add_argument("--print-hashes", action="store_true", help="report digests without failing on a mismatch")
    parser.add_argument("--force", action="store_true", help="download again even if the set is cached")
    arguments = parser.parse_args()
    sys.exit(fetch_sets(arguments.sets, cache_root(arguments.cache), arguments.print_hashes, arguments.force))


if __name__ == "__main__":
    main()
