#!/usr/bin/env python3
"""Fetch the real-world inputs for `jacket bench --path stdlib/Extensions/Json`.

    PYTHONPATH=tools python3 tools/dev/json_bench_data.py

Writes target/json-bench-data, or $JACKET_OUTPUT_DIR/json-bench-data. An explicit
output directory may be passed as the first argument. Files retain their upstream
provenance: simdjson examples and nativejson-benchmark (MIT).
"""

import os
import sys
from pathlib import Path


from toolkit.download import fetch
from repo_output import output_dir

SIMDJSON = "https://raw.githubusercontent.com/simdjson/simdjson/master/jsonexamples/"
NATIVEJSON = "https://raw.githubusercontent.com/miloyip/nativejson-benchmark/master/data/"
FILES = [("twitter.json", SIMDJSON), ("canada.json", NATIVEJSON), ("citm_catalog.json", SIMDJSON)]


def fetch_data(target=None):
    target = target or output_dir("json-bench-data")
    os.makedirs(target, exist_ok=True)

    for name, base in FILES:
        path = os.path.join(target, name)

        if not os.path.exists(path):
            Path(path).write_bytes(fetch(base + name))

        print("%s: %d bytes" % (path, os.path.getsize(path)))


if __name__ == "__main__":
    fetch_data(sys.argv[1] if len(sys.argv) > 1 else None)
