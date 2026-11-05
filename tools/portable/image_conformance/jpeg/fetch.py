#!/usr/bin/env python3
"""Download the complete pinned JPEG inputs and attribution; verify every byte before reuse."""

import argparse
import hashlib
import json
from pathlib import Path
import urllib.parse
import urllib.request


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--manifest", type=Path, default=Path(__file__).with_name("manifest.json")
    )
    ap.add_argument("--output", type=Path, required=True)
    args = ap.parse_args()
    manifest = json.loads(args.manifest.read_text())
    args.output.mkdir(parents=True, exist_ok=True)
    total = 0
    for entry in manifest["sources"]:
        path = args.output / entry["local_path"]
        if path.exists():
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise SystemExit(
                    f"hash mismatch in existing input: {entry['local_path']}"
                )
        else:
            request = urllib.request.Request(
                urllib.parse.quote(entry["url"], safe=":/%"),
                headers={"User-Agent": "JPEG-conformance-fetch"},
            )
            with urllib.request.urlopen(request, timeout=60) as response:
                data = response.read()
            if (
                len(data) != entry["bytes"]
                or hashlib.sha256(data).hexdigest() != entry["sha256"]
            ):
                raise SystemExit(
                    f"download hash/length mismatch: {entry['local_path']}"
                )
            path.parent.mkdir(parents=True, exist_ok=True)
            temporary = path.with_name(path.name + ".download")
            temporary.write_bytes(data)
            temporary.replace(path)
        total += 1
    (args.output / "fetch-receipt.json").write_text(
        json.dumps(
            {
                "manifest_sha256": hashlib.sha256(
                    args.manifest.read_bytes()
                ).hexdigest(),
                "verified_files": total,
                "jpeg_cases": len(manifest["cases"]),
            },
            indent=2,
        )
        + "\n"
    )
    print(
        f"Verified {total} pinned source files, including {len(manifest['cases'])} JPEG cases."
    )


if __name__ == "__main__":
    main()
