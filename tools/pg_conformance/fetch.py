#!/usr/bin/env python3
"""Fetches PostgreSQL's test suites at pinned versions for the Postgres conformance runs.

The source tarballs are downloaded once into the download cache and checked against their
SHA-256; only the test directories are unpacked, under `target/pg-conformance/src/<version>/`.
Nothing is copied into the repository (docs/postgres/conformance.md).

    python3 tools/pg_conformance/fetch.py            # every pinned version
    python3 tools/pg_conformance/fetch.py 17         # one major version
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tarfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from repo_output import shared_output_dir
from toolkit.download import pinned

# Major version: (release, SHA-256 of postgresql-<release>.tar.bz2). The digests are the ones
# published next to each tarball on ftp.postgresql.org.
VERSIONS = {
    "15": ("15.19", "e1a64a87a46b825b88c082e4518161a47aab53c45694964f8ba1df28f7859f89"),
    "16": ("16.15", "c1575341fa7bd40f5274ea465b34390f4dc64cdd0770af327005caaeb9f6b7ed"),
    "17": ("17.11", "dd27f2b3c59e73ed14aa3324901242bf69a032a6347805f274e6260322d42979"),
    "18": ("18.6", "555610c24d53e4316da5b7d3fc25c279d96856d5e0e23ee308c328c5fa881d9f"),
}

# The parts of the source tree the runners use.
SUITES = (
    "src/test/regress",
    "src/test/authentication",
    "src/test/ssl",
    "src/test/perl",
    "src/interfaces/libpq/t",
    "src/test/modules/libpq_pipeline",
    "src/bin/psql/t",
    "contrib/spi",  # autoinc and refint, which the regression suite's triggers test loads
)


def source_dir(major: str) -> Path:
    """Where the unpacked suites of one major version live."""
    return shared_output_dir("pg-conformance", "src", VERSIONS[major][0])


def fetch(major: str) -> Path:
    release, digest = VERSIONS[major]
    target = source_dir(major)
    stamp = target / ".complete"
    # The stamp names the release and the parts unpacked, so adding a suite unpacks again.
    contents = "\n".join([release, *SUITES]) + "\n"
    if stamp.exists() and stamp.read_text() == contents:
        return target

    name = f"postgresql-{release}.tar.bz2"
    archive = pinned(f"https://ftp.postgresql.org/pub/source/v{release}/{name}", digest, f"postgres/{name}")
    prefix = f"postgresql-{release}/"

    if target.exists():
        shutil.rmtree(target)
    target.mkdir(parents=True)

    with tarfile.open(archive, "r:bz2") as tar:
        members = []
        for member in tar.getmembers():
            if not member.name.startswith(prefix):
                continue
            relative = member.name[len(prefix):]
            if any(relative == suite or relative.startswith(suite + "/") for suite in SUITES):
                member.name = relative
                members.append(member)
        tar.extractall(target, members=members, filter="data")

    stamp.write_text(contents)
    return target


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("majors", nargs="*", help=f"major versions: {', '.join(VERSIONS)} (default: all)")
    args = parser.parse_args()
    unknown = [major for major in args.majors if major not in VERSIONS]
    if unknown:
        parser.error(f"unknown version {unknown[0]}; pinned: {', '.join(VERSIONS)}")
    for major in args.majors or VERSIONS:
        print(fetch(major))


if __name__ == "__main__":
    main()
