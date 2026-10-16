#!/usr/bin/env python3
"""Prepare hash-pinned public C header inputs and explicit SDL test scratch.

Print environment assignments for the caller/CI environment file. No SDK build,
compiler invocation, Git access or test execution. Cached bytes are revalidated;
offline mode never downloads. Only exact archive members needed by configuration
tests are staged, not a replacement SDK or fabricated configuration.
"""

import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import urllib.request
from sdl_test_support import repository_root


MEMBERS = ("CMakeLists.txt", "include/SDL2/SDL_config_unix.h.cmake", "LICENSE.txt")


def prepare(scratch, cache, offline=False):
    scratch, cache = Path(scratch), Path(cache)
    if not scratch.is_absolute() or not cache.is_absolute():
        raise ValueError("SDL scratch/cache must be absolute explicitly owned paths")
    scratch.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    pin = json.loads(
        (repository_root() / "tools/native_bindings/dependencies.json").read_text()
    )["sdl2-runtime-oracle"]["archive"]
    archive = cache / ("sdl2-runtime-oracle-" + pin["sha256"] + ".tar.gz")
    if not archive.exists():
        if offline:
            raise FileNotFoundError(
                "offline: missing pinned SDL2 header archive: " + str(archive)
            )
        data = urllib.request.urlopen(pin["url"], timeout=60).read()
        if hashlib.sha256(data).hexdigest() != pin["sha256"]:
            raise ValueError("SDL2 header download checksum mismatch")
        archive.write_bytes(data)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != pin["sha256"]:
        raise ValueError("SDL2 header cache checksum mismatch")
    source = scratch / "sdl-sdk-source"
    # Read all required real members before writing; never accept a partial or
    # substituted template. Extractfile avoids archive path traversal entirely.
    with tarfile.open(archive) as stream:
        contents = {}
        for member in MEMBERS:
            entry = stream.getmember(pin["root"] + "/" + member)
            if not entry.isfile():
                raise ValueError(
                    "SDL2 header archive member is not a regular file: " + member
                )
            contents[member] = stream.extractfile(entry).read()
    for member, data in contents.items():
        path = source / member
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    # A previous SDK configure output must not become this source-only oracle.
    (source / "include/SDL2/SDL_config_unix.h").unlink(missing_ok=True)
    (scratch / "sdl-sdk-test-inputs.json").write_text(
        json.dumps(
            {"archive": str(archive), "sha256": pin["sha256"], "root": pin["root"]},
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return {"SDL_TEST_TMPDIR": str(scratch), "SDL_SDK_TEST_SOURCE": str(source)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scratch", required=True, type=Path)
    parser.add_argument("--cache", required=True, type=Path)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    for key, value in prepare(args.scratch, args.cache, args.offline).items():
        if "\n" in value or "\r" in value:
            raise ValueError("SDL test environment paths cannot contain newlines")
        print(key + "=" + value)


if __name__ == "__main__":
    main()
