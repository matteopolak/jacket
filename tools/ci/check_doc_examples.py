#!/usr/bin/env python3
"""Render real lint-reference examples through the producer and production Astro component.

Example: PYTHONPATH=tools python3 tools/ci/check_doc_examples.py --work /tmp/owned-doc-examples
Run through the shared heavy-work gate; --work must be absent.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess

from toolkit.repo import ROOT
RULES = ("self_assignment", "duplicate_condition", "use_after_free")


def run(work):
    """Keep generated code/diagnostics exact while checking explicit component ownership."""
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    work.mkdir(parents=True)
    builder = ROOT / "website/docs"
    generated = subprocess.run(["cargo", "run", "--locked", "-q", "-p", "jailint", "--example", "lint_reference"], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout
    (work / "lint-reference.md").write_text(generated)
    sections = dict(re.findall(r"^### ([^\n]+)\n(.*?)(?=^### |\Z)", generated, re.MULTILINE | re.DOTALL))
    cases = [{"example": name, "markdown": sections[name]} for name in RULES]
    (work / "cases.json").write_text(json.dumps(cases))
    subprocess.run(["cargo", "run", "--locked", "-q", "-p", "jacket", "--example", "doc_commonmark", "--", str(work / "cases.json"), str(work / "nodes.json")], cwd=ROOT, check=True)
    records = json.loads((work / "nodes.json").read_text())
    for record in records:
        groups = [node for node in record["nodes"] if node["kind"] == "example_group"]
        assert groups and groups[0]["title"] == "example", record["id"]
        assert groups[0]["result"]["kind"] == "diagnostic", record["id"]
        code = (ROOT / "tests/lint" / record["id"] / "example.jai").read_text().rstrip()
        assert groups[0]["source"]["source"]["source"].rstrip() == code, record["id"]
    matrix = "\n".join(f"{severity}: operation colors\n 1 - removed;\n 1 + added;\n 1 ~ replaced;\n   | --- secondary origin\n   | ^^^ primary origin\n" for severity in ("error", "warning"))
    records.append({"id":"diagnostic-operations","nodes":[{"kind":"code_block","language":"diagnostic","info":"diagnostic","source":{"source":matrix,"spans":[]}}]})
    (work / "nodes.json").write_text(json.dumps(records))
    pages = work / "src/pages"
    pages.mkdir(parents=True)
    source = "---\n" + f"import Document from {json.dumps(str(builder / 'src/components/Document.astro'))};\n"
    source += f"import {json.dumps(str(builder / 'src/styles/site.css'))};\n"
    source += "import {readFileSync} from 'node:fs';\n"
    source += "export function getStaticPaths(){const records=JSON.parse(readFileSync(" + json.dumps(str(work / "nodes.json")) + ",'utf8'));return records.map(record=>({params:{id:record.id},props:{record}}));}\nconst {record}=Astro.props;\nconst href=target=>target.url??'/'+(target.id??'');\n---\n"
    source += '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>{record.id}</title></head><body><main><div class="prose guide"><Document nodes={record.nodes} href={href}/></div></main></body></html>\n'
    (pages / "[id].astro").write_text(source)
    (work / "package.json").write_text('{"type":"module","private":true}')
    (work / "node_modules").symlink_to(builder / "node_modules", target_is_directory=True)
    (work / "astro.config.mjs").write_text("export default {output:'static',trailingSlash:'never',build:{format:'file'},compressHTML:false,devToolbar:{enabled:false}};\n")
    package = json.loads((builder / "node_modules/astro/package.json").read_text())
    cli = builder / "node_modules/astro" / package["bin"]["astro"]
    subprocess.run(["node", str(cli), "build", "--root", str(work)], cwd=work, check=True)
    subprocess.run(["node", str(builder / "scripts/check-examples.mjs"), str(work)], cwd=builder, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", required=True, type=Path)
    run(parser.parse_args().work.resolve())
