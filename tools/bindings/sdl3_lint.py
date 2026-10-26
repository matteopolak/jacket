#!/usr/bin/env python3
"""Lint owned SDL3 files and all six source-identical generated declaration sets.

jailint normally loads only the host branch of module.jai. Isolated module roots below
target/sdl3/lint/ make every other target's exact source visible to its existing rules.
This is source/type/lint validation, never execution of foreign target binaries.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
from bindings.sdl3_gen import ROOT, TARGETS
from repo_output import output_dir
from toolkit.hashing import sha256_file


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jailint", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    args = parser.parse_args()
    binary = str(args.jailint.resolve())
    stdlib = args.stdlib.resolve()
    # Everything is linted from the standard library given, never the checkout's copy: the
    # SDL3 link files import private helpers, which only that library's own files may do.
    sdl = stdlib / "Extensions/SDL3"
    env = dict(os.environ, JAIC_STDLIB=str(stdlib))
    common = [
        binary,
        "--config",
        str(ROOT / "jacket.toml"),
        "-D",
        "warnings",
        "-v",
        "-j",
        "2",
    ]

    def lint(paths):
        result = subprocess.run(
            [*common, *paths], env=env, cwd=ROOT, text=True, capture_output=True
        )
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        if (
            result.returncode
            or "did not compile completely" in result.stderr + result.stdout
        ):
            raise RuntimeError(
                "SDL3 lint requires a completely compiled program, not a syntax-only fallback"
            )

    lint([str(sdl), str(sdl / "tests")])
    roots = []
    for profile in TARGETS:
        directory = output_dir("sdl3", "lint", profile)
        directory.mkdir(parents=True, exist_ok=True)
        pairs = [
            (sdl / f"{profile}.jai", "declarations.jai"),
            (sdl / "macros.jai", "macros.jai"),
            (sdl / f"tests/driven/layout-{profile}.jai", "abi.jai"),
        ]
        # The link file is loaded in place, not copied (see `sdl`).
        link = (sdl / f"link-{profile.split('-')[0]}.jai").as_posix()
        for source, name in pairs:
            dest = directory / name
            shutil.copyfile(source, dest)
            if sha256_file(source) != sha256_file(dest):
                raise RuntimeError(f"nonidentical lint input: {source}")
        (directory / "module.jai").write_text(
            '// Isolated source/type lint root, not a runtime portability test.\nSDL3 :: #system_library "SDL3";\n#load "declarations.jai";\n#load "macros.jai";\n'
            f'#load "{link}";\n#load "abi.jai";\n'
        )
        roots.append(str(directory))
    lint(roots)
    print(
        "jailint -D warnings passed: owned files and all six source-identical declaration/ABI sets"
    )


if __name__ == "__main__":
    main()
