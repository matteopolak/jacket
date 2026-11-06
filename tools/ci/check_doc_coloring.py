#!/usr/bin/env python3
"""Compare the production Rust coloring interpreter and generated browser runtime.

Run under the shared heavy gate:
  PYTHONPATH=tools python3 tools/ci/check_doc_coloring.py --out /tmp/owned-coloring.json
"""
import argparse
from pathlib import Path
import subprocess

from toolkit.repo import ROOT


def run(output):
    """Retain the Rust fixture output and compare against reviewed spans in both languages."""
    if output.exists():
        raise ValueError("--out must be absent")
    fixtures = ROOT / "crates/jacket/src/doc/coloring/fixtures.json"
    subprocess.run(["cargo", "run", "--locked", "--no-default-features", "-p", "jacket", "--example", "doc_coloring", "--", str(fixtures), str(output)], cwd=ROOT, check=True)
    subprocess.run(["node", "--experimental-strip-types", "scripts/check-coloring.mjs", str(fixtures), str(output)], cwd=ROOT / "website/docs", check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    run(parser.parse_args().out.resolve())
