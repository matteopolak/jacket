#!/usr/bin/env python3
"""Pin and account for every CommonMark case through the real producer and Astro.

Run inside the shared heavy-work gate, for example:
  PYTHONPATH=tools python3 tools/ci/check_doc_commonmark.py --work /tmp/owned-commonmark
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from toolkit.repo import ROOT
FIXTURES = Path(__file__).parent / "fixtures/commonmark"


def load_cases(root):
    """Reject altered or incomplete fixtures before starting builds."""
    manifest = json.loads((root / "manifest.json").read_text())
    raw = (root / "spec.json").read_bytes()
    if hashlib.sha256(raw).hexdigest() != manifest["sha256"]:
        raise ValueError("CommonMark corpus digest differs from the pinned source")
    if "licenseFile" in manifest:
        license_raw = (root / manifest["licenseFile"]).read_bytes()
        if hashlib.sha256(license_raw).hexdigest() != manifest["licenseSha256"]:
            raise ValueError("CommonMark license digest differs from the pinned source")
    if "transformations" in manifest:
        group = manifest["transformations"]
        if hashlib.sha256((root / group["file"]).read_bytes()).hexdigest() != group["sha256"]:
            raise ValueError("reviewed transformation digest differs")
    cases = json.loads(raw)
    if [case["example"] for case in cases] != list(range(1, manifest["count"] + 1)):
        raise ValueError("CommonMark case IDs must be complete, unique and contiguous")
    for case in cases:
        if any(not isinstance(case.get(key), str) for key in ("markdown", "html", "section")):
            raise ValueError("malformed CommonMark case")
    if "extensions" in manifest:
        extension = manifest["extensions"]
        raw = (root / extension["file"]).read_bytes()
        if hashlib.sha256(raw).hexdigest() != extension["sha256"]:
            raise ValueError("extension corpus digest differs from the pinned source")
        extra = json.loads(raw)
        if len(extra) != extension["count"] or len({row["example"] for row in extra}) != len(extra):
            raise ValueError("extension case inventory is incomplete or duplicated")
        cases.extend(extra)
    return cases


def validate_ledger(cases, ledger):
    """Every original case must pass; there is no silent skip or expected-failure channel."""
    expected = {case["example"] for case in cases}
    ids = [row.get("id") for row in ledger]
    if len(ids) != len(set(ids)) or set(ids) != expected:
        raise ValueError("CommonMark ledger IDs are missing, duplicated or unknown")
    failures = [row for row in ledger if row.get("status") != "pass"]
    if failures:
        raise ValueError(f"{len(failures)} CommonMark cases did not pass")


def run(work, fixtures=FIXTURES):
    """Build an isolated route importing the production component, then compare independent trees."""
    cases = load_cases(fixtures)
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    work.mkdir(parents=True)
    builder = ROOT / "website/docs"
    (work / "cases.json").write_text(json.dumps(cases))
    payload = work / "producer.json"
    subprocess.run(["cargo", "run", "--locked", "--no-default-features", "-p", "jacket", "--example", "doc_commonmark",
                    "--", str(work / "cases.json"), str(payload)], cwd=ROOT, check=True)
    source = work / "src/pages"
    source.mkdir(parents=True)
    component = builder / "src/components/Document.astro"
    (source / "[id].astro").write_text("---\n"
        + f"import Document from {json.dumps(str(component))};\n"
        + f"import {{fixtureURL}} from {json.dumps(str(builder / 'scripts/check-commonmark.mjs'))};\n"
        + f"import data from {json.dumps(str(payload))};\n"
        + "export function getStaticPaths() { return data.map(record => ({params:{id:String(record.id)},props:{record}})); }\n"
        + "const {record} = Astro.props;\n"
        + "---\n<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>CommonMark fixture</title></head>"
        + "<body><main id=\"fixture\"><Document nodes={record.nodes} href={target => fixtureURL(target.url)} /></main></body></html>\n")
    (work / "package.json").write_text('{"type":"module","private":true}')
    (work / "node_modules").symlink_to(builder / "node_modules", target_is_directory=True)
    config = work / "astro.config.mjs"
    config.write_text("export default {output:'static',trailingSlash:'never',build:{format:'file'},compressHTML:false,devToolbar:{enabled:false}};\n")
    package = json.loads((builder / "node_modules/astro/package.json").read_text())
    cli = builder / "node_modules/astro" / package["bin"]["astro"]
    subprocess.run(["node", str(cli), "build", "--root", str(work)], cwd=work, check=True)
    ledger = work / "ledger.json"
    subprocess.run(["node", str(builder / "scripts/check-commonmark.mjs"), str(work / "cases.json"),
                    str(payload), str(work / "dist"), str(ledger)], cwd=builder, check=True)
    validate_ledger(cases, json.loads(ledger.read_text()))
    print(f"CommonMark: {len(cases)} original cases passed; ledger {ledger}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    if not shutil.which("node"):
        parser.error("install pinned Node and run npm ci in website/docs first")
    run(args.work.resolve())


if __name__ == "__main__":
    main()
