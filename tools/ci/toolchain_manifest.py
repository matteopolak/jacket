#!/usr/bin/env python3
"""Write `toolchain-manifest.json`: which toolchain release the editors download, and each archive's SHA-256.

The release workflow publishes the file next to the archives; the VS Code extension and the Neovim plugin
ship a copy (or, in a development checkout, fetch the release's) and hold every download to it.

    python3 tools/ci/toolchain_manifest.py --version 0.20.0 --archives dist/ --out toolchain-manifest.json
    python3 tools/ci/toolchain_manifest.py --version 0.20.0 --sums SHA256SUMS --out toolchain-manifest.json

Format: {"version": "0.20.0", "sha256": {"jacket-linux-x64.tar.gz": "<hex>", ...}}, archives sorted by name.
Fixtures shared with the editors' tests: editors/fixtures/.
"""

import argparse
import json
import re
import sys
from pathlib import Path


from toolkit.hashing import sha256_file

ARCHIVE = re.compile(r"jacket-(?!vscode-)[a-z0-9-]+\.(?:tar\.gz|zip)")
VERSION = re.compile(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?")
SUM_LINE = re.compile(r"([0-9a-fA-F]{64})\s+\*?(\S+)")


def from_sums(text: str) -> dict[str, str]:
    sums = {}
    for line in text.splitlines():
        match = SUM_LINE.fullmatch(line.strip())
        if match and ARCHIVE.fullmatch(match[2]):
            sums[match[2]] = match[1].lower()
    return sums


def from_archives(directory: Path) -> dict[str, str]:
    return {
        path.name: sha256_file(path)
        for path in sorted(directory.iterdir())
        if ARCHIVE.fullmatch(path.name)
    }


def manifest(version: str, sha256: dict[str, str]) -> dict:
    version = version.removeprefix("v")
    if not VERSION.fullmatch(version):
        raise ValueError(f"not a version: {version!r}")
    if not sha256:
        raise ValueError("no jacket-* archives found to list")
    return {"version": version, "sha256": dict(sorted(sha256.items()))}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--version", required=True)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--sums", type=Path, help="a SHA256SUMS file")
    source.add_argument(
        "--archives", type=Path, help="a folder holding the release archives"
    )
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        sums = (
            from_sums(args.sums.read_text())
            if args.sums
            else from_archives(args.archives)
        )
        result = manifest(args.version, sums)
    except (ValueError, OSError) as error:
        print(f"toolchain_manifest: {error}", file=sys.stderr)
        return 1
    args.out.write_text(json.dumps(result, indent=2) + "\n")
    print(f"{args.out}: jaic {result['version']}: {', '.join(result['sha256'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
