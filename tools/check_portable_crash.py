#!/usr/bin/env python3
"""Verify bootstrap crash options using newly source-built Compiler workspaces."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess

from portable_toolchain import default_compiler, checked_compiler

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=default_compiler())
    parser.add_argument("--stdlib", type=Path, required=True)
    args = parser.parse_args()
    compiler = checked_compiler(args.jaic, parser)
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()))
    fixtures = ROOT / "stdlib/Runtime_Support_Crash_Handler/tests/driven/startup-crash"
    for enabled in [False, True]:
        output = ROOT / "target/portable-tests" / ("startup-on" if enabled else "startup-off")
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
