#!/usr/bin/env python3
"""Turns the vendored llhttp fixtures (MIT) into stdlib/Extensions/Http/tests/data/llhttp_cases.jai,
so the Http tests need no file system (they also run under wasm).

    python3 tools/gen_http_corpus.py

Each fixture is an HTTP message (the ```http block, with the escapes llhttp's own runner expands)
and the events llhttp reported for it (the ```log block). The log is reduced to what the codec can
be compared on: the request or status line, the headers, the body bytes, whether each message
completed, and whether the parser stopped with an error. Cases that exercise llhttp's C API
(pausing, skipping a body) have no counterpart and are left out.

A case whose result differs from llhttp's on purpose carries a `deviation`: the reason lives in
DEVIATIONS below, keyed by `file:line` of the fixture.
"""

import json
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "stdlib", "Extensions", "Http", "tests", "data")
SOURCE = os.path.join(ROOT, "llhttp")
OUTPUT = os.path.join(ROOT, "llhttp_cases.jai")
FILES = [
    "request/sample", "request/method", "request/uri", "request/connection", "request/content-length",
    "request/transfer-encoding", "request/invalid", "request/finish", "request/pipelining",
    "request/lenient-headers", "request/lenient-header-value-relaxed", "request/lenient-version",
    "response/sample", "response/connection", "response/content-length", "response/transfer-encoding",
    "response/invalid", "response/finish", "response/pipelining", "response/lenient-version",
]

# Where the strict codec and llhttp disagree, by fixture ("file.md:line"), and why. A case that is
# only a deviation because llhttp ran with a leniency flag is marked from its type instead.
DEVIATIONS = {}
DEVIATION_RULES = []  # (regex over the case name, reason), applied when DEVIATIONS has no entry
if os.path.exists(os.path.join(os.path.dirname(os.path.abspath(__file__)), "http_deviations.json")):
    with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "http_deviations.json")) as f:
        data = json.load(f)
        DEVIATIONS = data.get("cases", {})
        DEVIATION_RULES = [(re.compile(r["match"]), r["reason"]) for r in data.get("rules", [])]


def literal(data: bytes) -> str:
    out = ['"']

    for b in data:
        c = chr(b)

        if c == '"':
            out.append('\\"')
        elif c == "\\":
            out.append("\\\\")
        elif 0x20 <= b < 0x7F and c != "%":
            out.append(c)
        else:
            out.append("\\x%02x" % b)

    out.append('"')

    return "".join(out)


def expand_input(text: str) -> bytes:
    """What llhttp's md-test runner does to the ```http block."""
    if text.endswith("\n"):
        text = text[:-1]

    text = re.sub(r"\\(\r\n|\r|\n)", "", text)
    text = re.sub(r"\r\n|\r|\n", "\r\n", text)
    text = text.replace("\\r", "\r").replace("\\n", "\n").replace("\\t", "\t").replace("\\f", "\f")
    text = re.sub(r"\\x([0-9a-fA-F]+)", lambda m: chr(int(m.group(1), 16)), text)
    text = re.sub(r"\\([0-7]{1,3})", lambda m: chr(int(m.group(1), 8)), text)

    if "${" in text:
        raise ValueError("inline JavaScript")

    return text.encode("utf-8")


SPAN = re.compile(r'^off=\d+ len=\d+ span\[(\w+)\]=(.*)$')
FINISH = re.compile(r"^off=NULL finish=(\d+)$")


def unquote(raw: str) -> str:
    if raw in ("lf", "cr"):
        return {"lf": "\n", "cr": "\r"}[raw]

    assert raw.startswith('"') and raw.endswith('"'), raw

    return raw[1:-1].replace("\\t", "\t").replace("\\f", "\f")


def reduce_log(log: str):
    """-> (lines for the Jai side, finish code or -1). Lines: M/S start, H header, D headers
    complete, B body (hex), C message complete, E error."""
    lines = []
    finish = -1
    current = None  # the header being assembled: [name, value]
    pending = {}
    after_head = False  # header fields from here on are trailers

    def flush_header():
        nonlocal current, after_head

        if current is not None:
            lines.append("%s\x01%s\x01%s" % ("T" if after_head else "H", current[0], current[1]))
            current = None

    for raw in log.split("\n"):
        if not raw:
            continue

        m = FINISH.match(raw)

        if m:
            finish = int(m.group(1))

            continue

        m = SPAN.match(raw)

        if m:
            kind, value = m.group(1), unquote(m.group(2))

            if kind == "header_field":
                flush_header()
                current = [value, ""]
            elif kind == "header_value":
                current[1] += value
            elif kind == "body":
                lines.append("B\x01" + value.encode("utf-8").hex())
            elif kind in ("method", "url", "protocol", "version", "status"):
                pending[kind] = pending.get(kind, "") + value

            continue

        body = raw.split(" ", 1)[1] if raw.startswith("off=") else raw

        if body == "message begin":
            flush_header()
            pending = {}
            after_head = False
        elif body.startswith("headers complete"):
            flush_header()

            if "method" in pending:
                lines.append("M\x01%s\x01%s\x01%s" % (pending["method"], pending.get("url", ""), pending.get("version", "")))
            else:
                lines.append("S\x01%s\x01%s\x01%s" % (re.search(r"status=(\d+)", body).group(1), pending.get("status", ""), pending.get("version", "")))

            lines.append("D")
            after_head = True
        elif body == "message complete":
            flush_header()
            lines.append("C")
        elif body.startswith("error "):
            flush_header()
            lines.append("E\x01" + re.search(r'reason="(.*)"', body).group(1))

    flush_header()

    return lines, finish


def reorder(lines):
    """H lines are collected before their start line is known; move each message's headers after
    its M/S line."""
    out = []
    headers = []

    for line in lines:
        if line.startswith("H\x01"):
            headers.append(line)
        elif line[0] in "MS":
            out.append(line)
            out.extend(headers)
            headers = []
        else:
            if line.startswith("E") and headers:
                headers = []

            out.append(line)

    return out


def parse_file(rel: str):
    path = os.path.join(SOURCE, rel + ".md")

    with open(path) as f:
        lines = f.read().split("\n")

    cases = []
    heading = ""
    i = 0

    while i < len(lines):
        line = lines[i]

        if line.startswith("#"):
            heading = line.lstrip("#").strip()
            i += 1

            continue

        if line.startswith("<!-- meta="):
            meta = json.loads(line[len("<!-- meta="):].rsplit("-->", 1)[0].strip())
            j = i + 1

            while not lines[j].startswith("```"):
                j += 1

            assert lines[j].startswith("```http"), (rel, i)
            start = j + 1
            end = start

            while not lines[end].startswith("```"):
                end += 1

            http = "\n".join(lines[start:end]) + "\n"
            end += 1

            while not lines[end].startswith("```log"):
                end += 1

            lstart = end + 1
            lend = lstart

            while not lines[lend].startswith("```"):
                lend += 1

            log = "\n".join(lines[lstart:lend]) + "\n"
            cases.append((heading, meta, http, log, start))
            i = lend + 1

            continue

        i += 1

    return cases


def main():
    out = [
        "// Generated by tools/gen_http_corpus.py from data/llhttp (llhttp, MIT). Do not edit.",
        "",
        "Llhttp_Case :: struct {",
        "    id: string; // file:line of the fixture",
        "    name: string;",
        "    response: bool;",
        "    lenient: string; // the llhttp leniency flag the fixture ran with, else empty",
        "    finish: int; // -1, or what llhttp_finish returned (0 and 1 are fine, 2 is an error)",
        "    input: string;",
        "    expected: string; // lines: M/S start line, H header, D head complete, B body hex, C complete, E error",
        "    deviation: string; // why the strict codec differs, else empty",
        "}",
        "",
        "LLHTTP_CASES :: Llhttp_Case.[",
    ]

    total = 0
    skipped = 0
    deviations = 0

    for rel in FILES:
        for heading, meta, http, log, line in parse_file(rel):
            if "pause" in meta or meta.get("skipBody"):
                skipped += 1

                continue

            try:
                data = expand_input(http)
            except ValueError:
                skipped += 1

                continue

            ty = meta["type"]
            response = ty.startswith("response")
            lenient = ""
            m = re.match(r"(?:request|response)-(lenient-.*|finish)$", ty)

            if m and m.group(1) != "finish":
                lenient = m.group(1)

            reduced, finish = reduce_log(log)
            expected = "\n".join(reorder(reduced))
            case_id = "%s.md:%d" % (rel, line)
            name = "%s: %s" % (rel.split("/")[0], heading)

            deviation = DEVIATIONS.get(case_id, "")

            if not deviation:
                for pattern, reason in DEVIATION_RULES:
                    if pattern.search(case_id + " " + name):
                        deviation = reason

                        break

            if deviation:
                deviations += 1

            out.append("    .{ %s, %s, %s, %s, %d, %s, %s, %s }," % (
                literal(case_id.encode()), literal(name.encode()), "true" if response else "false",
                literal(lenient.encode()), finish, literal(data), literal(expected.encode()),
                literal(deviation.encode())))
            total += 1

    out.append("];")

    with open(OUTPUT, "w") as f:
        f.write("\n".join(out) + "\n")

    print("wrote %d cases (%d skipped, %d with a deviation)" % (total, skipped, deviations))


main()
