"""Fetch pinned csv-spectrum fixtures and make the CSV benchmark input.

Run with PYTHONPATH=tools python3 tools/csv_conformance/fetch.py.
"""
from __future__ import annotations

import tarfile
from pathlib import Path

from repo_output import output_dir
from toolkit.download import pinned

REVISION = "d30e80f8b99d2eecb3778f1d7b9ed1cb425502ec"
SHA256 = "5b8ddb407b11fc44cf363041a5372c003adc801f13078a279ba726e81850a7ed"


def fetch() -> Path:
    """Materialize only CSV and expected JSON files from the verified archive."""
    archive = pinned(
        f"https://codeload.github.com/max-mapper/csv-spectrum/tar.gz/{REVISION}",
        SHA256,
        f"csv-spectrum/{REVISION}.tar.gz",
    )
    target = output_dir("csv-conformance")
    prefix = f"csv-spectrum-{REVISION}/"
    with tarfile.open(archive) as tar:
        for member in tar:
            relative = member.name.removeprefix(prefix)
            path = Path(relative)
            if member.isfile() and len(path.parts) == 2 and path.parts[0] in ("csvs", "json"):
                destination = target / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = tar.extractfile(member)
                assert source is not None
                destination.write_bytes(source.read())
    # Upstream's expected JSON gives a different phone number than its own CSV.
    expected = target / "json" / "location_coordinates.json"
    expected.write_bytes(expected.read_bytes().replace(b'"1234567890"', b'"2095257564"'))
    return target


def bench_input() -> Path:
    """A ~50 MB CSV of quoted and unquoted columns for `jacket bench` and bench.py."""
    benchmark = output_dir("csv-conformance", "bench.csv")
    row = b'12345,"Ada, Lovelace",3.14159,true,"a ""quoted"" line"\r\n'
    if not benchmark.exists():
        benchmark.parent.mkdir(parents=True, exist_ok=True)
        with benchmark.open("wb") as out:
            out.write(b"id,name,value,active,note\r\n")
            block = row * 10000
            for _ in range(50_000_000 // len(block) + 1):
                out.write(block)
    return benchmark


if __name__ == "__main__":
    print(fetch())
    print(bench_input())
