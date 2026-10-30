#!/usr/bin/env python3
"""Write the checked-in copies of the compiler version from the root Cargo.toml.

    python3 tools/generators/generate_version_files.py [--check]

`jaifmt/version.jai` is what `jaifmt --version` prints; `editors/nvim/lua/jai/version.lua` is the toolchain
release the Neovim plugin downloads. Both are plain text with no formatting step, so the generator needs no
`jaifmt`. `tools/bump_version.py` runs it after editing Cargo.toml.
"""

import argparse
import sys
from toolkit.repo import ROOT
from toolkit.version import workspace_version

OUTPUTS = ["jaifmt/version.jai", "editors/nvim/lua/jai/version.lua"]

JAIFMT = """// The release this jaifmt belongs to. Generated from Cargo.toml by tools/generators/generate_version_files.py.
JAIFMT_VERSION :: "{version}";
"""
NVIM = """---The toolchain release this plugin downloads. Generated from Cargo.toml by
---tools/generators/generate_version_files.py; the release workflow stamps the release's version into
---the published plugin.
return '{version}'
"""


def render(version):
    """Each output path with the text it must hold."""
    return {OUTPUTS[0]: JAIFMT.format(version=version), OUTPUTS[1]: NVIM.format(version=version)}


def stale(root):
    """The outputs under `root` that differ from what the root's Cargo.toml version produces."""
    wanted = render(workspace_version(root))
    return [rel for rel, text in wanted.items() if not (root / rel).is_file() or (root / rel).read_text() != text]


def write(root):
    for rel, text in render(workspace_version(root)).items():
        (root / rel).write_text(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true", help="fail if a checked-in file differs instead of writing it")
    args = parser.parse_args()
    if args.check:
        problems = stale(ROOT)
        if problems:
            print("stale generated files: " + ", ".join(problems), file=sys.stderr)
            return 1
        return 0
    write(ROOT)
    return 0


if __name__ == "__main__":
    sys.exit(main())
