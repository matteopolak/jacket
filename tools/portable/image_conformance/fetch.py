"""Fetch the pinned PNG, BMP, PNM, QOI and ICO corpora and derive their case lists.

    PYTHONPATH=tools python3 tools/portable/image_conformance/fetch.py --output target/image-corpus

`manifest.json` pins each source set by repository, revision and the SHA-256 of every file; the
BMP Suite is one pinned zip. Files are cached by `toolkit.download` and verified before use.
"""

import argparse
import io
import json
from pathlib import Path
import struct
import zipfile

from toolkit.download import pinned_to
from toolkit.hashing import sha256_bytes, sha256_file

MANIFEST = Path(__file__).with_name("manifest.json")
RAW = "https://raw.githubusercontent.com/{repository}/{revision}/{path}"


def pins(manifest):
    """Every pinned file as {local path: (url, sha256)}; the local path is `<set>/<repository path>`."""
    found = {}
    for source in manifest["sets"]:
        repository = source["repository"].removeprefix("https://github.com/")
        for path, digest in source["files"].items():
            url = RAW.format(repository=repository, revision=source["revision"], path=path)
            found[f"{source['name']}/{path}"] = (url, digest)
    return found


def png_cases(corpus, oracle):
    """One case per PngSuite file: valid unless it starts with `x`, with its IHDR size and oracle pin.

    For example `png_cases(corpus, {"basi0g01.png": digest})[1]["width"] == 32`.
    """
    root = corpus / "pngsuite"
    names = ["PngSuite.png"] + sorted(p.name for p in (root / "png").glob("*.png"))
    cases = []
    for name in names:
        data = (root / (name if name == "PngSuite.png" else "png/" + name)).read_bytes()
        valid = not name.startswith("x")
        width, height = struct.unpack(">II", data[16:24]) if valid else (0, 0)
        case = {
            "name": name,
            "valid": valid,
            "width": width,
            "height": height,
            "sha256": sha256_bytes(data),
        }
        if valid:
            case["oracle_sha256"] = oracle[name]
        cases.append(case)
    return cases


def place(url, digest, destination):
    """Put the pinned file at `destination`; an existing file must already match."""
    if destination.exists():
        if sha256_file(destination) != digest:
            raise SystemExit(f"hash mismatch in existing input: {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    pinned_to(url, digest, destination)


def fetch_all(output):
    """Download every pinned file into `output`, unpack the BMP Suite and return the derived cases."""
    manifest = json.loads(MANIFEST.read_text())
    for local_path, (url, digest) in pins(manifest).items():
        place(url, digest, output / local_path)
    bmp = manifest["bmp"]
    archive = output / "bmpsuite-2.8.zip"
    place(bmp["url"], bmp["sha256"], archive)
    destination = output / "bmpsuite"
    destination.mkdir(exist_ok=True)
    with zipfile.ZipFile(io.BytesIO(archive.read_bytes())) as bundle:
        for member in bundle.infolist():
            target = (destination / member.filename).resolve()
            if not target.is_relative_to(destination.resolve()):
                raise ValueError("unsafe BMP Suite archive path")
            if member.is_dir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bundle.read(member))
    manifest["png_cases"] = png_cases(output, manifest["png_oracle"])
    (output / "source-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    fetch_all(parser.parse_args().output.resolve())
