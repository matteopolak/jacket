#!/usr/bin/env python3
"""Run the unittest suites under tools/: one `unittest discover` per folder.

tools/ has no __init__.py, so a plain `discover -s tools` never descends into the subfolders.
Each folder is discovered on its own (unittest puts that folder on sys.path, so a test imports its
siblings by bare name), with tools/ on PYTHONPATH so cross-folder imports are folder-qualified
(`from native.fetch_url import fetch`), exactly as they are when a script runs directly.

    python3 tools/run_tests.py                       # every test_*.py
    python3 tools/run_tests.py -p 'test_sdl*.py' -v  # only the matching files, verbosely
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
NO_TESTS_RAN = 5  # unittest's exit status when a folder has no file matching the pattern


def folders():
    """tools/ itself, then each subfolder."""
    return [TOOLS, *sorted(p for p in TOOLS.iterdir() if p.is_dir() and not p.name.startswith(("_", ".")))]


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-p", "--pattern", default="test_*.py", help="test file pattern (default: %(default)s)")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    env = dict(os.environ, PYTHONPATH=os.pathsep.join([str(TOOLS), os.environ.get("PYTHONPATH", "")]))
    total, failed = 0, []
    for folder in folders():
        command = [sys.executable, "-m", "unittest", "discover", "-s", str(folder), "-p", args.pattern]
        if args.verbose:
            command.append("-v")
        result = subprocess.run(command, env=env, text=True, stderr=subprocess.PIPE, stdout=subprocess.PIPE)
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        ran = re.search(r"^Ran (\d+) tests?", result.stderr, re.M)
        total += int(ran.group(1)) if ran else 0
        if result.returncode not in (0, NO_TESTS_RAN):
            failed.append(folder.name if folder != TOOLS else "tools")
    print(f"tools/run_tests.py: {total} tests; failed folders: {', '.join(failed) or 'none'}")
    return 1 if failed or total == 0 else 0


if __name__ == "__main__":
    sys.exit(main())
