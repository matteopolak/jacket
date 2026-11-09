#!/usr/bin/env python3
"""Round-trip Archive and Python in both directions, including ZIP64 streaming descriptors.

Needs `jaic` on PATH and JAIC_STDLIB set; output goes under `target/archive-differential/`.

Example: PYTHONPATH=tools python3 tools/archive_conformance/differential.py --bench
"""
import io
from pathlib import Path
import subprocess
import sys
import tarfile
import zipfile
import random
import shutil
import statistics
import time

from repo_output import output_dir
from toolkit.repo import ROOT

NAMES = ["dir/", "dir/世界.txt", "a" * 180]
PAYLOADS = [b"", b"hello world\n", b"long path\n"]


class Unseekable(io.BytesIO):
    def seekable(self):
        return False

    def seek(self, *args):
        raise io.UnsupportedOperation("stream")


def main():
    work = output_dir("archive-differential")
    work.mkdir(parents=True, exist_ok=True)
    for variant in range(3):
        stream = Unseekable() if variant else io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            for name, data in zip(NAMES, PAYLOADS):
                info = zipfile.ZipInfo(name)
                info.compress_type = zipfile.ZIP_DEFLATED if data else zipfile.ZIP_STORED
                with archive.open(info, "w", force_zip64=variant == 2) as entry:
                    entry.write(data)
        (work / f"python-{variant}.zip").write_bytes(stream.getvalue())
    for suffix, mode in [("tar", "w"), ("tar.gz", "w:gz")]:
        with tarfile.open(work / f"python-0.{suffix}", mode, format=tarfile.PAX_FORMAT) as archive:
            for i, (name, data) in enumerate(zip(NAMES, PAYLOADS)):
                info = tarfile.TarInfo(name)
                info.size = len(data)
                info.mtime = -123.5 if i == 1 else 1234
                if not data:
                    info.type = tarfile.DIRTYPE
                archive.addfile(info, io.BytesIO(data))
    exe = work / "interop"
    subprocess.run(["jaic", str(ROOT / "stdlib/Extensions/Archive/tests/interop.jai"), "-exe", str(exe)], check=True)
    subprocess.run([str(exe), str(work)], check=True)
    with zipfile.ZipFile(work / "ours.zip") as archive:
        assert archive.namelist() == NAMES
        assert [archive.read(name) for name in NAMES] == PAYLOADS
    for suffix in ["tar", "tar.gz"]:
        with tarfile.open(work / f"ours.{suffix}") as archive:
            entries = archive.getmembers()
            assert [e.name.rstrip("/") for e in entries] == [n.rstrip("/") for n in NAMES]
            assert [archive.extractfile(e).read() for e in entries if e.isfile()] == PAYLOADS[1:]
    print("Python differential: 5 inputs and 3 outputs passed")
    if "--bench" in sys.argv:
        bench(work)


def bench_fixtures():
    """The shared 32 MiB inputs for the Jacket and Python archive benchmarks."""
    work = output_dir("archive-bench")
    work.mkdir(parents=True, exist_ok=True)
    payload = random.Random(42).randbytes(128 * 1024)
    for label in ["stored.zip", "deflate.zip", "plain.tar"]:
        path = work / label
        kind = "zip" if label.endswith("zip") else "tar"
        if kind == "zip":
            method = zipfile.ZIP_STORED if label.startswith("stored") else zipfile.ZIP_DEFLATED
            with zipfile.ZipFile(path, "w", compression=method) as archive:
                for i in range(256):
                    archive.writestr(f"files/{i:04d}", payload)
        else:
            with tarfile.open(path, "w", format=tarfile.USTAR_FORMAT) as archive:
                for i in range(256):
                    entry = tarfile.TarInfo(f"files/{i:04d}")
                    entry.size = len(payload)
                    archive.addfile(entry, io.BytesIO(payload))


def bench(work):
    """Compare the standard runner's cases with Python on the shared payloads."""
    import json
    bench_fixtures()
    work = output_dir("archive-bench")
    output = subprocess.check_output(["jacket", "bench", "--path", "stdlib/Extensions/Archive",
                                      "--format", "json"], cwd=ROOT, text=True)
    measured = {row["name"]: row for row in json.loads(output)["results"]}
    for label in ["stored.zip", "deflate.zip", "plain.tar"]:
        path = work / label
        kind = "zip" if label.endswith("zip") else "tar"
        data = path.read_bytes()
        python = []
        for _ in range(3):
            dest = work / "extract-python"
            dest.mkdir()
            start = time.perf_counter()
            archive = zipfile.ZipFile(io.BytesIO(data)) if kind == "zip" else tarfile.open(fileobj=io.BytesIO(data))
            entries = archive.infolist() if kind == "zip" else archive.getmembers()
            assert len(entries) == 256
            indexed = time.perf_counter()
            if kind == "zip":
                archive.extractall(dest)
            else:
                archive.extractall(dest, filter="data")
            done = time.perf_counter()
            python.append((indexed - start, done - indexed))
            archive.close()
            shutil.rmtree(dest)
        case = {"stored.zip": "stored", "deflate.zip": "deflate", "plain.tar": "tar"}[label]
        j = [measured[f"{case}_{operation}"]["time_ns"]["estimate"] / 1e9 for operation in ("index", "extract")]
        p = [statistics.median(v[i] for v in python) for i in range(2)]
        print(f"{label}: Jai list {j[0]*1000:.2f} ms, extract {32/j[1]:.1f} MiB/s; "
              f"Python list {p[0]*1000:.2f} ms, extract {32/p[1]:.1f} MiB/s")


if __name__ == "__main__":
    main()
