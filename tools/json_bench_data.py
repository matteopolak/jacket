#!/usr/bin/env python3
"""Downloads the JSON files the Json benchmark is usually run on (simdjson's jsonexamples and
the nativejson-benchmark data) into
a directory, and prints the command to run:

    python3 tools/json_bench_data.py /tmp/json-data
    jaic stdlib/Extensions/Json/bench/bench.jai -O2 -exe /tmp/json-bench
    /tmp/json-bench /tmp/json-data/twitter.json /tmp/json-data/canada.json /tmp/json-data/citm_catalog.json

The files are not kept in the repository: they are several MB, and each has its own provenance
(twitter.json is a sample of public tweets, canada.json is geographic data, citm_catalog.json
comes from the nativejson-benchmark, MIT). simdjson itself is Apache-2.0 / MIT.
"""

import os
import sys
import urllib.request

SIMDJSON = "https://raw.githubusercontent.com/simdjson/simdjson/master/jsonexamples/"
NATIVEJSON = "https://raw.githubusercontent.com/miloyip/nativejson-benchmark/master/data/"
FILES = [("twitter.json", SIMDJSON), ("canada.json", NATIVEJSON), ("citm_catalog.json", SIMDJSON)]


def main():
    target = sys.argv[1] if len(sys.argv) > 1 else "json-data"
    os.makedirs(target, exist_ok=True)

    for name, base in FILES:
        path = os.path.join(target, name)

        if not os.path.exists(path):
            with urllib.request.urlopen(base + name) as response, open(path, "wb") as out:
                out.write(response.read())

        print("%s: %d bytes" % (path, os.path.getsize(path)))


main()
