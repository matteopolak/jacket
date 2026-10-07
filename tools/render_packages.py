#!/usr/bin/env python3
"""Render the Homebrew formula and winget manifests for a release from packaging/.

The version and checksums come from the release itself: its tag and its SHA256SUMS file.

    python3 tools/render_packages.py --version 0.3.0 --sums SHA256SUMS --out packages

writes packages/homebrew/jai.rb and packages/winget/manifests/m/matteopolak/jai/0.3.0/*.yaml.
See docs/tools/package-managers.md.
"""
import argparse
import datetime
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEMPLATES = ROOT / "packaging"
REPOSITORY = "matteopolak/jai"
WINGET_ID = "matteopolak.jai"

# Placeholder -> release asset (without its `<prefix>-`) whose SHA-256 it takes.
PLATFORM_ASSETS = {
    "SHA256_MACOS_ARM64": "macos-arm64.tar.gz",
    "SHA256_LINUX_X64": "linux-x64.tar.gz",
    "SHA256_WINDOWS_X64": "windows-x64.zip",
    "SHA256_WINDOWS_ARM64": "windows-arm64.zip",
}
# Platforms built since the release after 0.4.1: (placeholder suffix, asset, the asset a release
# without it falls back to, the architecture the formula then requires on that OS). A release
# up to 0.4.1 (or a dry run of the workflow at such a version) may lack them; a later one may not.
NEWER_PLATFORMS = [
    ("MACOS_INTEL", "macos-x64.tar.gz", "macos-arm64.tar.gz", "MACOS_ARCH", ":arm64"),
    ("LINUX_ARM", "linux-arm64.tar.gz", "linux-x64.tar.gz", "LINUX_ARCH", ":x86_64"),
]
LAST_WITHOUT_NEWER_PLATFORMS = (0, 4, 1)
# Archives are `jai-<platform>` since 0.4.1; 0.4.0 and earlier shipped `jaic-<platform>`.
LAST_JAIC_NAMED = (0, 4, 0)
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


def version_core(version):
    """(major, minor, patch) of `version`."""
    return tuple(int(part) for part in re.match(r"(\d+)\.(\d+)\.(\d+)", version).groups())


def expected_prefix(version):
    """The archive prefix a release of `version` was published with: `jaic` up to 0.4.0, else `jai`."""
    return "jaic" if version_core(version) <= LAST_JAIC_NAMED else "jai"


def archive_prefix(version, sums):
    """The prefix whose archives `sums` lists for every platform: the one `version` was released
    with, else the other (a dry run of the release workflow builds `jai-*` archives at whatever
    version Cargo.toml has, 0.4.0 included)."""
    preferred = expected_prefix(version)
    other = "jai" if preferred == "jaic" else "jaic"
    for prefix in (preferred, other):
        if all(f"{prefix}-{asset}" in sums for asset in PLATFORM_ASSETS.values()):
            return prefix
    missing = [f"{preferred}-{asset}" for asset in PLATFORM_ASSETS.values() if f"{preferred}-{asset}" not in sums]
    raise ValueError(f"SHA256SUMS has no entry for {', '.join(missing)}")


def substitutions(version, sums, base_url, release_date):
    if not VERSION.fullmatch(version):
        raise ValueError(f"version {version!r} is not x.y.z (without the leading v)")
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", release_date):
        raise ValueError(f"release date {release_date!r} is not YYYY-MM-DD")
    values = {
        "VERSION": version,
        "BASE_URL": base_url or f"https://github.com/{REPOSITORY}/releases/download/v{version}",
        "RELEASE_DATE": release_date,
    }
    prefix = archive_prefix(version, sums)
    values["ARCHIVE_PREFIX"] = prefix
    for key, asset in PLATFORM_ASSETS.items():
        values[key] = sums[f"{prefix}-{asset}"]
    newer_required = version_core(version) > LAST_WITHOUT_NEWER_PLATFORMS
    for name, asset, fallback, arch_key, arch in NEWER_PLATFORMS:
        if f"{prefix}-{asset}" in sums:
            values[f"{name}_ASSET"] = asset
            values[f"SHA256_{name}"] = sums[f"{prefix}-{asset}"]
            values[arch_key] = None
        elif newer_required:
            raise ValueError(f"SHA256SUMS has no entry for {prefix}-{asset}")
        else:
            values[f"{name}_ASSET"] = fallback
            values[f"SHA256_{name}"] = sums[f"{prefix}-{fallback}"]
            values[arch_key] = arch
    return values


def render(template, values, uppercase_hashes=False):
    """Fill in the @PLACEHOLDERS@. A line holding one whose value is None is left out, together
    with a blank line right after it."""
    def replace(match):
        key = match.group(1)
        if key not in values:
            raise ValueError(f"unknown placeholder @{key}@")
        value = values[key]
        return value.upper() if uppercase_hashes and key.startswith("SHA256_") else value

    lines = []
    dropped = False
    for line in template.splitlines(keepends=True):
        if dropped and not line.strip():
            dropped = False
            continue
        dropped = any(values.get(key, "") is None for key in PLACEHOLDER.findall(line))
        if not dropped:
            lines.append(PLACEHOLDER.sub(replace, line))
    return "".join(lines)


def render_all(values, out):
    """Write every package file under `out`; returns the paths written."""
    written = []
    formula = out / "homebrew" / "jai.rb"
    formula.parent.mkdir(parents=True, exist_ok=True)
    formula.write_text(render((TEMPLATES / "homebrew" / "jai.rb.tmpl").read_text(), values))
    written.append(formula)
    # winget's convention is upper-case hex, under manifests/<first letter>/<publisher>/<name>/<version>.
    publisher, name = WINGET_ID.split(".", 1)
    manifests = out / "winget" / "manifests" / publisher[0].lower() / publisher / name / values["VERSION"]
    manifests.mkdir(parents=True, exist_ok=True)
    for template in sorted((TEMPLATES / "winget").glob("*.yaml.tmpl")):
        target = manifests / template.name.removesuffix(".tmpl")
        target.write_text(render(template.read_text(), values, uppercase_hashes=True))
        written.append(target)
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
    parser.add_argument("--release-date", default=datetime.date.today().isoformat(),
                        help="winget ReleaseDate, YYYY-MM-DD (default: today)")
    args = parser.parse_args(argv)
    try:
        sums = parse_sums(args.sums.read_text()) if args.sums else hash_archives(args.archives)
        values = substitutions(args.version.removeprefix("v"), sums, args.base_url, args.release_date)
        for path in render_all(values, args.out):
            print(path)
    except ValueError as error:
        sys.exit(f"error: {error}")


if __name__ == "__main__":
    main()
