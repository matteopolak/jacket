#!/usr/bin/env python3
"""Fail when CHANGELOG.md's unreleased notes are too long to read at a glance.

The notes under `## [Unreleased]` (or `## Unreleased`) become the GitHub release, so they are held to
fixed limits: a summary of at most SUMMARY_SENTENCES sentences and SUMMARY_WORDS words before the
first `###` heading, only the SECTIONS headings in that order, at most MAX_HIGHLIGHTS highlights, and
at most BULLET_WORDS words per bullet (TABLE_CELL_WORDS per table cell). Link targets and a trailing
attribution such as `(#84, @someone)` are not counted, nor are images (`![...](...)` or HTML tags such as an
`<img>` table). Released sections are left alone.
`tools/bump_version.py` runs the same check before it names the section.

    PYTHONPATH=tools python3 tools/ci/check_changelog.py
"""

import re
import sys
from toolkit.changelog import headings, release_name
from toolkit.repo import ROOT

SUMMARY_SENTENCES = 2
SUMMARY_WORDS = 30
BULLET_WORDS = 20
TABLE_CELL_WORDS = 15
MAX_HIGHLIGHTS = 6
SECTIONS = ("Highlights", "Breaking changes", "Fixes")

IMAGE = re.compile(r"!\[[^\]]*\]\([^)]*\)|<[^>]*>")
LINK_TARGET = re.compile(r"\]\([^)]*\)")
ATTRIBUTION = re.compile(r"\s*\((?:#\d+|@[\w-]+)(?:,\s*(?:#\d+|@[\w-]+))*\)\s*$")
SENTENCE_END = re.compile(r"[.!?](?:\s|$)")


def unreleased(text):
    """The body of the Unreleased section, or None."""
    marks = [(start, end, release_name(title)) for start, end, title in headings(text)]
    for i, (_, end, name) in enumerate(marks):
        if name == "Unreleased":
            stop = marks[i + 1][0] if i + 1 < len(marks) else len(text)
            return text[end:stop]
    return None


def words(text):
    return len(ATTRIBUTION.sub("", LINK_TARGET.sub("]", IMAGE.sub(" ", text))).split())


def preview(text):
    return " ".join(text.split()[:6]) + " ..."


def problems(text):
    """What breaks the limits in the Unreleased notes of `text` (a CHANGELOG.md)."""
    body = unreleased(text)
    if body is None:
        return []
    found = []
    parts = re.split(r"^###[ \t]+(.+?)[ \t]*$", body, flags=re.M)
    summary = parts[0].strip()
    if not summary:
        found.append("the summary is missing: one or two sentences before the first ### heading")
    else:
        sentences = len(SENTENCE_END.findall(summary))
        if sentences > SUMMARY_SENTENCES:
            found.append(f"the summary has {sentences} sentences, the limit is {SUMMARY_SENTENCES}")
        if words(summary) > SUMMARY_WORDS:
            found.append(f"the summary has {words(summary)} words, the limit is {SUMMARY_WORDS}")
    titles = parts[1::2]
    order = [SECTIONS.index(t) for t in titles if t in SECTIONS]
    for title in titles:
        if title not in SECTIONS:
            found.append(f"unknown section `### {title}`: use {', '.join(SECTIONS[:-1])} or {SECTIONS[-1]}, in that order")
    if order != sorted(set(order)):
        found.append(f"sections are out of order or repeated: use {', '.join(SECTIONS)}")
    for title, content in zip(titles, parts[2::2]):
        bullets = [line for line in content.splitlines() if re.match(r"^[-*] ", line)]
        if title == "Highlights" and len(bullets) > MAX_HIGHLIGHTS:
            found.append(f"Highlights has {len(bullets)} bullets, the limit is {MAX_HIGHLIGHTS}")
        for bullet in bullets:
            if words(bullet[2:]) > BULLET_WORDS:
                found.append(f"{title}: a bullet has {words(bullet[2:])} words, the limit is {BULLET_WORDS}: {preview(bullet)}")
        for line in content.splitlines():
            if line.startswith("|"):
                for cell in line.strip().strip("|").split("|"):
                    if words(cell) > TABLE_CELL_WORDS:
                        found.append(f"{title}: a table cell has {words(cell)} words, the limit is {TABLE_CELL_WORDS}: {preview(cell)}")
            elif line.strip() and not re.match(r"^[-*] ", line):
                found.append(f"{title}: write bullets or a table, not paragraphs: {preview(line)}")
    return found


def main():
    found = problems((ROOT / "CHANGELOG.md").read_text())
    for problem in found:
        print(f"CHANGELOG.md: {problem}")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
