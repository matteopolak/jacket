#!/usr/bin/env python3
"""Build the independent Linux X11 clipboard service; never fetch SDK sources."""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
from toolkit.hashing import sha256_file
from toolkit.repo import ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Linux":
        parser.error(
            "native clipboard archive requires a Linux SDK/host; source checks are separate"
        )
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = ROOT / "stdlib/internal/clipboard/native"
    compiler = os.environ.get("CC", "cc")
    commands = []
    for name in ["data", "x11"]:
        command = [
            compiler,
            "-std=c11",
            "-O2",
            "-fPIC",
            "-fvisibility=hidden",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pthread",
            "-c",
            str(source / (name + ".c")),
            "-o",
            str(output / (name + ".o")),
        ]
        subprocess.run(command, check=True)
        commands.append(command)
    archive = output / "libportable_clipboard.a"
    command = [
        os.environ.get("AR", "ar"),
        "rcs",
        str(archive),
        str(output / "data.o"),
        str(output / "x11.o"),
    ]
    subprocess.run(command, check=True)
    commands.append(command)
    shared = output / "libportable_clipboard.so"
    command = [
        compiler,
        "-shared",
        "-pthread",
        str(output / "data.o"),
        str(output / "x11.o"),
        "-lX11",
        "-o",
        str(shared),
    ]
    subprocess.run(command, check=True)
    commands.append(command)
    manifest = dict(
        provenance="Independent project implementation; system Xlib and pthread ABI only",
        host=platform.platform(),
        commands=commands,
        required_system_libraries=["X11", "pthread"],
        static_consumer_dependencies=["X11", "pthread"],
        static_consumer_retention="Required even for a type-only X11 import; the Jai consumer declares private link_always dependencies.",
        inputs={
            str(p.relative_to(ROOT)): sha256_file(p)
            for p in [source / "data.h", source / "data.c", source / "x11.c"]
        },
        archive_sha256=sha256_file(archive),
        shared_sha256=sha256_file(shared),
    )
    (output / "portable-clipboard-build.json").write_text(
        json.dumps(manifest, indent=2) + "\n"
    )


if __name__ == "__main__":
    main()
