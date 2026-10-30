#!/usr/bin/env python3
"""Fail when a compiler diagnostic has no test that expects its text (docs/tools/diagnostic-coverage.md).

Usage: tools/ci/check_diagnostics_covered.py [--update] [--list]

The compiler's errors and warnings are string templates at call sites (`err(span, format!("..."))`,
`Diagnostic::error(..)`, `Diagnostic::warning(..)`, the parser's and lexer's `error`/`err`). This script finds
those call sites in `crates/jaic/src` (outside test code), turns each template into a pattern (`{...}`
placeholders match any text on the line) and looks for it in the test corpus: every file under `tests/`, every
`crates/*/tests/` file, test modules inside `crates/*/src` and `tools/**/test_*`. A template that no test text
matches is "untested". The untested ones recorded in `tools/ci/untested_diagnostics.txt` are the known gap; the
list only shrinks:

- a new untested template fails the check (add a test that expects its text, usually a `tests/corpus/negative`
  case or a `tests/compiler/rejections` program);
- a listed template that is now tested, or that no longer exists, fails until it is removed (`--update`).

`--update` rewrites the list from the tree; review its diff, a new line is a diagnostic added without a test.
"""

import argparse
import re
import subprocess
import sys
from toolkit.repo import ROOT

SOURCE_DIR = "crates/jaic/src"
ALLOWLIST = ROOT / "tools/ci/untested_diagnostics.txt"
# Calls whose string argument is a diagnostic message.
CALL = re.compile(
    r"(?<![A-Za-z0-9_])(?:self\.error|self\.err|err)\(|Diagnostic::(?:error|warning)\("
)
# A template needs at least this much literal text to be matched against the corpus.
MIN_LITERAL = 6
# Literal characters of a template, from its start, that a test must quote.
PREFIX = 14


def tracked(patterns):
    out = subprocess.check_output(["git", "ls-files", "-z", "--", *patterns], cwd=ROOT)
    return [p.decode() for p in out.split(b"\0") if p]


def is_test_path(path: str) -> bool:
    name = path.rsplit("/", 1)[-1]
    return (
        "/tests/" in path
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


def read_string(text: str, i: int):
    """The Rust string literal starting at text[i] (`"` or a raw string): (decoded, index after it) or None."""
    raw = re.compile(r'r(#*)"').match(text, i)
    if raw:
        end = text.find('"' + raw.group(1), raw.end())
        return (
            (text[raw.end() : end], end + 1 + len(raw.group(1))) if end >= 0 else None
        )
    if text[i] != '"':
        return None
    out, j = [], i + 1
    while j < len(text):
        c = text[j]
        if c == "\\":
            n = text[j + 1]
            if (
                n == "\n"
            ):  # a line continuation drops the newline and the next line's indentation
                j += 2
                while j < len(text) and text[j] in " \t\n":
                    j += 1
                continue
            out.append(
                {"n": "\n", "t": "\t", '"': '"', "\\": "\\", "'": "'", "0": "\0"}.get(
                    n, "\x00"
                )
            )
            j += 2 if n != "u" else text.index("}", j) + 1 - j
            continue
        if c == '"':
            return "".join(out), j + 1
        out.append(c)
        j += 1
    return None


def split_arguments(text: str, start: int) -> list[str]:
    """The top-level arguments of the call whose `(` is at text[start - 1]; strings and nesting are skipped."""
    args, depth, i, begin = [], 1, start, start
    while i < len(text) and depth:
        c = text[i]
        if c == '"' or (
            c == "r"
            and re.compile(r'r#*"').match(text, i)
            and not text[i - 1].isalnum()
        ):
            literal = read_string(text, i)
            i = literal[1] if literal else i + 1
            continue
        if (
            c == "'"
            and i + 2 < len(text)
            and (text[i + 2] == "'" or text[i + 1] == "\\")
        ):
            i += text.index("'", i + 2) + 1 - i if text[i + 1] == "\\" else 3
            continue
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                args.append(text[begin:i])
        elif c == "," and depth == 1:
            args.append(text[begin:i])
            begin = i + 1
        i += 1
    return args


MESSAGE_ARG = re.compile(r'\s*&?\s*(?:format!\(|String::from\(|)\s*(r#*"|")')


def templates(text: str):
    """Message templates (literal text with `{}` for each placeholder) of the diagnostics built in `text`."""
    found = []
    for call in CALL.finditer(text):
        for arg in split_arguments(text, call.end()):
            m = MESSAGE_ARG.match(arg)
            if not m:
                continue
            literal = read_string(arg, m.start(1))
            if literal:
                found.append(
                    re.sub(
                        r"\{[^{}]*\}",
                        "{}",
                        literal[0].replace("{{", "\x01").replace("}}", "\x02"),
                    )
                    .replace("\x01", "{")
                    .replace("\x02", "}")
                )
            break
    return found


def production_templates():
    found = set()
    for path in tracked([SOURCE_DIR]):
        if is_test_path(path):
            continue
        production, _ = strip_test_modules((ROOT / path).read_text())
        found.update(t for t in templates(production) if t.strip())
    return found


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
)


def corpus() -> str:
    """All text a test may expect a diagnostic in, with Rust string escapes read back."""
    parts = []
    for path in tracked(["tests", "crates", "tools", "editors"]):
        in_src = (
            path.startswith("crates/")
            and "/src/" in path
            and path.endswith(".rs")
            and not is_test_path(path)
        )
        if not (
            path.startswith(("tests/", "crates/")) or is_test_path(path)
        ) or not path.endswith(TEXT_SUFFIXES):
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
    return re.sub(r"\\\n\s*", "", "\n".join(parts))


def pattern(template: str):
    """A regex for the start of the template inside one line, or None when it has too little literal text to mean anything.

    Tests usually quote part of a message, so only the first PREFIX literal characters are looked for
    (placeholders in that part match any text)."""
    pieces, left = [], PREFIX
    for piece in re.split(r"\{\}|\n", template):
        if not piece:
            continue
        pieces.append(piece[:left])
        left -= len(pieces[-1])
        if left <= 0:
            break
    if sum(len(p) for p in pieces) < MIN_LITERAL:
        return None
    return re.compile("[^\n]*?".join(re.escape(p) for p in pieces))


def untested(found, text):
    missing = set()
    for template in found:
        regex = pattern(template)
        if regex is None or not regex.search(text):
            missing.add(template)
    return missing


def encode(template: str) -> str:
    return template.replace("\\", "\\\\").replace("\n", "\\n")


def decode(line: str) -> str:
    return line.replace("\\n", "\n").replace("\\\\", "\\")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--update", action="store_true", help="rewrite the list of untested diagnostics"
    )
    parser.add_argument(
        "--list", action="store_true", help="print the untested diagnostics"
    )
    args = parser.parse_args()
    found = production_templates()
    missing = untested(found, corpus())
    if args.update:
        ALLOWLIST.write_text("".join(encode(t) + "\n" for t in sorted(missing)))
        print(
            f"{len(found)} diagnostics, {len(missing)} untested; wrote {ALLOWLIST.relative_to(ROOT)}"
        )
        return 0
    if args.list:
        for t in sorted(missing):
            print(encode(t))
        return 0
    allowed = {decode(line) for line in ALLOWLIST.read_text().splitlines() if line}
    errors = []
    for t in sorted(missing - allowed):
        errors.append(
            f"untested diagnostic: {encode(t)!r}\n  add a test that expects this text (tests/corpus/negative, tests/compiler/rejections or a crate test)"
        )
    for t in sorted(allowed - missing):
        why = "is now tested" if t in found else "no longer exists"
        errors.append(
            f"{encode(t)!r} {why}; remove it from tools/ci/untested_diagnostics.txt (python3 tools/ci/check_diagnostics_covered.py --update)"
        )
    for e in errors:
        print(e, file=sys.stderr)
    print(
        f"{len(found)} diagnostics, {len(missing)} untested, {len(allowed)} allowlisted",
        file=sys.stderr,
    )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
