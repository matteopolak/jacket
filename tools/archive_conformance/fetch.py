#!/usr/bin/env python3
"""Unpack Go's BSD-licensed ZIP/TAR test fixtures from a pinned source archive into
`target/archive-conformance/`, where `stdlib/Extensions/Archive/tests/conformance.jai` reads them.
`tools/portable/build_portable.py data` runs this too.

Example: PYTHONPATH=tools python3 tools/archive_conformance/fetch.py
"""
import base64
import tarfile

from repo_output import output_dir
from toolkit.download import pinned

URL = "https://codeload.github.com/golang/go/tar.gz/refs/tags/go1.23.0"
SHA256 = "0bc1065ced4718d43c8cf945e7c321b6a8814defd4f7cae9588719494021bddf"


def fetch():
    """Verify the source digest and unpack only regular fixture files, never Go/Jai sources."""
    packed = pinned(URL, SHA256, "archive-conformance/go.tar.gz")
    target = output_dir("archive-conformance")
    with tarfile.open(packed) as archive:
        for member in archive:
            for kind in ("zip", "tar"):
                prefix = f"go-go1.23.0/src/archive/{kind}/testdata/"
                if member.isfile() and member.name.startswith(prefix):
                    name = member.name[len(prefix):]
                    if "/" in name or name in (".", ".."):
                        raise ValueError(name)
                    output = target / kind / name
                    output.parent.mkdir(parents=True, exist_ok=True)
                    data = archive.extractfile(member).read()
                    if name.endswith(".base64"):
                        output = output.with_suffix("")
                        data = base64.b64decode(data)
                    output.write_bytes(data)
    return target


if __name__ == "__main__":
    print(fetch())
