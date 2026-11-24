#!/usr/bin/env python3
"""Prints the stdlib module folders whose own sources are written for one OS, one per line.

A module counts as specific to an OS when a source file outside its `tests/` folder names that
OS's APIs or its `OS` constant (`PATTERNS`). CI's macOS legs run only these modules' tests
(`JACKET_TEST_PLATFORM_ONLY`, tools/ci/stdlib_jacket_test.sh): the rest behaves the same on every
host and Linux x64 already runs it.

    python3 tools/ci/platform_modules.py            # the host's OS
    python3 tools/ci/platform_modules.py windows
"""

import re
import sys

from toolkit.repo import ROOT

STDLIB = ROOT / "stdlib"
# Folders whose children are module folders.
CONTAINERS = ("", "Extensions", "internal")
SOURCES = {".jai", ".m", ".mm", ".c", ".h", ".cpp"}
PATTERNS = {
    "macos": r"\.MACOS\b|\bDarwin\b|Objective_C|\bAppKit\b|\bCocoa\b|CoreFoundation|<Metal/|<Cocoa/",
    "linux": r"\.LINUX\b|<sys/epoll\.h>|<linux/",
    "windows": r"\.WINDOWS\b|<windows\.h>|\bkernel32\b",
}
HOSTS = {"darwin": "macos", "linux": "linux", "win32": "windows"}


def modules(os_name):
    """The module folders (relative to the repository) that have source for `os_name`."""
    pattern = re.compile(PATTERNS[os_name])
    found = []
    for container in CONTAINERS:
        for folder in sorted((STDLIB / container).iterdir()):
            if not folder.is_dir() or folder.name in ("Extensions", "internal"):
                continue
            if any(
                path.suffix in SOURCES
                and "tests" not in path.relative_to(folder).parts
                and pattern.search(path.read_text(errors="replace"))
                for path in folder.rglob("*")
                if path.is_file()
            ):
                found.append(str(folder.relative_to(ROOT)))
    return found


def main(argv):
    os_name = argv[1] if len(argv) > 1 else HOSTS.get(sys.platform, "linux")
    if os_name not in PATTERNS:
        sys.exit(f"platform_modules: OS is one of {', '.join(PATTERNS)}")
    print("\n".join(modules(os_name)))


if __name__ == "__main__":
    main(sys.argv)
