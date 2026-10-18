#!/usr/bin/env python3
"""Fail when a tracked Rust file is too long or a Rust or Python file carries a banner comment.

Length: a `.rs` file may have at most MAX_LINES lines. Files already over the limit are listed in
ALLOWED_LONG with their current line count. The list only shrinks: a listed file may not grow past
its recorded count, and a listed file that is back under the limit (or gone) must be removed from
the list. Never add a new entry; split the file instead.

Banners: a comment made only of rule characters (`// ----`, `# ====`) is rejected. Put the section
name on a single plain comment line, or split the file. Jai files are covered by the jailint
`banner_comment` rule.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAX_LINES = 1200

# path -> line count it may not grow past. Remove an entry once the file is split.
ALLOWED_LONG = {
}

BANNER = re.compile(r"^\s*(//|#)\s*[-=*#~]{3,}\s*$")


def tracked(patterns):
    out = subprocess.check_output(["git", "ls-files", "-z", "--", *patterns], cwd=ROOT)
    return [p.decode() for p in out.split(b"\0") if p]


def line_count(text):
    return text.count("\n") + (0 if text == "" or text.endswith("\n") else 1)


def check_lengths(counts, allowed=None, limit=MAX_LINES):
    """Errors for `counts` (path -> lines) against the `allowed` over-limit list."""
    allowed = ALLOWED_LONG if allowed is None else allowed
    errors = []
    for path, n in sorted(counts.items()):
        cap = allowed.get(path)
        if cap is None:
            if n > limit:
                errors.append(f"{path}: {n} lines, over the {limit}-line limit; split it into modules")
        elif n > cap:
            errors.append(f"{path}: grew to {n} lines, past its allowlisted {cap}; split it (or move part of it into a module) instead of growing it")
        elif n <= limit:
            errors.append(f"{path}: now {n} lines; remove it from ALLOWED_LONG in check_source_style.py")
        elif n < cap:
            errors.append(f"{path}: shrank to {n} lines; lower its ALLOWED_LONG entry from {cap}")
    for path in sorted(allowed):
        if path not in counts:
            errors.append(f"{path}: gone; remove it from ALLOWED_LONG in check_source_style.py")
    return errors


def check_banners(name, text):
    return [f"{name}:{i}: banner comment; use one plain comment line or split the file"
            for i, line in enumerate(text.splitlines(), 1) if BANNER.match(line)]


def main():
    counts, errors = {}, []
    for path in tracked(["*.rs"]):
        if (ROOT / path).is_file():
            counts[path] = line_count((ROOT / path).read_text(encoding="utf-8", errors="replace"))
    errors += check_lengths(counts)
    for path in tracked(["*.rs", "*.py"]):
        if (ROOT / path).is_file():
            errors += check_banners(path, (ROOT / path).read_text(encoding="utf-8", errors="replace"))
    for e in errors:
        print(e, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
