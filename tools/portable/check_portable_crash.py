#!/usr/bin/env python3
"""Verify bootstrap crash options using newly source-built Compiler workspaces."""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess

from portable_toolchain import checked_compiler
from toolkit.cli import add_jaic_argument
from toolkit.repo import ROOT
from repo_output import output_dir



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    add_jaic_argument(parser)
    parser.add_argument("--stdlib", type=Path, required=True)
    args = parser.parse_args()
    compiler = checked_compiler(args.jaic, parser)
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()))
    fixtures = ROOT / "stdlib/Runtime_Support_Crash_Handler/tests/driven/startup-crash"
    for enabled in [False, True]:
        output = output_dir("portable-tests") / ("startup-on" if enabled else "startup-off")
        output.mkdir(parents=True, exist_ok=True)
        executable = output / "startup-crash"
        executable.unlink(missing_ok=True)
        definitions = (f"OUTPUT_PATH :: {json.dumps(str(output) + '/')}; "
                       f"CHILD_SOURCE :: {json.dumps(str(fixtures / 'child.jai'))}; "
                       f"BACKTRACE_ENABLED :: {'true' if enabled else 'false'};")
        result = subprocess.run([str(compiler), "-interp", str(fixtures / "build.jai"),
                                 "-add", definitions, "--"], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=60)
        if result.returncode or result.stdout or result.stderr:
            raise RuntimeError(f"workspace build failed: {result.returncode}: {result.stdout}{result.stderr}")
        result = subprocess.run([str(executable)], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=20)
        if result.returncode != -signal.SIGSEGV or result.stdout:
            raise RuntimeError(f"unexpected startup fault: {result.returncode}: {result.stdout}{result.stderr}")
        if enabled:
            if "crash signal 11\nnative stack:\n" not in result.stderr:
                raise RuntimeError(f"startup failed to install crash handler: {result.stderr}")
        elif result.stderr:
            raise RuntimeError(f"disabled crash handler produced diagnostics: {result.stderr}")
        print(f"passed native Build_Options.backtrace_on_crash={'ON' if enabled else 'OFF'}", flush=True)


if __name__ == "__main__":
    main()
