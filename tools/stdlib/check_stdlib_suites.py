#!/usr/bin/env python3
"""Run the suites that need special flags, fixtures or hosts against the production library.

No staging or overlays. Every requested suite is required; independent suites
continue after failures and the aggregate exit stays nonzero. Native dependencies
and fonts must be real source-built or host assets.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import concurrent.futures
import os
from pathlib import Path
import subprocess
import sys
from toolkit.repo import ROOT
from toolkit.cli import add_jaic_argument

SUITES = (
    "foundations",
    "portable",
    "platform",
    "audio",
    "window",
    "numerics",
    "numerics-native",
    "bindings",
    "tooling",
)

# Suites that share scratch directories, the X display or the audio device run one after the other
# inside a group; the groups are independent and run concurrently under --jobs. The slowest group
# comes first so it starts at once.
GROUPS = (
    ("tooling",),
    ("bindings",),
    ("foundations",),
    ("portable", "platform"),
    ("audio", "window", "numerics", "numerics-native"),
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    add_jaic_argument(parser, required=True)
    parser.add_argument("--stdlib", required=True, type=Path)
    parser.add_argument("--jailint", required=True, type=Path)
    parser.add_argument("--jaifmt", required=True, type=Path)
    parser.add_argument("--suite", action="append", choices=SUITES)
    parser.add_argument(
        "--exclude",
        action="append",
        choices=SUITES,
        default=[],
        help="leave this suite out of the selection (repeatable)",
    )
    parser.add_argument(
        "--jobs",
        type=int,
        default=1,
        help="independent suite groups to run at once (default 1: one suite at a time)",
    )
    parser.add_argument("--native-libs", required=True, type=Path)
    parser.add_argument(
        "--bindings-native-libs",
        type=Path,
        help="real source-built ABI8 SDK and C probe directory",
    )
    parser.add_argument(
        "--font", type=Path, help="real scalable font required by the window suite"
    )
    parser.add_argument(
        "--private-x11-display",
        action="store_true",
        help="Caller owns an ephemeral X11 display for Linux clipboard tests",
    )
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be at least 1")
    selected = [s for s in (args.suite or SUITES) if s not in args.exclude]
    compiler, library = args.jaic.resolve(strict=True), args.stdlib.resolve(strict=True)
    lint, fmt = args.jailint.resolve(strict=True), args.jaifmt.resolve(strict=True)
    native = args.native_libs.resolve(strict=True)
    if library != (ROOT / "stdlib").resolve():
        parser.error(
            "integration requires the combined production stdlib, not an overlay"
        )
    if "window" in selected and (not args.font or not args.font.is_file()):
        parser.error("--font must name a real host font for the window suite")
    binding_native = (
        args.bindings_native_libs.resolve(strict=True)
        if args.bindings_native_libs
        else native
    )
    search = list(
        dict.fromkeys(
            [
                str(binding_native),
                str(native),
                *filter(None, os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep)),
            ]
        )
    )
    env = dict(
        os.environ, JAIC_STDLIB=str(library), JAIC_NATIVE_LIBS=os.pathsep.join(search)
    )
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    if args.font:
        env["JAIC_TEST_FONT_PATH"] = str(args.font.resolve(strict=True))
    commands = {
        "foundations": [
            "tools/stdlib/check_foundations.py",
            "--jaic",
            compiler,
            "--stdlib",
            library,
            "--jaifmt",
            fmt,
            "--jailint",
            lint,
            "--cross",
            "--wasm",
        ],
        "portable": [
            "tools/portable/check_portable_stdlib.py",
            "--jaic",
            compiler,
            "--stdlib",
            library,
            "--native",
            "--keep-going",
        ],
        "platform": [
            "tools/stdlib/check_platform_io.py",
            "--jaic",
            compiler,
            "--stdlib",
            library,
            "--native",
            "--cross-check",
            "--wasm",
        ],
        "audio": [
            "tools/stdlib/check_audio_output.py",
            "--jaic",
            compiler,
            "--native-libs",
            native,
        ],
        "window": [
            "tools/window_render/check_window_render.py",
            "--jaic",
            compiler,
            "--stdlib",
            library,
            "--native-libs",
            native,
            "--font",
            args.font,
        ],
        "numerics": [
            "tools/numerics/run.py",
            "--compiler",
            compiler,
            "--stdlib",
            library,
        ],
        "numerics-native": [
            "tools/numerics/corrective.py",
            "--compiler",
            compiler,
            "--stdlib",
            library,
        ],
        "bindings": [
            "tools/native_bindings/verify.py",
            "--compiler",
            compiler,
            "--stdlib",
            library,
            "--native-libs",
            binding_native,
            "--runtime",
            "--work-dir",
            ROOT / "target/production-bindings/checks",
            "--report",
            ROOT / "target/production-bindings/results.json",
        ],
        "tooling": ["tools/developer_checks/check_developer_tools.py"],
    }
    if args.private_x11_display:
        commands["numerics-native"].append("--private-x11-display")
    failures = []

    def run_suite(suite, capture):
        print("RUN suite:", suite, flush=True)
        result = subprocess.run(
            [sys.executable, *map(str, commands[suite])],
            cwd=ROOT,
            env=env,
            **(
                dict(stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                if capture
                else {}
            ),
        )
        # A concurrent suite's output is printed whole when it ends, so lines never interleave.
        if capture:
            sys.stdout.write(result.stdout)
        print("RESULT suite:", suite, "exit", result.returncode, flush=True)
        return result.returncode

    def run_group(group):
        return [
            suite
            for suite in group
            if run_suite(suite, args.jobs > 1)
        ]

    groups = [g for g in ([s for s in group if s in selected] for group in GROUPS) if g]
    if args.jobs == 1:
        # Keep the requested order when running serially.
        failures = [suite for suite in selected if run_suite(suite, False)]
    else:
        with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
            for failed in pool.map(run_group, groups):
                failures.extend(failed)
    print(
        "Suites:",
        len(selected) - len(failures),
        "passed; failed:",
        ", ".join(failures) or "none",
        flush=True,
    )
    return bool(failures)


if __name__ == "__main__":
    sys.exit(main())
