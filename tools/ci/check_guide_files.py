#!/usr/bin/env python3
"""Check that a guide's `jai` block headed `// <path>.jai` is a stretch of that file in the guide's examples.

A guide `docs/guides/<name>.md` shows files of the projects under `docs/examples/<name>/`. A block whose
first line is a comment naming a file (`// slugs/module.jai`) must be the file, or a run of its consecutive
lines (a guide may show a file in pieces); blank lines and trailing spaces do not count.

    PYTHONPATH=tools python3 tools/ci/check_guide_files.py
"""

import re
import sys

from toolkit.repo import ROOT

FENCE = re.compile(r"^```jai\b[^\n]*\n(.*?)^```", re.M | re.S)
HEADER = re.compile(r"^// ([\w./-]+\.jai)\n")


def guide_blocks(text):
    """(line, path, body) for each block that names a file."""
    for match in FENCE.finditer(text):
        header = HEADER.match(match.group(1))
        if header:
            line = text.count("\n", 0, match.start()) + 1
            yield line, header.group(1), match.group(1)[header.end():]


def significant(text):
    return [line.rstrip() for line in text.splitlines() if line.strip()]


def contains_run(whole, part):
    """Whether `part` is consecutive lines of `whole`."""
    return any(whole[i:i + len(part)] == part for i in range(len(whole) - len(part) + 1))


def problems(name, text, example_root):
    """Mismatches between the blocks of guide `name` (its text) and the files under `example_root`."""
    errors = []
    for line, path, body in guide_blocks(text):
        where = f"docs/guides/{name}.md:{line}"
        files = sorted(example_root.glob(f"**/{path}"))
        if not files:
            errors.append(f"{where}: names {path}, which is not under docs/examples/{name}/")
        elif not any(contains_run(significant(f.read_text()), significant(body)) for f in files):
            errors.append(f"{where}: is not a run of lines of docs/examples/{name}/**/{path}")
    return errors


def main():
    errors = []
    for guide in sorted((ROOT / "docs/guides").glob("*.md")):
        root = ROOT / "docs/examples" / guide.stem
        if root.is_dir():
            errors += problems(guide.stem, guide.read_text(), root)
    for e in errors:
        print(e, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
