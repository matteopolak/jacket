#!/usr/bin/env python3
"""Fail when a public standard-library declaration has no doc comment (docs/tools/stdlib-doc-gaps.md).

Usage: tools/ci/check_stdlib_docs.py [--update] [--report [--examples] [--top N] [--module NAME]]

`jaic -doc_gaps DIR` lists the public procedures, structs, enums, aliases and the module itself of one
library with whether each has docs and an example (public: not under `#scope_file` or `#scope_module`;
files a module `#load`s count, `internal/` modules it only imports do not; an alias of a documented
declaration is documented). This script runs it over every module of `stdlib/` and compares the
undocumented ones with `tools/ci/undocumented_stdlib.txt` (`Module.symbol`, or `Module` for the module's
own docs). The list only shrinks:

- a new undocumented public declaration fails the check (write the doc comment);
- a listed one that is now documented, or no longer exists, fails until it is removed (`--update`).

`--report` prints the gaps instead, worst module first (`--examples`: count the missing examples too).
`--update` rewrites the list from the tree; review its diff, a new line is a declaration added without docs.

The binary is `$JAIC`, else the built `jaic`, built with cargo when there is none.
"""

import argparse
import os
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from toolkit.repo import ROOT, cargo_binary, stdlib_env

STDLIB = ROOT / "stdlib"
ALLOWLIST = ROOT / "tools/ci/undocumented_stdlib.txt"
# Folders of the standard library that are not modules anyone imports.
SKIP_PARTS = {"internal", "tests", "target"}


def jaic_binary() -> Path:
    given = os.environ.get("JAIC")
    if given:
        return Path(given)
    binary = cargo_binary("jaic")
    if not binary.is_file():
        subprocess.run(["cargo", "build", "--locked", "-q", "-p", "jaic-cli"], cwd=ROOT, check=True)
    return binary


def modules() -> list[Path]:
    """The public modules of the standard library: folders with a `module.jai`, as paths relative to `stdlib/`."""
    found = []
    for entry in sorted(STDLIB.rglob("module.jai")):
        relative = entry.parent.relative_to(STDLIB)
        if any(p.startswith(".") or p in SKIP_PARTS for p in relative.parts) or not relative.parts:
            continue
        found.append(relative)
    return found


def parse(output: str, root: Path) -> list[dict]:
    """The records of `jaic -doc_gaps` as dicts, with files relative to `root`."""
    rows = []
    for line in output.splitlines():
        file, number, kind, symbol, doc, example = line.split("\t")
        try:
            file = str(Path(file).relative_to(root))
        except ValueError:
            pass
        rows.append(
            {"file": file, "line": int(number), "kind": kind, "symbol": symbol,
             "documented": doc == "1", "example": example == "1"}
        )
    return rows


def gaps_of(binary: Path, module: Path) -> list[dict]:
    done = subprocess.run([str(binary), "-doc_gaps", str(STDLIB / module)], cwd=ROOT, env=stdlib_env(),
                          capture_output=True, text=True)
    if done.returncode:
        raise SystemExit(f"jaic -doc_gaps {module} failed: {done.stderr.strip()}")
    return parse(done.stdout, ROOT)


def key(module: Path, row: dict) -> str:
    name = module.as_posix()
    return f"{name}.{row['symbol']}" if row["symbol"] else name


def collect() -> dict[Path, list[dict]]:
    binary = jaic_binary()
    return {module: gaps_of(binary, module) for module in modules()}


def undocumented(everything: dict[Path, list[dict]]) -> set[str]:
    return {key(m, r) for m, rows in everything.items() for r in rows if not r["documented"]}


def report(everything: dict[Path, list[dict]], examples: bool, top: int | None, only: str | None) -> str:
    """Modules with the most gaps first; each gap on a line below its module."""
    scored = []
    for module, rows in everything.items():
        if only and module.as_posix() != only:
            continue
        lacking = [r for r in rows if not r["documented"]]
        no_example = [r for r in rows if r["documented"] and not r["example"]] if examples else []
        if lacking or no_example:
            scored.append((len(lacking) + len(no_example), module, lacking, no_example))
    scored.sort(key=lambda s: (-s[0], s[1].as_posix()))
    lines = []
    for count, module, lacking, no_example in scored[:top]:
        public = len(everything[module])
        lines.append(f"{count:5}  {module.as_posix()}  ({len(lacking)} undocumented"
                     + (f", {len(no_example)} without an example" if examples else "") + f" of {public})")
        if only:
            for r in lacking:
                lines.append(f"         {r['file']}:{r['line']}  {r['kind']} {r['symbol'] or '(module)'}  no docs")
            for r in no_example:
                lines.append(f"         {r['file']}:{r['line']}  {r['kind']} {r['symbol'] or '(module)'}  no example")
    total = sum(s[0] for s in scored)
    lines.append(f"{total} gaps in {len(scored)} modules")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--update", action="store_true", help="rewrite tools/ci/undocumented_stdlib.txt from the tree")
    parser.add_argument("--report", action="store_true", help="print the gaps, worst module first")
    parser.add_argument("--examples", action="store_true", help="with --report: count documented items without an example too")
    parser.add_argument("--top", type=int, help="with --report: only the N worst modules")
    parser.add_argument("--module", help="with --report: list one module's gaps by line (e.g. Hash_Table)")
    args = parser.parse_args()
    everything = collect()
    if args.report:
        print(report(everything, args.examples, args.top, args.module))
        return 0
    missing = undocumented(everything)
    if args.update:
        ALLOWLIST.write_text("".join(f"{k}\n" for k in sorted(missing)))
        print(f"{len(missing)} undocumented; wrote {ALLOWLIST.relative_to(ROOT)}")
        return 0
    allowed = {line for line in ALLOWLIST.read_text().splitlines() if line}
    known = {key(m, r) for m, rows in everything.items() for r in rows}
    errors = []
    for k in sorted(missing - allowed):
        errors.append(f"undocumented public declaration: {k}\n  add a doc comment directly above it (docs/compiler/doc-comments.md)")
    for k in sorted(allowed - missing):
        why = "is now documented" if k in known else "no longer exists"
        errors.append(f"{k} {why}; remove it from tools/ci/undocumented_stdlib.txt (python3 tools/ci/check_stdlib_docs.py --update)")
    for e in errors:
        print(e, file=sys.stderr)
    print(f"{len(missing)} undocumented, {len(allowed)} allowlisted", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
