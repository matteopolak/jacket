#!/usr/bin/env python3
"""Merges the JUnit files of `jacket test` runs into the table `--partition time:K/N` balances by.

CI's stdlib `jacket test` steps write one JUnit file per mode when JACKET_TEST_TIMES_DIR is set
(tools/ci/stdlib_jacket_test.sh) and upload them as `jacket-test-times-*` artifacts. To refresh
the table after the tests change much:

    gh run download RUN --pattern 'jacket-test-times-*' --dir "$TMPDIR/times"
    python3 tools/ci/test_times.py "$TMPDIR/times" > tools/ci/test-times.json

A test's entry is its seconds summed over every file it appears in (the modes and platforms run),
so a test that is slow in any mode weighs more. Skipped tests are left out.
"""

import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


def totals(paths):
    """{test name: seconds} summed over the JUnit files at `paths`."""
    times = {}
    for path in paths:
        for case in ET.parse(path).iter("testcase"):
            if case.find("skipped") is not None:
                continue
            name = case.get("name")
            times[name] = times.get(name, 0.0) + float(case.get("time", 0))
    return {name: round(times[name], 3) for name in sorted(times)}


def main(argv):
    if len(argv) < 2:
        sys.exit("usage: test_times.py DIR_OR_FILE...")
    paths = []
    for arg in argv[1:]:
        path = Path(arg)
        paths += sorted(path.rglob("*.xml")) if path.is_dir() else [path]
    if not paths:
        sys.exit("test_times: no JUnit files")
    json.dump(totals(paths), sys.stdout, indent=0)
    print()


if __name__ == "__main__":
    main(sys.argv)
