#!/usr/bin/env python3
"""Preserve independently authored workspace sources without build artifacts."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
from datetime import datetime, timezone


EXCLUDED = {"reference", "artifacts", "target", ".git"}


def git(root, *arguments):
    return subprocess.check_output(["git", "-C", str(root), *arguments])


def included(name):
    path = Path(name)
    return path.parts[0] not in EXCLUDED and path.parts[:2] != ("corpus", "upstream")


def capture(root, destination):
    head = git(root, "rev-parse", "HEAD").decode().strip()
    destination.mkdir(parents=True, exist_ok=False)
    source = destination / "source"
    source.mkdir()
    paths = sorted(set(git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").split(b"\0")) - {b""})
    records = []

    for encoded in paths:
        name = os.fsdecode(encoded)
        if not included(name):
            continue
        original = root / name
        if not original.exists() and not original.is_symlink():
            records.append({"path": name, "kind": "deleted"})
            continue
        output = source / name
        output.parent.mkdir(parents=True, exist_ok=True)
        if original.is_symlink():
            target = os.readlink(original)
            output.symlink_to(target)
            records.append({"path": name, "kind": "symlink", "target": target})
            continue
        before = original.stat()
        if not stat.S_ISREG(before.st_mode):
            raise RuntimeError(f"unsupported source file type: {name}")
        data = original.read_bytes()
        after = original.stat()
        if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
            raise RuntimeError(f"source changed during capture: {name}")
        output.write_bytes(data)
        output.chmod(before.st_mode & 0o777)
        records.append({"path": name, "kind": "file", "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})

    for record in records:
        original = root / record["path"]
        if record["kind"] == "file":
            if hashlib.sha256(original.read_bytes()).hexdigest() != record["sha256"]:
                raise RuntimeError(f"source changed before sealing: {record['path']}")
        elif record["kind"] == "symlink":
            if not original.is_symlink() or os.readlink(original) != record["target"]:
                raise RuntimeError(f"symlink changed before sealing: {record['path']}")
        elif original.exists() or original.is_symlink():
            raise RuntimeError(f"deleted path reappeared: {record['path']}")

    current_paths = sorted(set(git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").split(b"\0")) - {b""})
    if [p for p in paths if included(os.fsdecode(p))] != [p for p in current_paths if included(os.fsdecode(p))]:
        raise RuntimeError("source inventory changed during capture")
    if git(root, "rev-parse", "HEAD").decode().strip() != head:
        raise RuntimeError("Git HEAD changed during capture")

    manifest = {
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "head": head,
        "source_root": str(root),
        "files": records,
        "excluded": sorted(EXCLUDED | {"corpus/upstream"}),
        "verification": "Source bytes and inventory unchanged during capture; compilation unverified.",
    }
    encoded_manifest = (json.dumps(manifest, indent=2) + "\n").encode()
    (destination / "manifest.json").write_bytes(encoded_manifest)
    print(json.dumps({"destination": str(destination), "paths": len(records), "manifest_sha256": hashlib.sha256(encoded_manifest).hexdigest()}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("name", help="New checkpoint directory name")
    arguments = parser.parse_args()
    if not arguments.name or Path(arguments.name).name != arguments.name or arguments.name in {".", ".."}:
        parser.error("name must be one directory component")
    root = Path(__file__).resolve().parents[1]
    capture(root, root / "artifacts" / "source-checkpoints" / arguments.name)


if __name__ == "__main__":
    main()
