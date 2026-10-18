"""Bounded CI error headlines; full subprocess output stays in local receipts."""

import re
import sys

ANSI = re.compile(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))")
HEADER = re.compile(
    r"(?:^|:\s)(?:fatal error|error(?:\[[^\]]+\])?|note)(?::|\s)"
    r"|(?:^|/)(?:ld(?:\.lld)?|lld): "
    r"|:\sundefined reference to "
    r"|^Undefined symbols for architecture "
    r"|^LLVM ERROR:",
    re.IGNORECASE,
)
CREDENTIAL = re.compile(
    r"(?i)\b(token|password|authorization|api[_-]?key)\s*[:=]\s*(?:Bearer\s+)?[^\s,;]+"
)


def clean(text):
    text = ANSI.sub("", text)
    text = "".join(character for character in text if character.isprintable())
    text = CREDENTIAL.sub(r"\1=<redacted>", text)
    return re.sub(r"(https?://)[^/\s@]+@", r"\1<redacted>@", text)


def failure_diagnostic(row):
    """Return at most 4 KiB, never source context, argv or environment contents."""
    lines = [f"{clean(str(row['name']))[:120]} FAIL exit={row.get('exit')}"]
    for output in [row.get("stderr", ""), row.get("stdout", "")]:
        for original in output.splitlines():
            line = ANSI.sub("", original)
            if line != line.lstrip():
                continue
            if not HEADER.search(line):
                continue
            lines.append(clean(line)[:280])
            if len(lines) == 13:
                break
        if len(lines) == 13:
            break
    if len(lines) == 1:
        lines.append(
            "No compiler/linker error header found; full output preserved in receipt."
        )
    return "\n".join(lines).encode("utf-8")[:4096].decode("utf-8", errors="ignore")


def report_failure(row, capture, *, stream=None):
    """Keep stdout's final receipt path intact for the existing package driver."""
    if stream is None:
        stream = sys.stderr
    print(failure_diagnostic(row), file=stream, flush=True)
    print(f"Full command capture: {capture}", file=stream, flush=True)
