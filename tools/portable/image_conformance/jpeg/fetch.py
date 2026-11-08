#!/usr/bin/env python3
"""Download the pinned JPEG corpus and attribution files and verify every byte.

    PYTHONPATH=tools python3 tools/portable/image_conformance/jpeg/fetch.py --output target/jpeg-corpus
"""

import argparse
import json
from pathlib import Path

from inventory import cases, check_corpus, sources
from toolkit.download import pinned_to
from toolkit.hashing import sha256_file


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--manifest", type=Path, default=Path(__file__).with_name("manifest.json")
    )
    ap.add_argument("--output", type=Path, required=True)
    args = ap.parse_args()
    manifest = json.loads(args.manifest.read_text())
    pins = sources(manifest)
    for local_path, pin in pins.items():
        path = args.output / local_path
        if path.exists():
            if sha256_file(path) != pin["sha256"]:
                raise SystemExit(f"hash mismatch in existing input: {local_path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            pinned_to(pin["url"], pin["sha256"], path)
    errors = check_corpus(manifest, args.output, sha256_file)
    if errors:
        raise SystemExit("\n".join(errors))
    (args.output / "fetch-receipt.json").write_text(
        json.dumps(
            {
                "manifest_sha256": sha256_file(args.manifest),
                "verified_files": len(pins),
                "jpeg_cases": len(cases(manifest)),
            },
            indent=2,
        )
        + "\n"
    )
    print(
        f"Verified {len(pins)} pinned source files, including {len(cases(manifest))} JPEG cases."
    )


if __name__ == "__main__":
    main()
