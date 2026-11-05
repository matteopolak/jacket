"""Prepare an authenticated opaque comparison checkout, without inspecting implementation.

    python3 prepare_community.py --archive /tmp/toml-jai.tar.gz --output /tmp/opaque-imports

Only the pinned archive is accepted. Existing checkouts are never replaced. Compile and run
the resulting module only through tools/bench Sandbox; do not view its implementation files.
"""
from __future__ import annotations
import argparse
import io
import json
from pathlib import Path, PurePosixPath
import tarfile
from run import fetch

HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    pin = json.loads((HERE / 'community-pin.json').read_text())
    args.archive.parent.mkdir(parents=True, exist_ok=True)
    data = fetch(args.archive, pin['archive_url'], pin['archive_sha256'])
    target = args.output.resolve() / 'toml-jai'
    if target.exists():
        raise RuntimeError('opaque checkout already exists; use a fresh output directory')
    files = 0
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for entry in archive.getmembers():
            parts = PurePosixPath(entry.name).parts
            if len(parts) < 2:
                continue
            relative = PurePosixPath(*parts[1:])
            if '..' in relative.parts or relative.is_absolute() or entry.issym() or entry.islnk():
                raise RuntimeError('unsafe archive member')
            if not entry.isfile():
                continue
            output = target.joinpath(*relative.parts)
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(archive.extractfile(entry).read())
            files += 1
    print(json.dumps(dict(revision=pin['revision'], archive_sha256=pin['archive_sha256'], opaque_files=files)))


if __name__ == '__main__':
    main()
