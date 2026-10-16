#!/usr/bin/env python3
"""Run producer behavior suites against the combined production library.

No staging or overlays. Every requested suite is required; independent suites
continue after failures and the aggregate exit stays nonzero. Native dependencies
and fonts must be real source-built or host assets.
"""
import argparse
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SUITES = ("foundations", "portable", "platform", "audio", "window", "numerics", "numerics-native", "bindings", "tooling")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--stdlib", required=True, type=Path)
    parser.add_argument("--jailint", required=True, type=Path)
    parser.add_argument("--jaifmt", required=True, type=Path)
    parser.add_argument("--suite", action="append", choices=SUITES)
    parser.add_argument("--native-libs", required=True, type=Path)
    parser.add_argument("--bindings-native-libs", type=Path, help="real source-built ABI8 SDK and C probe directory")
    parser.add_argument("--font", type=Path, help="real scalable font required by the window suite")
    args = parser.parse_args()
    selected = args.suite or list(SUITES)
    compiler, library = args.jaic.resolve(strict=True), args.stdlib.resolve(strict=True)
    lint, fmt = args.jailint.resolve(strict=True), args.jaifmt.resolve(strict=True)
    native = args.native_libs.resolve(strict=True)
    if library != (ROOT / "stdlib").resolve():
        parser.error("integration requires the combined production stdlib, not an overlay")
    if "window" in selected and (not args.font or not args.font.is_file()):
        parser.error("--font must name a real host font for the window suite")
    binding_native = args.bindings_native_libs.resolve(strict=True) if args.bindings_native_libs else native
    search = list(dict.fromkeys([str(binding_native), str(native), *filter(None, os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep))]))
    env = dict(os.environ, JAIC_STDLIB=str(library), JAIC_NATIVE_LIBS=os.pathsep.join(search))
    if args.font:
        env["JAIC_TEST_FONT_PATH"] = str(args.font.resolve(strict=True))
    commands = {
        "foundations": ["tools/check_foundations.py", "--jaic", compiler, "--stdlib", library,
                        "--jaifmt", fmt, "--jailint", lint, "--cross", "--wasm"],
        "portable": ["tools/check_portable_stdlib.py", "--jaic", compiler, "--stdlib", library,
                     "--native", "--keep-going"],
        "platform": ["tools/check_platform_io.py", "--jaic", compiler, "--stdlib", library,
                     "--native", "--cross-check", "--wasm"],
        "audio": ["tools/audio_verify.py", "--jaic", compiler, "--stdlib", library,
                  "--jaifmt", fmt, "--jailint", lint],
        "window": ["tools/check_window_render.py", "--jaic", compiler, "--stdlib", library,
                   "--native-libs", native, "--font", args.font],
        "numerics": ["tests/numerics/run.py", "--compiler", compiler, "--stdlib", library],
        "numerics-native": ["tests/numerics/corrective.py", "--compiler", compiler, "--stdlib", library],
        "bindings": ["tools/native_bindings/verify.py", "--compiler", compiler, "--stdlib", library,
                     "--native-libs", binding_native, "--runtime", "--work-dir", ROOT / "target/production-bindings/checks", "--report", ROOT / "target/production-bindings/results.json"],
        "tooling": ["tools/check_developer_tools.py"],
    }
    failures = []
    for suite in selected:
        print("RUN production suite:", suite, flush=True)
        result = subprocess.run([sys.executable, *map(str, commands[suite])], cwd=ROOT, env=env)
        print("RESULT production suite:", suite, "exit", result.returncode, flush=True)
        if result.returncode:
            failures.append(suite)
    print("Producer suites:", len(selected) - len(failures), "passed; failed:", ", ".join(failures) or "none", flush=True)
    return bool(failures)


if __name__ == "__main__":
    sys.exit(main())
