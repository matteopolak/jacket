#!/usr/bin/env python3
"""Enforce a blank line between Rust items, which rustfmt does not do.

    python3 tools/rust_item_spacing.py [--check] [paths...]

Items are the declarations at the top of a file and inside `impl`, `trait`, `mod` and `extern`
blocks. A multi-line item gets a blank line before and after it; runs of one-line items (`use`,
`mod a;`, constants, type aliases) may stay together. Comments and attributes in front of an item
belong to it, so the blank line goes above them;
a blank line between a comment and the next item (a section banner) detaches the comment. Imports (`use`, `mod a;`) form one block, set
apart from the items after it. Function bodies, struct fields and enum variants
are left alone.

Without paths, every `.rs` file under `crates/` is processed. `--check` lists the files that would
change and exits 1.
"""
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTAINER = re.compile(
    r"^(pub(\([^)]*\))?\s+)?(default\s+)?(unsafe\s+)?(impl|trait|mod|extern)\b"
)
ATTRIBUTE = re.compile(r"#!?\[[^\]]*\]")
GROUPED = re.compile(r"^(pub(\([^)]*\))?\s+)?(use\b|extern\s+crate\b|mod\s+\w+\s*;)")


@dataclass
class Line:
    depth_start: int = 0
    depth_end: int = 0
    code: str = ""  # code characters only (no comments, string contents kept)
    # Whether the block holding this line's start is an item container.
    in_container: bool = True
    blank: bool = True


@dataclass
class Lexer:
    text: str
    i: int = 0
    lines: list = field(default_factory=list)

    def run(self):
        text, n = self.text, len(self.text)
        depth = 0
        # Kind of each open block (True = item container); index 0 is the file.
        kinds = [True]
        header = [""]  # code since the last `;`, `{` or `}` at each depth
        line = Line(depth_start=0, in_container=True)
        block_comment = 0

        def newline():
            nonlocal line
            line.depth_end = depth
            self.lines.append(line)
            line = Line(depth_start=depth, in_container=kinds[depth])

        i = 0
        while i < n:
            c = text[i]
            if c == "\n":
                newline()
                i += 1
                continue
            if block_comment:
                if text.startswith("*/", i):
                    block_comment -= 1
                    i += 2
                elif text.startswith("/*", i):
                    block_comment += 1
                    i += 2
                else:
                    i += 1
                if not c.isspace():
                    line.blank = False
                continue
            if text.startswith("//", i):
                line.blank = False
                while i < n and text[i] != "\n":
                    i += 1
                continue
            if text.startswith("/*", i):
                line.blank = False
                block_comment = 1
                i += 2
                continue
            if c.isspace():
                # One space keeps keywords apart (`use crate`, `impl Foo`).
                if line.code and not line.code.endswith(" "):
                    line.code += " "
                if header[depth] and not header[depth].endswith(" "):
                    header[depth] += " "
                i += 1
                continue
            line.blank = False
            # Raw strings: r"..", r#".."#, br#".."#, cr"..".
            m = re.compile(r'(b|c)?r(#*)"').match(text, i)
            if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
                close = '"' + m.group(2)
                end = text.find(close, m.end())
                end = n if end < 0 else end + len(close)
                self.code_chars(line, header, depth, text[i:end])
                for _ in range(text.count("\n", i, end)):
                    i = text.index("\n", i)
                    newline()
                    i += 1
                i = end
                continue
            if c == '"':
                j = i + 1
                while j < n and text[j] != '"':
                    if text[j] == "\\":
                        j += 1
                    j += 1
                end = min(j + 1, n)
                self.code_chars(line, header, depth, '""')
                for _ in range(text.count("\n", i, end)):
                    i = text.index("\n", i)
                    newline()
                    i += 1
                i = end
                continue
            if c == "'":
                # A char literal ('a', '\n', '\u{..}') or a lifetime ('a).
                if i + 1 < n and text[i + 1] == "\\":
                    j = i + 2
                    while j < n and text[j] != "'":
                        j += 1
                    self.code_chars(line, header, depth, "' '")
                    i = j + 1
                    continue
                if i + 2 < n and text[i + 2] == "'":
                    self.code_chars(line, header, depth, "' '")
                    i += 3
                    continue
                self.code_chars(line, header, depth, c)
                i += 1
                continue
            if c == "{":
                head = ATTRIBUTE.sub("", header[depth]).strip()
                kinds.append(kinds[depth] and bool(CONTAINER.match(head)))
                header[depth] = ""
                depth += 1
                header.append("")
                line.code += c
                i += 1
                continue
            if c == "}":
                if depth > 0:
                    depth -= 1
                    kinds.pop()
                    header.pop()
                header[depth] = ""
                line.code += c
                i += 1
                continue
            if c == ";":
                header[depth] = ""
                line.code += c
                i += 1
                continue
            self.code_chars(line, header, depth, c)
            i += 1
        line.depth_end = depth
        self.lines.append(line)
        return self.lines

    @staticmethod
    def code_chars(line, header, depth, s):
        line.code += s
        header[depth] += s


def space_items(text: str) -> str:
    raw = text.split("\n")
    lines = Lexer(text).run()
    assert len(lines) == len(raw), (len(lines), len(raw))

    # Items open at each container depth (depth -> first line), and finished items as
    # (first line, last line, depth). An item ends with `;` or `}` back at its own depth.
    open_items = {}
    spans = []
    for k, ln in enumerate(lines):
        if ln.blank and not raw[k].strip():
            # A blank line detaches comments above it (a section banner) from the next item.
            start = open_items.get(ln.depth_start)
            if start is not None and not any(lines[j].code.strip() for j in range(start, k)):
                del open_items[ln.depth_start]
            continue
        if ln.blank or raw[k].lstrip().startswith(("//!", "#![")):
            continue  # inner docs and attributes belong to the enclosing module
        depth = ln.depth_start
        closes_block = ln.code.lstrip()[:1] == "}"
        if closes_block:
            open_items.pop(depth, None)
        elif ln.in_container and depth not in open_items:
            open_items[depth] = k
        for d in [d for d in open_items if d > ln.depth_end]:
            del open_items[d]
        code = ln.code.rstrip()
        if code and code[-1] in ";}" and ln.depth_end in open_items:
            spans.append((open_items.pop(ln.depth_end), k, ln.depth_end))
    by_start = {(a, d): (a, b, d) for a, b, d in spans}

    def grouped(span):
        # Imports and `mod a;`, after any attributes and comments in front of them.
        code = " ".join(lines[k].code.strip() for k in range(span[0], span[1] + 1))
        return bool(GROUPED.match(ATTRIBUTE.sub("", code).strip()))

    insert_after = set()
    for a in spans:
        b = by_start.get((a[1] + 1, a[2]))
        if not b:
            continue  # already separated, or the container ends
        if grouped(a) or grouped(b):
            if grouped(a) and grouped(b):
                continue  # one import block
        elif a[0] == a[1] and b[0] == b[1]:
            continue  # a run of one-line items
        insert_after.add(a[1])

    out = []
    for k, s in enumerate(raw):
        out.append(s)
        if k in insert_after:
            out.append("")
    return "\n".join(out)


def rust_files(paths):
    if not paths:
        paths = [ROOT / "crates"]
    for p in map(Path, paths):
        if p.is_dir():
            yield from sorted(f for f in p.rglob("*.rs") if "target" not in f.parts)
        else:
            yield p


def main(argv):
    check = "--check" in argv
    paths = [a for a in argv if a != "--check"]
    changed = []
    for f in rust_files(paths):
        text = f.read_text()
        spaced = space_items(text)
        if spaced != text:
            changed.append(f)
            if not check:
                f.write_text(spaced)
    for f in changed:
        rel = f.resolve().relative_to(ROOT) if f.resolve().is_relative_to(ROOT) else f
        print(f"{'needs item spacing' if check else 'spaced'}: {rel}")
    return 1 if (check and changed) else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
