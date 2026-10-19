#!/usr/bin/env python3
"""Print the CHANGELOG.md section of a release: the text under `## [version]` up to the next `## [`.

python3 tools/ci/release_notes.py v0.20.0 > notes.md   # exits 1 when there is no such section or it is empty
"""

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT


def section(changelog: str, version: str) -> str:
    version = version.removeprefix("v")
    lines, inside = [], False
    for line in changelog.splitlines():
        if line.startswith("## ["):
            if inside:
                break
            inside = line.startswith(f"## [{version}]")
            continue
        if inside:
            lines.append(line)
    return "\n".join(lines).strip("\n") + "\n" if "".join(lines).strip() else ""


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "version", help="the version or tag, with or without the leading v"
    )
    parser.add_argument("--changelog", type=Path, default=ROOT / "CHANGELOG.md")
    args = parser.parse_args(argv)
    notes = section(args.changelog.read_text(encoding="utf-8"), args.version)
    if not notes:
        print(
            f"{args.changelog.name} has no section for {args.version.removeprefix('v')}",
            file=sys.stderr,
        )
        return 1
    sys.stdout.write(notes)
    return 0


if __name__ == "__main__":
    sys.exit(main())
