#!/usr/bin/env python3
"""Runs the HTTP codec's conformance suites (docs/stdlib/http-codec.md).

    python3 tools/http_conformance.py                       # corpus check + all codec tests, interp and native
    python3 tools/http_conformance.py --modes interp,native,wasm-interp,wasm-native
    python3 tools/http_conformance.py --jaic path/to/jaic
    python3 tools/http_conformance.py --garden ~/src/http-garden   # also look at http-garden

What it does:

1. Checks that stdlib/Extensions/Http/tests/data/llhttp_cases.jai is what tools/gen_http_corpus.py
   writes from the vendored llhttp fixtures (so a changed fixture or deviation file is not forgotten).
2. Runs every Http test (stdlib/Extensions/Http/tests/*.jai and tests/stdlib/http-codec-*.jai)
   through tools/stdlib_runtime.py in the requested modes: the llhttp fixtures, the smuggling
   table, the case-by-case codec tests and the chunking tests.
3. Prints how many llhttp fixtures matched, deviate on purpose or need a lenient mode.
4. External suites, only when they are available. http-garden (Docker) differentially tests whole
   HTTP servers, so it applies once the jaic HTTP server exists; until then it is reported as not
   applicable. Nothing here fails because Docker or a suite is missing.

Exit status: 0 when everything that ran passed, 1 otherwise.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HTTP_TESTS = ROOT / "stdlib" / "Extensions" / "Http" / "tests"


def find_jaic(explicit):
    if explicit:
        return Path(explicit)

    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

    for profile in ("release", "debug"):
        candidate = target / profile / "jaic"

        if candidate.exists():
            return candidate

    found = shutil.which("jaic")

    if found:
        return Path(found)

    sys.exit("jaic not found: build it (cargo build --release -p jaic-cli) or pass --jaic")


def check_corpus():
    print("== llhttp corpus is current")
    out = ROOT / "stdlib" / "Extensions" / "Http" / "tests" / "data" / "llhttp_cases.jai"
    before = out.read_bytes() if out.exists() else b""
    result = subprocess.run([sys.executable, str(ROOT / "tools" / "gen_http_corpus.py")], capture_output=True, text=True)

    if result.returncode != 0:
        print(result.stdout + result.stderr)

        return False

    after = out.read_bytes()

    if before != after:
        out.write_bytes(before)
        print("FAIL: llhttp_cases.jai is out of date: run python3 tools/gen_http_corpus.py and commit it")

        return False

    print(result.stdout.strip())

    return True


def run_harness(jaic, modes, filters, jobs):
    ok = True

    for text in filters:
        print(f"== stdlib runtime tests matching '{text}' ({modes})")
        cmd = [sys.executable, str(ROOT / "tools" / "stdlib_runtime.py"), "--jaic", str(jaic), "--filter", text,
               "--modes", modes, "-j", str(jobs), "--timeout", "900"]
        result = subprocess.run(cmd, cwd=ROOT)
        ok = ok and result.returncode == 0

    return ok


def llhttp_summary(jaic):
    result = subprocess.run([str(jaic), "run", str(HTTP_TESTS / "llhttp.jai")], capture_output=True, text=True, cwd=ROOT)
    match = re.search(r"(\d+) cases: (\d+) match llhttp, (\d+) deviate on purpose, (\d+) need a lenient mode", result.stdout)

    if match:
        print("== llhttp fixtures: %s run, %s match, %s deviate on purpose (tools/http_deviations.json), %s need a lenient mode llhttp has and the codec does not" % match.groups())
    else:
        print(result.stdout + result.stderr)


def external_suites(garden):
    print("== external suites")
    docker = shutil.which("docker")

    if garden is None:
        print("http-garden: not requested (--garden DIR with a checkout of narfindustries/http-garden)")

        return True

    if not docker:
        print("http-garden: skipped, Docker is not installed")

        return True

    if subprocess.run([docker, "info"], capture_output=True).returncode != 0:
        print("http-garden: skipped, the Docker daemon is not running")

        return True

    if not (Path(garden) / "README.md").exists():
        print(f"http-garden: skipped, {garden} is not a checkout")

        return True

    print("http-garden: Docker and a checkout are present, but http-garden tests whole HTTP servers and the "
          "jaic server arrives with the next milestone (docs/plans/async-io.md); not applicable to the codec alone")

    return True


def main():
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--jaic", help="compiler to test (default: $CARGO_TARGET_DIR/release/jaic, then debug)")
    ap.add_argument("--modes", default="interp,native", help="stdlib_runtime.py modes (default interp,native)")
    ap.add_argument("--garden", help="a checkout of http-garden (needs Docker)")
    ap.add_argument("-j", "--jobs", type=int, default=2)
    args = ap.parse_args()

    jaic = find_jaic(args.jaic)
    ok = check_corpus()
    ok = run_harness(jaic, args.modes, ["Extensions/Http/tests", "http-codec"], args.jobs) and ok
    llhttp_summary(jaic)
    ok = external_suites(args.garden) and ok

    print("\nHTTP conformance:", "ok" if ok else "FAILED")
    sys.exit(0 if ok else 1)


main()
