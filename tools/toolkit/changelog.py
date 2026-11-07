"""Fence-aware release headings shared by version bumps and note extraction."""
import re


def headings(text):
    """Yield (start, end, title) outside fences; `## Unreleased` yields that title."""
    fence = None
    offset = 0
    for raw in text.splitlines(keepends=True):
        line = raw.rstrip("\r\n")
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if marker and marker[1][0] == fence[0] and len(marker[1]) >= len(fence) and not marker[2].strip():
                fence = None
        # Backticks in a backtick info string make this text, not a fence opener.
        elif marker and (marker[1][0] == "~" or "`" not in marker[2]):
            fence = marker[1]
        elif match := re.fullmatch(r"##[ \t]+(.+?)[ \t]*", line):
            yield offset, offset + len(line), match[1]
        offset += len(raw)


def release_name(title):
    """Recognize exact Unreleased forms or bracketed versions, e.g. `[1.2.3] - pending`."""
    if title in ("Unreleased", "[Unreleased]"):
        return "Unreleased"
    match = re.fullmatch(r"\[(\d+\.\d+\.\d+)\](?:[ \t].*)?", title)
    return match[1] if match else None
