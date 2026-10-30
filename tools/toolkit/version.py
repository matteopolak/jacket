"""The compiler version: `[workspace.package] version` of the root Cargo.toml.

    workspace_version(root)   the X.Y.Z string; exits when the manifest has none

Everything else that names the version (the Neovim plugin, jaifmt, the release tooling) derives it
from here; `tools/generators/generate_version_files.py` writes the two checked-in copies.
"""

import re

WORKSPACE_VERSION = re.compile(r'(\[workspace\.package\][^\[]*?^version = ")([^"]+)(")', re.M | re.S)

__all__ = ["WORKSPACE_VERSION", "workspace_version"]


def workspace_version(root):
    match = WORKSPACE_VERSION.search((root / "Cargo.toml").read_text())
    if not match:
        raise SystemExit("Cargo.toml has no [workspace.package] version")
    return match.group(2)
