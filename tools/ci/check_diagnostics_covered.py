#!/usr/bin/env python3
"""Fail when a compiler diagnostic code has no test that expects it (docs/tools/diagnostic-coverage.md).

Usage: tools/ci/check_diagnostics_covered.py [--update] [--list]

Every diagnostic the compiler reports has a code, defined once in `crates/jaic-syntax/src/diagnostics/catalog.rs`
(`Variant = "kebab-name", Severity, "summary"`). This script reads that table and looks for each code in the test
corpus: every tracked text file under `tests/`, every `crates/*/tests/` file, the golden files of the standard
library's own tests, test modules inside `crates/*/src` and `tools/**/test_*`. A code is tested when a test names it as

- `[name]` (the header of a rendered diagnostic: `error[type-mismatch]`),
- `"name"` (a JSON field, as the negative corpus manifest's `code` or the language server's diagnostic code), or
- `Code::Variant` (a Rust test that builds or matches the code).

The untested codes recorded in `tools/ci/untested_codes.txt` are the known gap; the list only shrinks:

- a new untested code fails the check (add a test that expects it, usually a `tests/corpus/negative` case);
- a listed code that is now tested, or that no longer exists, fails until it is removed (`--update`).

`--update` rewrites the list from the tree; review its diff, a new line is a code added without a test.
"""

import argparse
import re
import subprocess
import sys
from toolkit.repo import ROOT

CATALOG = "crates/jaic-syntax/src/diagnostics/catalog.rs"
ALLOWLIST = ROOT / "tools/ci/untested_codes.txt"
ENTRY = re.compile(r'^\s+(\w+) = "([a-z0-9-]+)",\s*(Error|Warning)\b', re.M)
# Codes reported by tools around the compiler (a usage mistake, a crash) are tested by the tools' own tests
# like every other code; nothing is exempt.

TEXT_SUFFIXES = (
    ".rs",
    ".py",
    ".json",
    ".txt",
    ".jai",
    ".md",
    ".lua",
    ".ts",
    ".mjs",
    ".snap",
    ".out",
    ".stderr",
    ".stdout",
)


def tracked(patterns):
    out = subprocess.check_output(["git", "ls-files", "-z", "--", *patterns], cwd=ROOT)
    return [p.decode() for p in out.split(b"\0") if p]


def is_test_path(path: str) -> bool:
    name = path.rsplit("/", 1)[-1]
    return (
        path.startswith("tests/")
        or "/tests/" in path
        or name in ("tests.rs",)
        or name.endswith("_tests.rs")
        or name.startswith("test_")
    )


def strip_test_modules(text: str) -> tuple[str, str]:
    """(production code, test code) of a Rust file: everything from a `#[cfg(test)]` module on is test code."""
    match = re.search(r"^#\[cfg\(test\)\]\s*\n\s*(?:pub\s+)?mod\s", text, re.M)
    if match:
        return text[: match.start()], text[match.start() :]
    return text, ""


def catalog_codes(text: str):
    """[(variant, name)] of the table in catalog.rs, in order."""
    return [(m.group(1), m.group(2)) for m in ENTRY.finditer(text)]


def corpus() -> str:
    """All text a test may expect a code in, with Rust string escapes read back."""
    parts = []
    for path in tracked(["tests", "crates", "tools", "editors", "stdlib"]):
        in_src = (
            path.startswith("crates/")
            and "/src/" in path
            and path.endswith(".rs")
            and not is_test_path(path)
        )
        if not is_test_path(path) and not in_src:
            continue
        if not path.endswith(TEXT_SUFFIXES):
            continue
        try:
            text = (ROOT / path).read_text()
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        if in_src:
            text = strip_test_modules(text)[1]
        if path.endswith(".rs"):
            text = text.replace('\\"', '"').replace("\\\\", "\\")
        parts.append(text)
    return "\n".join(parts)


def is_tested(variant: str, name: str, text: str) -> bool:
    return (
        f"[{name}]" in text
        or f'"{name}"' in text
        or re.search(rf"\bCode::{variant}\b", text) is not None
    )


def untested(codes, text):
    """Names of the codes in `codes` ([(variant, name)]) that `text` never expects."""
    return {name for variant, name in codes if not is_tested(variant, name, text)}


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--update", action="store_true", help="rewrite the list of untested codes"
    )
    parser.add_argument("--list", action="store_true", help="print the untested codes")
    args = parser.parse_args()
    codes = catalog_codes((ROOT / CATALOG).read_text())
    names = {name for _, name in codes}
    missing = untested(codes, corpus())
    if args.update:
        ALLOWLIST.write_text("".join(f"{n}\n" for n in sorted(missing)))
        print(
            f"{len(codes)} codes, {len(missing)} untested; wrote {ALLOWLIST.relative_to(ROOT)}"
        )
        return 0
    if args.list:
        for name in sorted(missing):
            print(name)
        return 0
    allowed = {line.strip() for line in ALLOWLIST.read_text().splitlines() if line}
    errors = []
    for name in sorted(missing - allowed):
        errors.append(
            f"untested code: {name}\n  add a test that expects it (a tests/corpus/negative case with `code`, a rejection program or a crate test)"
        )
    for name in sorted(allowed - missing):
        why = "is now tested" if name in names else "no longer exists"
        errors.append(
            f"{name} {why}; remove it from tools/ci/untested_codes.txt (python3 tools/ci/check_diagnostics_covered.py --update)"
        )
    for e in errors:
        print(e, file=sys.stderr)
    print(
        f"{len(codes)} codes, {len(missing)} untested, {len(allowed)} allowlisted",
        file=sys.stderr,
    )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
