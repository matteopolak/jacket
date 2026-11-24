"""Pins the Debian packages `jacket toolchain add` unpacks into a Linux sysroot.

Reads each architecture's `Packages.xz` from one Debian snapshot and writes the file name, size and
sha256 of every package a sysroot needs to `crates/jaic-toolchain/data/sysroots.txt`. The Rust code
only reads that file, so a build never asks Debian anything.

    PYTHONPATH=tools python3 tools/embedded/pin_sysroots.py            # rewrite the pins
    PYTHONPATH=tools python3 tools/embedded/pin_sysroots.py --check    # fail if they are stale

To move to a newer snapshot or release, change `SNAPSHOT` or `RELEASE` and rewrite the pins.
"""

import argparse
import lzma
import sys
from pathlib import Path

from toolkit.download import fetch
from toolkit.repo import ROOT

OUTPUT = ROOT / "crates/jaic-toolchain/data/sysroots.txt"

# One snapshot of the archive: the same bytes forever, so a pin never moves under a build.
SNAPSHOT = "20251201T000000Z"
RELEASE = "bookworm"
MIRROR = f"https://snapshot.debian.org/archive/debian/{SNAPSHOT}"

# Debian's architecture names, as `jaic-toolchain/src/pins.rs` maps targets to them.
ARCHITECTURES = ["armhf", "arm64", "i386", "amd64", "s390x"]

# What a link and a run need: the C library and its headers, the kernel headers, the C
# library's own helper programs are not needed. `libgcc-12-dev` holds `crtbegin.o` and
# `libgcc.a`; `libgcc-s1` the unwinder the C library loads.
PACKAGES = [
    "libc6",
    "libc6-dev",
    "linux-libc-dev",
    "libcrypt1",
    "libcrypt-dev",
    "libgcc-s1",
    "libgcc-12-dev",
    "gcc-12-base",
]


def parse(text: str) -> dict[str, dict[str, str]]:
    """The stanzas of a Packages file by package name."""
    packages: dict[str, dict[str, str]] = {}
    for stanza in text.split("\n\n"):
        fields: dict[str, str] = {}
        key = ""
        for line in stanza.splitlines():
            if line[:1] in (" ", "\t"):
                fields[key] += "\n" + line
            elif ":" in line:
                key, _, value = line.partition(":")
                fields[key] = value.strip()
        if "Package" in fields:
            packages[fields["Package"]] = fields
    return packages


def pins() -> list[str]:
    lines = [
        f"# Debian {RELEASE}, snapshot {SNAPSHOT}. Written by tools/embedded/pin_sysroots.py.",
        f"mirror {MIRROR}",
        "# architecture package version size sha256 path-under-the-mirror",
    ]
    for arch in ARCHITECTURES:
        url = f"{MIRROR}/dists/{RELEASE}/main/binary-{arch}/Packages.xz"
        index = parse(lzma.decompress(fetch(url)).decode())
        for name in PACKAGES:
            entry = index.get(name)
            if entry is None:
                raise SystemExit(f"{name} is not in {url}")
            lines.append(
                " ".join(
                    [
                        arch,
                        name,
                        entry["Version"],
                        entry["Size"],
                        entry["SHA256"],
                        entry["Filename"],
                    ]
                )
            )
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="fail when the file differs")
    args = parser.parse_args()
    text = "\n".join(pins()) + "\n"
    if args.check:
        if OUTPUT.read_text() != text:
            print(f"{OUTPUT} is stale: run tools/embedded/pin_sysroots.py", file=sys.stderr)
            return 1
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(text)
    print(f"wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
