#!/usr/bin/env python3
"""List the stdlib tests a change can affect, for the `jacket test` CI steps.

`jacket test --changed FILE` keeps the tests whose programs read FILE (what `jaic -deps` says the
compiles read), so the dependency graph is the compiler's, not a table kept here. This script
asks for each mode's selected-target catalogue and writes its exact filters. A host-only empty
selection cannot suppress cases compiled only for another target.

    JACKET_TEST_CHANGED='["stdlib/Sort/module.jai"]' python3 tools/ci/select_tests.py --jacket target/release/jacket --out names.txt

Exit status 0: `--out` holds the names, one per line (empty: no test is affected). Exit status 3:
run every test (no changed files are given, or jacket could not say). Nothing here is allowed to
skip tests on a doubt.
"""

import argparse
import json
import os
import subprocess
import sys

EVERYTHING = 3


def test_names(listing):
    """The names of the tests in a `jacket test --list --format json` output: subtests belong to
    their test, and benchmarks are not run by `jacket test`. `None` if a line is not JSON."""
    names = []
    for line in listing.splitlines():
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except ValueError:
            return None
        if record.get("type") == "list" and record.get("kind") not in (
            "subtest",
            "bench",
        ):
            names.append(record["name"])
    return names


def affected(jacket, changed, run=subprocess.run, flags=()):
    """The names of the tests `changed` (repo-relative paths) can affect, or `None` to run all."""
    if not changed:
        return None
    command = [jacket, "test", "--list", "--list-target", "--format", "json"]
    for path in changed:
        command += ["--changed", path]
    command += list(flags)
    result = run(command, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"select_tests: jacket failed:\n{result.stderr}", file=sys.stderr)
        return None
    sys.stderr.write(
        "".join(
            line + "\n"
            for line in result.stderr.splitlines()
            if "can be affected" in line
        )
    )
    return test_names(result.stdout)


def main(argv, env):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jacket", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("flags", nargs=argparse.REMAINDER, help="after --, the exact arguments of this jacket mode")
    args = parser.parse_args(argv)
    flags = args.flags[1:] if args.flags[:1] == ["--"] else args.flags
    try:
        changed = json.loads(env.get("JACKET_TEST_CHANGED") or "[]")
    except ValueError:
        changed = []
    names = affected(args.jacket, changed, flags=flags)
    if names is None:
        print("select_tests: running every test", file=sys.stderr)
        return EVERYTHING
    with open(args.out, "w") as out:
        out.write("".join(name + "\n" for name in names))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:], os.environ))
