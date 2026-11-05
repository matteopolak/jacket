#!/usr/bin/env python3
"""Generate the docs schema, TypeScript declarations and browser validators from Rust DTOs.

    PYTHONPATH=tools python3 tools/generators/generate_doc_site.py [--check]

Install website/docs dependencies explicitly before using this generator. The check
compares a temporary Rust schema and invokes the pinned JavaScript generator's check mode.
"""

import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

from toolkit.repo import ROOT

OUTPUTS = [
    "website/docs/schema/doc-site.schema.json",
    "website/docs/src/generated/doc-site.d.ts",
    "website/docs/src/generated/validate-manifest.js",
    "website/docs/src/generated/validate-manifest.d.ts",
    "website/docs/src/generated/validate-site.js",
    "website/docs/src/generated/validate-site.d.ts",
    "website/docs/src/generated/validate-module.js",
    "website/docs/src/generated/validate-module.d.ts",
    "website/docs/src/generated/validate-document.js",
    "website/docs/src/generated/validate-document.d.ts",
    "website/docs/src/generated/validate-search.js",
    "website/docs/src/generated/validate-search.d.ts",
    "website/docs/src/generated/validate-example.js",
    "website/docs/src/generated/validate-example.d.ts",
    "website/docs/src/generated/validate-navigation.js",
    "website/docs/src/generated/validate-navigation.d.ts",
]


def generate(check=False, *, root=ROOT, run=subprocess.run):
    """Reproduce the one wire authority; e.g. generate(True) must leave tracked files untouched."""
    builder = root / "website/docs"
    for package in ("ajv", "json-schema-to-typescript"):
        if not (builder / "node_modules" / package / "package.json").is_file():
            raise ValueError(
                "docs generation needs explicit setup: run npm ci in website/docs first"
            )
    node = shutil.which("node")
    if not node:
        raise ValueError(
            "docs generation needs the Node release in website/docs/.node-version"
        )
    with tempfile.TemporaryDirectory(prefix="jacket-doc-schema-") as temporary:
        output = Path(temporary) / "schema.json"
        run(
            [
                "cargo",
                "run",
                "--locked",
                "-q",
                "-p",
                "jacket",
                "--example",
                "doc_site_schema",
                "--",
                str(output),
            ],
            cwd=root,
            check=True,
        )
        schema = output.read_bytes()
        target = root / OUTPUTS[0]
        if check:
            if not target.is_file() or target.read_bytes() != schema:
                raise ValueError(
                    "documentation schema is stale; rerun tools/generators/generate_doc_site.py without --check"
                )
        elif not target.is_file() or target.read_bytes() != schema:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(schema)
        run(
            [
                node,
                str(builder / "scripts/generate-types.mjs"),
                *(["--check"] if check else []),
            ],
            cwd=builder,
            check=True,
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        generate(args.check)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"documentation generation: {error}\n")
    print("Documentation schema, TypeScript and browser validators are current")


if __name__ == "__main__":
    main()
