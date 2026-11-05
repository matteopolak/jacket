#!/usr/bin/env python3
"""Check the complete published Pages tree, including retained versions and assets.

    python3 tools/ci/check_docs_size.py target/doc-site
    python3 tools/ci/check_docs_size.py /owned/site --limit-bytes 1000000000

The conservative default is 1,000,000,000 logical file bytes, before compression.
This read-only gate never removes files to make an artifact fit.
"""
import argparse
import json
import os
from pathlib import Path
import stat

DEFAULT_LIMIT_BYTES = 1_000_000_000


def _is_reparse(metadata):
    """Windows junctions can have directory mode; use their separate reparse attribute."""
    return bool(getattr(metadata, "st_file_attributes", 0)
                & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400))


def check_tree(directory, limit_bytes=DEFAULT_LIMIT_BYTES):
    """Count all regular published paths; e.g. check_tree(Path('target/doc-site'))."""
    if not isinstance(limit_bytes, int) or isinstance(limit_bytes, bool) or limit_bytes < 0:
        raise ValueError("limit bytes must be a nonnegative integer")
    root = Path(directory)
    try:
        root_metadata = root.lstat()
        mode = root_metadata.st_mode
    except OSError as error:
        raise ValueError("cannot inspect published root") from error
    if stat.S_ISLNK(mode):
        raise ValueError("symlink is not allowed: .")
    if _is_reparse(root_metadata):
        raise ValueError("reparse node is not allowed: .")
    if not stat.S_ISDIR(mode):
        raise ValueError("published root is not a directory")
    files, total = 0, 0
    pending = [root]
    while pending:
        parent = pending.pop()
        try:
            with os.scandir(parent) as entries:
                for entry in entries:
                    relative = Path(entry.path).relative_to(root).as_posix()
                    try:
                        metadata = entry.stat(follow_symlinks=False)
                    except OSError as error:
                        raise ValueError(f"cannot inspect published path: {relative}") from error
                    if stat.S_ISLNK(metadata.st_mode):
                        raise ValueError(f"symlink is not allowed: {relative}")
                    if _is_reparse(metadata):
                        raise ValueError(f"reparse node is not allowed: {relative}")
                    if stat.S_ISDIR(metadata.st_mode):
                        pending.append(Path(entry.path))
                    elif stat.S_ISREG(metadata.st_mode):
                        if metadata.st_size < 0:
                            raise ValueError(f"invalid published file size: {relative}")
                        files += 1
                        total += metadata.st_size
                    else:
                        raise ValueError(f"special node is not allowed: {relative}")
        except OSError as error:
            relative = parent.relative_to(root).as_posix()
            raise ValueError(f"cannot inspect published directory: {relative}") from error
    return {"status": "pass" if total <= limit_bytes else "fail", "files": files,
            "bytes": total, "limitBytes": limit_bytes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--limit-bytes", type=int, default=DEFAULT_LIMIT_BYTES)
    args = parser.parse_args()
    try:
        result = check_tree(args.directory, args.limit_bytes)
    except ValueError as error:
        print(json.dumps({"status": "error", "error": str(error)}, separators=(",", ":")))
        return 1
    print(json.dumps(result, separators=(",", ":")))
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
