#!/usr/bin/env python3
"""Run the unittest suites under tools/: one `unittest discover` per folder.

tools/ has no __init__.py, so a plain `discover -s tools` never descends into the subfolders.
Every folder, at any depth, holding a matching file is discovered on its own (unittest puts that folder on sys.path, so a test imports its
siblings by bare name), with tools/ on PYTHONPATH so cross-folder imports are folder-qualified
(`from toolkit.download import fetch`), exactly as they are when a script runs directly.

    python3 tools/run_tests.py                       # every test_*.py
    python3 tools/run_tests.py -p 'test_sdl*.py' -v  # only the matching files, verbosely
"""
import argparse
import fnmatch
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
NO_TESTS_RAN = 5  # unittest's exit status when a folder has no file matching the pattern


def folders(pattern="test_*.py"):
    """tools/ itself, then every subfolder (at any depth) holding a file that matches `pattern`."""
    found = [TOOLS]
    for directory, names, files in os.walk(TOOLS):
        names[:] = sorted(n for n in names if not n.startswith(("_", ".")))
        here = Path(directory)
        if here != TOOLS and any(fnmatch.fnmatch(f, pattern) for f in files):
            found.append(here)
    return found


def sdl_environment(env):
    """The SDL tests need hash-pinned header inputs (tools/bindings/prepare_sdl_python_tests.py);
    CI prepares them first. Do it here when the caller has not, caching the download under target/."""
    if env.get("SDL_TEST_TMPDIR"):
        return {}
    prepare = TOOLS / "bindings" / "prepare_sdl_python_tests.py"
    scratch = str(Path(tempfile.mkdtemp(prefix="sdl-python-")).resolve())  # /var is a symlink on macOS
    cache = TOOLS.parent / "target" / "sdl-python-cache"
    result = subprocess.run([sys.executable, str(prepare), "--scratch", scratch, "--cache", str(cache)],
                            text=True, capture_output=True)
    if result.returncode:
        sys.stderr.write("run_tests.py: could not prepare the SDL test inputs (no network?); "
                         "the tools/bindings SDL tests will fail:\n" + result.stderr[-400:])
        return {}
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)


def ensure_formatter():
    """The generator tests format with target/jaifmt (tools/generators/jai_codegen.py). Build it when it is
    missing, as `tools/dev/dev.py fmt` does, and place it where they look; False when that was not possible."""
    wanted = TOOLS.parent / "target" / "jaifmt"
    if wanted.is_file():
        return True
    try:
        sys.path.insert(0, str(TOOLS))
        from dev.dev import ensure_jaifmt

        built = Path(ensure_jaifmt(str(os.cpu_count() or 1)))
        if built.resolve() != wanted.resolve():
            wanted.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(built, wanted)
    except (Exception, SystemExit) as error:
        sys.stderr.write(f"run_tests.py: skipping tools/generators, target/jaifmt is missing and could not be built ({error})\n")
        return False
    return wanted.is_file()


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-p", "--pattern", default="test_*.py", help="test file pattern (default: %(default)s)")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    env = dict(os.environ, PYTHONPATH=os.pathsep.join([str(TOOLS), os.environ.get("PYTHONPATH", "")]))
    if any(fnmatch.fnmatch(f.name, args.pattern) for f in (TOOLS / "bindings").glob("test_sdl*.py")):
        env.update(sdl_environment(env))
    total, failed = 0, []
    for folder in folders(args.pattern):
        if folder.name == "generators" and not ensure_formatter():
            continue
        command = [sys.executable, "-m", "unittest", "discover", "-s", str(folder), "-p", args.pattern]
        if args.verbose:
            command.append("-v")
        result = subprocess.run(command, env=env, text=True, stderr=subprocess.PIPE, stdout=subprocess.PIPE)
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        ran = re.search(r"^Ran (\d+) tests?", result.stderr, re.M)
        total += int(ran.group(1)) if ran else 0
        if result.returncode not in (0, NO_TESTS_RAN):
            failed.append(str(folder.relative_to(TOOLS)) if folder != TOOLS else "tools")
    print(f"tools/run_tests.py: {total} tests; failed folders: {', '.join(failed) or 'none'}")
    return 1 if failed or total == 0 else 0


if __name__ == "__main__":
    sys.exit(main())
