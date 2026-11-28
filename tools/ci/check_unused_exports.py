#!/usr/bin/env python3
"""Fail when a `pub` item of a workspace library crate is referenced nowhere in the workspace.

`unreachable_pub` and `dead_code` cover what a crate keeps to itself; this covers what it exports:
`rust-analyzer scip` indexes every crate together, so an exported item with a definition and no
reference anywhere is dead. Items of test code, trait impls, enum variants, fields and `main` are not
checked. An item that is public on purpose (an API for outside users) goes in `ALLOWED`.

    PYTHONPATH=tools python3 tools/ci/check_unused_exports.py [--index FILE] [--rust-analyzer PATH]

Without `--index` it runs `rust-analyzer scip .` (the `rust-analyzer` rustup component).
"""

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from toolkit.repo import ROOT

# `crate/path/to/item` as `symbol_name` returns it, for items the index cannot see used: public API,
# and what an exported macro calls (references written inside a macro body are not indexed).
ALLOWED: frozenset[str] = frozenset()

DEFINITION = 1  # SymbolRole.Definition
ITEM = re.compile(
    r"^\s*pub\s+(?:unsafe\s+|const\s+|async\s+|extern\s+\"[^\"]*\"\s+)*(fn|struct|enum|trait|type|const|static|union|mod)\b"
)


def varint(data, at):
    value = shift = 0
    while True:
        byte = data[at]
        at += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, at
        shift += 7


def fields(data):
    """`(field number, wire type, value)` of a protobuf message; length-delimited values are bytes."""
    at = 0
    while at < len(data):
        key, at = varint(data, at)
        number, wire = key >> 3, key & 7
        if wire == 0:
            value, at = varint(data, at)
        elif wire == 2:
            size, at = varint(data, at)
            value, at = data[at : at + size], at + size
        elif wire == 1:
            value, at = data[at : at + 8], at + 8
        elif wire == 5:
            value, at = data[at : at + 4], at + 4
        else:
            raise ValueError(f"unsupported wire type {wire}")
        yield number, wire, value


def packed(data):
    at = 0
    while at < len(data):
        value, at = varint(data, at)
        yield value


def read_index(data):
    """`(definitions, referenced)`: `{symbol: (path, line)}` and the set of symbols with a reference."""
    definitions, referenced = {}, set()
    for number, _, document in fields(data):
        if number != 2:
            continue
        path, occurrences = "", []
        for number, _, value in fields(document):
            if number == 1:
                path = value.decode()
            elif number == 2:
                occurrences.append(value)
        for occurrence in occurrences:
            line, symbol, roles = 0, "", 0
            for number, _, value in fields(occurrence):
                if number == 1:
                    line = next(packed(value))
                elif number == 2:
                    symbol = value.decode()
                elif number == 3:
                    roles = value
            if not symbol or symbol.startswith("local "):
                continue
            if roles & DEFINITION:
                definitions.setdefault(symbol, (path, line))
            else:
                referenced.add(symbol)
    return definitions, referenced


def symbol_name(symbol):
    """`crate/path/Item` for a rust-analyzer symbol, or None for one that is not a plain item."""
    parts = symbol.split(" ", 4)
    if len(parts) != 5 or parts[0] != "rust-analyzer":
        return None
    path = parts[4]
    # Items end in `.` (value), `#` (type) or `().` (function); members, impl blocks and generics do not.
    if "#" in path.rstrip("#") or "impl" in path or "[" in path or "<" in path:
        return None
    return parts[2] + "/" + path.rstrip(".#()").replace("`", "")


def mentions(name, files):
    """How many times the identifier `name` is written in `files` (path to text)."""
    word = re.compile(r"(?<![\w])" + re.escape(name) + r"(?![\w])")
    return sum(len(word.findall(text)) for text in files.values())


def unused(definitions, referenced, source, files=None):
    """The `pub` items of library crates' non-test code that nothing refers to, sorted.

    The index misses some uses (a glob re-export, a macro body), so when `files` (every workspace
    Rust file's text) is given, an item whose name is written anywhere besides its definition is kept.
    """
    found = []
    for symbol, (path, line) in definitions.items():
        name = symbol_name(symbol)
        if name is None or symbol in referenced or name in ALLOWED:
            continue
        crate, _, rest = path.partition("/src/")
        if (
            not path.startswith("crates/")
            or not rest
            or "/tests/" in path
            or "benches" in path
            or not has_lib(crate)
        ):
            continue
        if Path(path).name in {"tests.rs", "main.rs", "build.rs"} or Path(
            path
        ).stem.endswith("_tests"):
            continue
        text = source(path).splitlines()
        if (
            line >= len(text)
            or not (match := ITEM.match(text[line]))
            or match.group(1) == "mod"
        ):
            continue
        # Test code, and what other programs call by name (`#[unsafe(no_mangle)]`, `export_name`).
        above = "\n".join(text[max(0, line - 4) : line])
        if "cfg(test)" in above or "no_mangle" in above or "export_name" in above:
            continue
        if files is not None and mentions(name.rsplit("/", 1)[1], files) > 1:
            continue
        found.append(f"{path}:{line + 1}: {name}")
    return sorted(found)


def has_lib(crate):
    """Whether the workspace folder `crate` (`crates/jaic`) is a library crate."""
    return (ROOT / crate / "src/lib.rs").is_file()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--index",
        type=Path,
        help="a SCIP index to read instead of running rust-analyzer",
    )
    parser.add_argument("--rust-analyzer", default="rust-analyzer")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as scratch:
        index = args.index
        if index is None:
            if not shutil.which(args.rust_analyzer):
                sys.exit(
                    "rust-analyzer not found: run `rustup component add rust-analyzer`"
                )
            index = Path(scratch) / "index.scip"
            subprocess.run(
                [args.rust_analyzer, "scip", ".", "--output", str(index)],
                cwd=ROOT,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        definitions, referenced = read_index(index.read_bytes())
    cache = {}

    def source(path):
        if path not in cache:
            cache[path] = (ROOT / path).read_text(errors="replace")
        return cache[path]

    files = {
        str(p): p.read_text(errors="replace") for p in (ROOT / "crates").rglob("*.rs")
    }
    found = unused(definitions, referenced, source, files)
    if found:
        print(
            "pub items nothing in the workspace refers to (delete them, or narrow them to `pub(crate)`):",
            file=sys.stderr,
        )
        print("\n".join(found), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
