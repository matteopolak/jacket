"""Fetch every pinned image corpus member and verify its SHA-256 before use."""

import argparse
import hashlib
import io
import json
from pathlib import Path
import urllib.request
import zipfile

MANIFEST = Path(__file__).with_name("manifest.json")


def fetch(url, expected, destination):
    """Verify cached bytes too; for example fetch(url, digest, corpus / 'sample.png')."""
    if destination.exists():
        data = destination.read_bytes()
    else:
        request = urllib.request.Request(
            url, headers={"User-Agent": "jacket-image-conformance"}
        )
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read()
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        raise ValueError(f"hash mismatch for {destination.name}: {actual}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if not destination.exists():
        destination.write_bytes(data)
    return data


def fetch_all(output):
    """Download the complete manifest, retaining each source's license and case metadata."""
    manifest = json.loads(MANIFEST.read_text())
    for item in manifest["files"]:
        fetch(item["url"], item["sha256"], output / item["path"])
    bmp = manifest["sources"]["bmp"]
    archive = fetch(bmp["url"], bmp["sha256"], output / "bmpsuite-2.8.zip")
    destination = output / "bmpsuite"
    destination.mkdir(exist_ok=True)
    with zipfile.ZipFile(io.BytesIO(archive)) as bundle:
        for member in bundle.infolist():
            target = (destination / member.filename).resolve()
            if not target.is_relative_to(destination.resolve()):
                raise ValueError("unsafe BMP Suite archive path")
            if member.is_dir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bundle.read(member))
    (output / "source-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    fetch_all(parser.parse_args().output.resolve())
