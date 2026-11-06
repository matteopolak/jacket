#!/usr/bin/env python3
"""Exercise real guide resolution and URL/base policy with an installed owned Jacket.

Example (inside the shared heavy gate):
  PYTHONPATH=tools python3 tools/ci/check_doc_resolver.py --jacket /owned/bin/jacket --work /tmp/owned-resolver
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from toolkit.repo import ROOT


def fixture(root, home):
    """Create only authored inert input; Markdown fences are never executed."""
    (root / "library").mkdir(parents=True)
    (root / "library/module.jai").write_text("/// Fixture constant.\nfixture_value :: 7;\n\n// Fixture declaration.\nfixture_record :: struct {\n\n    first     :int;\n}\n")
    (root / "home.md").write_text(home)
    (root / "guide.md").write_text("# Fixture guide\n\n## Destination\n\n[Home](home.md)\n")
    (root / "jacket.toml").write_text('[doc]\npath = "library"\nname = "Resolver fixture"\nhome = "home.md"\n'
        '[[doc.page]]\nfile = "guide.md"\nname = "Fixture guide"\n')


def execute(command, cwd):
    return subprocess.run(command, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)


def run(jacket, work):
    """Require real successful root/nested rendering and explicit negative-policy failures."""
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    work.mkdir(parents=True)
    builder = ROOT / "website/docs"
    ledger = []
    home = '# Fixture home\n\n[Fixture guide](guide.md#destination)\n\n[Fixture API](library/module.jai)\n\n' \
           '[External](https://example.com/q?a=1&b=2 "A & B")\n\n[Unsafe](javascript:alert(1))\n\n' \
           '<script>window.fixtureInjected=1</script>\n\n<img src=x onerror=fixtureInjected()>\n\n' \
           'Text & "quotes" <not-a-template> Ω.\n'
    for label, base in (("root", "/"), ("nested", "/nested/docs/")):
        root = work / label
        fixture(root, home)
        export, output = root / "export", root / "site"
        result = execute([str(jacket), "doc", "library", "--export-only", "--export", str(export), "--base-url", base], root)
        (root / "export.log").write_text(result.stdout)
        if result.returncode:
            raise RuntimeError(f"{label} real CLI export failed: {result.stdout}")
        result = execute(["node", str(builder / "scripts/build.mjs"), "--export", str(export), "--out", str(output)], root)
        (root / "build.log").write_text(result.stdout)
        if result.returncode:
            raise RuntimeError(f"{label} actual site build failed: {result.stdout}")
        subprocess.run(["node", str(builder / "scripts/check-doc-resolver.mjs"), str(export), str(output), base], check=True)
        ledger.append({"id": label, "base": base, "status": "pass"})
    for label, markdown, reason in (
            ("unsafe-image", "# Unsafe\n\n![unsafe](//evil.invalid/x)\n", ("unsupported documentation URL", "unresolved image")),
            ("missing-image", "# Missing\n\n![missing](/missing-fixture.svg)\n", ("unknown", "missing", "unresolved"))):
        root = work / label
        fixture(root, markdown)
        export = root / "export"
        result = execute([str(jacket), "doc", "library", "--export-only", "--export", str(export)], root)
        stage = "producer"
        if result.returncode == 0:
            result = execute(["node", str(builder / "scripts/build.mjs"), "--export", str(export), "--out", str(root / "site")], root)
            stage = "consumer"
        (root / "rejection.log").write_text(result.stdout)
        if result.returncode == 0 or not any(word.lower() in result.stdout.lower() for word in reason):
            raise RuntimeError(f"{label} did not fail with its intended policy error: {result.stdout}")
        ledger.append({"id": label, "status": "pass", "rejectedBy": stage})
    (work / "ledger.json").write_text(json.dumps(ledger, indent=2) + "\n")
    print(f"Real resolver policies: {len(ledger)} cases passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jacket", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    os.environ.setdefault("JACKET_DOCS_BUILDER", str(ROOT / "website/docs"))
    run(args.jacket.resolve(), args.work.resolve())


if __name__ == "__main__":
    main()
