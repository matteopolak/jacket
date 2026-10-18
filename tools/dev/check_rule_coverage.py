#!/usr/bin/env python3
"""Rule-to-test traceability for the language docs.

docs/language/*.md and docs/metaprogramming/*.md mark each normative claim with a stable rule ID,
written `{#ops.3}` right after the claim. Tests name the rules they pin in a comment line
`// rules: ops.3 ops.4` (commas allowed) anywhere in the file; corpus cases list them in the
`rules` field of their tests/corpus/manifest.json entry. This script lists:

- rules no test cites (reported, not an error yet),
- tests citing a rule no doc defines (an error: a typo or a removed rule),
- rule IDs defined twice (an error).

Usage: check_rule_coverage.py [--list-uncovered] [--json]
Exit status 1 on unknown or duplicate IDs, else 0.
"""
import argparse, json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC_DIRS = ("docs/language", "docs/metaprogramming")
# Where tests live: the sweep's sets (corpus, stdlib, the stdlib's own tests, examples) and the
# Rust unit and integration tests.
TEST_GLOBS = ("tests/**/*.jai", "stdlib/**/tests/**/*.jai", "crates/**/tests/**/*.rs",
              "crates/**/src/**/tests.rs", "crates/**/src/**/tests/*.rs")
RULE_ID = r"[a-z][a-z0-9-]*(?:\.[a-z0-9-]+)+"
MARKER = re.compile(r"\{#(" + RULE_ID + r")\}")
CITATION = re.compile(r"^\s*//\s*rules:\s*(.+)$", re.M)


def doc_rules(root=ROOT):
    """{rule id: [doc:line, ...]} for every marker outside fenced code blocks."""
    rules = {}
    for d in DOC_DIRS:
        for doc in sorted((root / d).glob("*.md")):
            fenced = False
            for number, line in enumerate(doc.read_text(encoding="utf-8").split("\n"), 1):
                if line.lstrip().startswith("```"):
                    fenced = not fenced
                    continue
                if fenced:
                    continue
                for m in MARKER.finditer(line):
                    rules.setdefault(m.group(1), []).append(f"{doc.relative_to(root)}:{number}")
    return rules


def test_citations(root=ROOT):
    """{rule id: [test path, ...]} from `// rules:` comment lines."""
    cited = {}
    seen = set()
    for pattern in TEST_GLOBS:
        for path in sorted(root.glob(pattern)):
            if path in seen or "node_modules" in path.parts or not path.is_file():
                continue
            seen.add(path)
            text = path.read_text(encoding="utf-8", errors="replace")
            for m in CITATION.finditer(text):
                for rule in re.split(r"[\s,]+", m.group(1).strip()):
                    if rule:
                        cited.setdefault(rule, []).append(str(path.relative_to(root)))
    # Corpus cases are fingerprinted by hash, so they cite rules in a manifest field instead.
    manifest = root / "tests/corpus/manifest.json"
    if manifest.exists():
        for case in json.loads(manifest.read_text())["cases"]:
            for rule in case.get("rules", []):
                cited.setdefault(rule, []).append(f"tests/corpus/manifest.json#{case['id']}")
    return cited


def report(root=ROOT):
    rules = doc_rules(root)
    cited = test_citations(root)
    return {
        "rules": len(rules),
        "covered": sorted(r for r in rules if r in cited),
        "uncovered": sorted(r for r in rules if r not in cited),
        "unknown": {r: sorted(set(paths)) for r, paths in sorted(cited.items()) if r not in rules},
        "duplicates": {r: places for r, places in sorted(rules.items()) if len(places) > 1},
        "where": {r: places[0] for r, places in rules.items()},
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list-uncovered", action="store_true", help="print every uncovered rule")
    ap.add_argument("--json", action="store_true", help="machine-readable report")
    a = ap.parse_args()
    r = report()
    if a.json:
        print(json.dumps({k: v for k, v in r.items() if k != "where"}, indent=2))
    else:
        print(f"{r['rules']} rules, {len(r['covered'])} covered, {len(r['uncovered'])} uncovered")
        if a.list_uncovered:
            for rule in r["uncovered"]:
                print(f"  uncovered {rule}  ({r['where'][rule]})")
        for rule, paths in r["unknown"].items():
            print(f"error: unknown rule {rule} cited by {', '.join(paths)}")
        for rule, places in r["duplicates"].items():
            print(f"error: rule {rule} defined more than once: {', '.join(places)}")
    sys.exit(1 if r["unknown"] or r["duplicates"] else 0)


if __name__ == "__main__":
    main()
