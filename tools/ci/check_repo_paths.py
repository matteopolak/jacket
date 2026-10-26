#!/usr/bin/env python3
"""Fail when a doc or a tool/config header names a repo path that does not exist.

Scans docs/**/*.md (whole files, plus relative link targets), the workflows and actions under
.github/ (whole files) and the leading comment block of
tools/**/*.{py,sh,mjs}, Justfile, .githooks/*, fuzz/jai.dict and editors/*/*.yaml. A mention is a
token that starts with a top-level repo folder (`tools/x.py`, `docs/tools/y.md`). Tokens with a glob,
placeholder or variable are skipped, as are paths under GENERATED (build products, fetched corpora)
and the ALLOW set (mentions of files that only exist in a user's checkout).
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import re
import subprocess
import sys
from pathlib import Path
from toolkit.repo import ROOT

TOP = (
    "docs",
    "tools",
    "crates",
    "stdlib",
    "editors",
    "fuzz",
    "tests",
    "benchmarks",
    "examples",
    "packaging",
    "nix",
    "corpus",
    "third_party",
    "jaifmt",
    ".github",
    ".githooks",
)
DOC_TOP = {
    "docs",
    "tools",
    "crates",
    "editors",
    "fuzz",
    ".github",
    ".githooks",
    "benchmarks",
    "packaging",
    "nix",
}
GENERATED = (
    "target/",
    "corpus/upstream",
    "corpus/lsp-applications",
    "stdlib/internal/developer-tools/libgenerator_platform",
    "fuzz/corpus/",
    "fuzz/artifacts/",
    "artifacts/",
    "benchmarks/regex/target",
    "node_modules/",
    "tools/corpus/regex_oracle/target",
    "editors/vscode/jacket-vscode.vsix",
)
# Mentions of files that exist only in a user's project, or relative to some other folder.
ALLOW = {"tools/build.jai", "tests/driven"}
MENTION = re.compile(
    r"(?<![\w./$-])((?:%s)/[^\s`'\"()|,;:]*)" % "|".join(re.escape(t) for t in TOP)
)
LINK = re.compile(r"\]\(([^)\s#]+)(?:#[^)]*)?\)")
SKIP_CHARS = set("*<>{}$[]")


def header(text):
    """Leading comment block (shebang, `#` lines, a docstring) of a tool or config file."""
    lines, out = text.splitlines(), []
    i = 0
    while i < len(lines) and (lines[i].startswith("#") or not lines[i].strip()):
        out.append(lines[i])
        i += 1
    if i < len(lines) and lines[i].lstrip().startswith(('"""', "'''")):
        quote = lines[i].lstrip()[:3]
        out.append(lines[i])
        closed = quote in lines[i].lstrip()[3:]
        i += 1
        while not closed and i < len(lines):
            out.append(lines[i])
            closed = quote in lines[i]
            i += 1
    return "\n".join(out)


def tracked():
    out = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT)
    return [p.decode() for p in out.split(b"\0") if p]


def skipped(rel):
    return (
        any(c in SKIP_CHARS for c in rel) or rel.startswith(GENERATED) or rel in ALLOW
    )


def check_text(name, text, root=ROOT, doc_dir=None):
    """Errors for the paths `text` mentions; `doc_dir` also checks markdown link targets.

    In a doc, `tests/` and `stdlib/` mentions are module-relative prose (`tests/codec.jai` under a
    stdlib module), so only the folders below are checked there; link targets are always checked.
    """
    errors = []
    prose = LINK.sub("]()", text) if doc_dir is not None else text
    for match in MENTION.finditer(prose):
        token = match.group(1).split("#")[0].rstrip(".-").rstrip("/")
        if doc_dir is not None and token.split("/")[0] not in DOC_TOP:
            continue
        if (
            not skipped(token)
            and not (root / token).exists()
            and not (doc_dir and (doc_dir / token).exists())
        ):
            errors.append(f"{name}: missing path {token}")
    if doc_dir is not None:
        for target in LINK.findall(text):
            if (
                "://" in target
                or target.startswith(("mailto:", "/"))
                or skipped(target)
                or not re.search(r"[./]", target)
            ):
                continue
            resolved = (doc_dir / target).resolve()
            if not resolved.exists():
                errors.append(f"{name}: dead link {target}")
    return errors


def headed(rel):
    parts = Path(rel).parts
    if parts[0] == "tools":
        return parts[-1].endswith((".py", ".sh", ".mjs"))
    if parts[0] == "editors" and len(parts) == 3:
        return parts[-1].endswith((".yaml", ".yml"))
    return rel in ("Justfile", "fuzz/jai.dict") or parts[0] == ".githooks"


def run():
    errors = []
    for rel in tracked():
        if rel.startswith("tools/ci/") and "check_repo_paths" in rel:
            continue
        path = ROOT / rel
        if not path.is_file():
            continue
        if rel.startswith("docs/") and rel.endswith(".md"):
            errors += check_text(
                rel, path.read_text(errors="replace"), doc_dir=path.parent
            )
        elif rel.startswith(".github/") and rel.endswith((".yml", ".yaml")):
            errors += check_text(rel, path.read_text(errors="replace"))
        elif headed(rel):
            errors += check_text(rel, header(path.read_text(errors="replace")))
    return errors


def main():
    errors = run()
    if errors:
        sys.exit("\n".join(errors))
    print("Repo path mention check passed")


if __name__ == "__main__":
    main()
