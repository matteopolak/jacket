#!/usr/bin/env python3
"""Fail when a doc or a tool/config header names a repo path that does not exist.

Scans docs/**/*.md (whole files, plus relative link targets), the workflows and actions under
.github/ (whole files) and the leading comment block of
tools/**/*.{py,sh,mjs}, Justfile, .githooks/*, fuzz/jai.dict and editors/*/*.yaml. A mention is a
token that starts with a top-level repo folder (`tools/x.py`, `docs/tools/y.md`). Tokens with a glob,
placeholder or variable are skipped, as are paths under GENERATED (build products, fetched corpora)
and the ALLOW set (mentions of files that only exist in a user's checkout).

Two more checks keep the docs honest. A backticked `crate::path::Item` in docs/**/*.md (`jaic::progress::mode`,
`jacket_config::env`) must resolve: the crate is a folder under crates/, each module a file or folder below its
src/, and the first item after the modules a definition (or a `use`) in the last module's file. Anything after
that item (a method, a variant) only has to appear there. And every docs/**/*.md must be reachable from
docs/README.md through the links of the index pages (a folder's README.md lists its folder).
"""

import functools
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


CRATE_SPAN = re.compile(r"`([a-z][a-z0-9_]*)((?:::[A-Za-z_]\w*)+)(::\{[^}`]*\})?")
ITEM = r"\b(?:fn|struct|enum|union|const|static|trait|type|mod|macro_rules!)\s+%s\b"


# `jaic` re-exports the compiler's layers, so a path under it may resolve in any of them.
FACADE_LAYERS = {"jaic": ["jaic", "jaic_syntax", "jaic_ir", "jaic_target", "jaic_ide"]}


def crate_roots(root=ROOT):
    """The workspace crates by the name Rust code uses for them (`jacket_config`) and their src/ folder."""
    found = {}
    for manifest in sorted((root / "crates").glob("*/Cargo.toml")):
        found[manifest.parent.name.replace("-", "_")] = manifest.parent / "src"
    return found


def mentions(text, name):
    """Whether `name` is defined or imported in `text`."""
    word = re.escape(name)
    return bool(
        re.search(ITEM % word, text)
        or re.search(r"\buse\b[^;]*\b%s\b" % word, text, re.S)
    )


@functools.lru_cache(maxsize=None)
def tree_text(folder):
    """All the Rust under `folder`, as one string."""
    return "\n".join(p.read_text(errors="replace") for p in sorted(folder.rglob("*.rs")))


def resolve_symbol(src, segments):
    """None when `segments` (below the crate) resolve in the crate whose src/ is `src`, else what is missing.

    Modules are followed file by file; the first name that is no module must be defined or imported in the
    last module (its file, then the folder of its submodules, which it may re-export); the names after it
    (a method, a field, a variant) only have to appear somewhere in the crate.
    """
    here = src
    text = "\n".join(
        (src / name).read_text(errors="replace")
        for name in ("lib.rs", "main.rs")
        if (src / name).is_file()
    )
    for index, name in enumerate(segments):
        module = here / f"{name}.rs"
        if not module.is_file():
            module = here / name / "mod.rs"
        if module.is_file() and re.search(r"\bmod\s+%s\b" % re.escape(name), text):
            here, text = here / name, module.read_text(errors="replace")
            continue
        if not (mentions(text, name) or mentions(tree_text(here), name)):
            return name
        crate_text = tree_text(src)
        rest = segments[index + 1 :]
        return next((w for w in rest if not re.search(r"\b%s\b" % re.escape(w), crate_text)), None)
    return None


def check_symbols(name, text, crates):
    """Errors for the backticked `crate::path` references in the markdown `text`."""
    errors, fenced = [], False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
        if fenced:
            continue
        for match in CRATE_SPAN.finditer(line):
            crate, rest, group = match.groups()
            if crate not in crates:
                continue
            segments = rest.split("::")[1:]
            wanted = [segments]
            if group:
                members = [m.strip().split("::")[0] for m in group[3:-1].split(",")]
                wanted = [segments + [m] for m in members if m]
            for path in wanted:
                misses = [
                    resolve_symbol(crates[layer], path)
                    for layer in FACADE_LAYERS.get(crate, [crate])
                    if layer in crates
                ]
                missing = None if None in misses else misses[0]
                if missing:
                    errors.append(
                        f"{name}: `{crate}::{'::'.join(path)}` does not resolve: no `{missing}`"
                    )
    return errors


def reachable_docs(root=ROOT):
    """The docs/**/*.md files, with the ones docs/README.md reaches through links (a folder link means its README.md)."""
    docs = root / "docs"
    every = {p.resolve() for p in docs.rglob("*.md")}
    seen, todo = set(), [(docs / "README.md").resolve()]
    while todo:
        page = todo.pop()
        if page in seen or page not in every:
            continue
        seen.add(page)
        for target in LINK.findall(page.read_text(errors="replace")):
            if "://" in target or target.startswith(("mailto:", "/")):
                continue
            path = (page.parent / target).resolve()
            todo.append(path / "README.md" if path.is_dir() else path)
    return every, seen


def check_index(root=ROOT):
    every, seen = reachable_docs(root)
    docs = (root / "docs").resolve()
    return [
        f"docs/{p.relative_to(docs).as_posix()}: not listed in docs/README.md (or an index page it links)"
        for p in sorted(every - seen)
    ]


def headed(rel):
    parts = Path(rel).parts
    if parts[0] == "tools":
        return parts[-1].endswith((".py", ".sh", ".mjs"))
    if parts[0] == "editors" and len(parts) == 3:
        return parts[-1].endswith((".yaml", ".yml"))
    return rel in ("Justfile", "fuzz/jai.dict") or parts[0] == ".githooks"


def run():
    errors = check_index()
    crates = crate_roots()
    for rel in tracked():
        if rel.startswith("tools/ci/") and "check_repo_paths" in rel:
            continue
        path = ROOT / rel
        if not path.is_file():
            continue
        if rel.startswith("docs/") and rel.endswith(".md"):
            body = path.read_text(errors="replace")
            errors += check_text(rel, body, doc_dir=path.parent)
            errors += check_symbols(rel, body, crates)
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
