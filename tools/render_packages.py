#!/usr/bin/env python3
"""Render the Homebrew formula for a release from packaging/.

The version and checksums come from the release itself: its tag and its SHA256SUMS file.

    python3 tools/render_packages.py --version 0.3.0 --sums SHA256SUMS --out packages

writes packages/homebrew/jacket.rb.
See docs/tools/package-managers.md.
"""
import argparse
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEMPLATES = ROOT / "packaging"
REPOSITORY = "matteopolak/jacket"

# Placeholder -> release asset whose SHA-256 it takes.
PLATFORM_ASSETS = {
    "SHA256_MACOS_ARM64": "jacket-macos-arm64.tar.gz",
    "SHA256_MACOS_INTEL": "jacket-macos-x64.tar.gz",
    "SHA256_LINUX_X64": "jacket-linux-x64.tar.gz",
    "SHA256_LINUX_ARM": "jacket-linux-arm64.tar.gz",
}
VERSION = re.compile(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?")
PLACEHOLDER = re.compile(r"@([A-Z0-9_]+)@")


def parse_sums(text):
    """`sha256sum` output (`<hex>  <name>`, optionally `*<name>`) as {name: hex}."""
    sums = {}
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        match = re.fullmatch(r"([0-9a-fA-F]{64}) [ *](\S.*)", line.strip())
        if not match:
            raise ValueError(f"SHA256SUMS line {number} is not `<sha256>  <file>`: {line!r}")
        sums[match.group(2)] = match.group(1).lower()
    return sums


def hash_archives(directory):
    """{name: sha256} of the files in `directory`, as `sha256sum *` would list them."""
    sums = {}
    for path in sorted(Path(directory).iterdir()):
        if path.is_file():
            sums[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    return sums


def substitutions(version, sums, base_url):
    if not VERSION.fullmatch(version):
        raise ValueError(f"version {version!r} is not x.y.z (without the leading v)")
    missing = [asset for asset in PLATFORM_ASSETS.values() if asset not in sums]
    if missing:
        raise ValueError(f"SHA256SUMS has no entry for {', '.join(missing)}")
    values = {
        "VERSION": version,
        "BASE_URL": base_url or f"https://github.com/{REPOSITORY}/releases/download/v{version}",
    }
    for key, asset in PLATFORM_ASSETS.items():
        values[key] = sums[asset]
    return values


def render(template, values):
    """Fill in the @PLACEHOLDERS@."""
    def replace(match):
        key = match.group(1)
        if key not in values:
            raise ValueError(f"unknown placeholder @{key}@")
        return values[key]

    return PLACEHOLDER.sub(replace, template)


def render_all(values, out):
    """Write every package file under `out`; returns the paths written."""
    written = []
    formula = out / "homebrew" / "jacket.rb"
    formula.parent.mkdir(parents=True, exist_ok=True)
    formula.write_text(render((TEMPLATES / "homebrew" / "jacket.rb.tmpl").read_text(), values))
    written.append(formula)
    return written


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--version", required=True, help="release version, x.y.z (a leading v is dropped)")
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--sums", type=Path, help="the release's SHA256SUMS file")
    source.add_argument("--archives", type=Path,
                        help="a directory holding the release archives, hashed here (dry runs)")
    parser.add_argument("--out", required=True, type=Path, help="directory to write into")
    parser.add_argument("--base-url", help="where the assets are (default: the GitHub release of the tag)")
    args = parser.parse_args(argv)
    try:
        sums = parse_sums(args.sums.read_text()) if args.sums else hash_archives(args.archives)
        values = substitutions(args.version.removeprefix("v"), sums, args.base_url)
        for path in render_all(values, args.out):
            print(path)
    except ValueError as error:
        sys.exit(f"error: {error}")


if __name__ == "__main__":
    main()
