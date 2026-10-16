#!/usr/bin/env python3
"""Runs the HTTP conformance suites: the codec's (docs/stdlib/http-codec.md) and the server's
(docs/stdlib/http-server.md).

    python3 tools/http_conformance.py                       # corpus check + all codec tests, interp and native
    python3 tools/http_conformance.py --modes interp,native,wasm-interp,wasm-native
    python3 tools/http_conformance.py --jaic path/to/jaic
    python3 tools/http_conformance.py --garden ~/src/http-garden   # also look at http-garden

What it does:

1. Checks that stdlib/Extensions/Http/tests/data/llhttp_cases.jai is what tools/gen_http_corpus.py
   writes from the vendored llhttp fixtures (so a changed fixture or deviation file is not forgotten).
2. Runs every Http test (stdlib/Extensions/Http/tests/*.jai and tests/extensions/http-codec-*.jai)
   through tools/stdlib_runtime.py in the requested modes: the llhttp fixtures, the smuggling
   table, the case-by-case codec tests and the chunking tests.
3. Prints how many llhttp fixtures matched, deviate on purpose or need a lenient mode.
4. Builds the echo server (tools/http_garden/server.jai), starts it and probes it over TCP with
   hostile and awkward requests: request smuggling shapes (Content-Length with Transfer-Encoding,
   duplicate lengths, bare LF, obs-fold), oversized heads and bodies, pipelining, HEAD,
   Expect: 100-continue, keep-alive rules, and a Ctrl-C shutdown. Each probe states what a
   correct server does; the server must answer that (or close the connection).
5. External suites, only when they are available. http-garden (Docker) differentially tests whole
   HTTP servers; the echo server is the program to put in its image. Nothing here fails because
   Docker or a suite is missing.

Exit status: 0 when everything that ran passed, 1 otherwise.
"""

import argparse
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from http_native import ensure_native_libraries
from stdlib_test_catalog import HTTP_CONFORMANCE_CASE_IDS, validated_entries

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


def run_harness(jaic, modes, case_ids, jobs):
    discovered = {case["id"] for case in validated_entries()}
    missing = set(case_ids) - discovered
    if missing:
        raise RuntimeError("HTTP conformance cases missing from catalogue: " + ", ".join(sorted(missing)))
    print(f"== {len(case_ids)} exact HTTP conformance cases ({modes})")
    cmd = [sys.executable, str(ROOT / "tools" / "stdlib_runtime.py"), "--jaic", str(jaic),
           "--modes", modes, "-j", str(jobs), "--timeout", "900"]
    for name in case_ids:
        cmd.extend(["--case", name])
    return subprocess.run(cmd, cwd=ROOT).returncode == 0


def exchange(port, data, wait=1.0, shutdown_write=False):
    """Sends `data` to the server and returns everything it sends back until it closes or goes quiet."""
    sock = socket.create_connection(("127.0.0.1", port), timeout=wait)
    received = b""

    try:
        sock.sendall(data)

        if shutdown_write:
            sock.shutdown(socket.SHUT_WR)

        while True:
            chunk = sock.recv(65536)

            if not chunk:
                break

            received += chunk
    except (socket.timeout, ConnectionResetError, BrokenPipeError):
        pass
    finally:
        sock.close()

    return received


def statuses(raw):
    return [int(m.group(1)) for m in re.finditer(rb"HTTP/1\.[01] (\d{3}) ", raw)]


def check_server(jaic):
    """Builds the echo server and runs the probes against it."""
    print("== echo server probes")
    work = Path(tempfile.mkdtemp(prefix="http-conformance-"))
    binary = work / "echo_server"
    build = subprocess.run([str(jaic), str(ROOT / "tools" / "http_garden" / "server.jai"), "-exe", str(binary)],
                           capture_output=True, text=True, cwd=ROOT)

    if build.returncode != 0:
        print(build.stdout + build.stderr)

        return False

    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]

    server = subprocess.Popen([str(binary), f"127.0.0.1:{port}"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    failures = []

    def expect(name, condition, raw=b""):
        if not condition:
            failures.append(name)
            print(f"FAIL: {name}: {raw[:300]!r}")

    try:
        for _ in range(100):
            try:
                socket.create_connection(("127.0.0.1", port), timeout=0.2).close()

                break
            except OSError:
                time.sleep(0.1)

        ok_get = b"GET /a/b?x=1 HTTP/1.1\r\nHost: t\r\n\r\n"

        raw = exchange(port, ok_get, 0.5)
        expect("a plain GET is answered 200 with the request echoed", statuses(raw) == [200] and b"GET /a/b?x=1 HTTP/1.1" in raw, raw)
        expect("the Date header is present", b"Date: " in raw, raw)

        raw = exchange(port, ok_get * 3, 0.5)
        expect("three pipelined requests get three answers in order", statuses(raw) == [200, 200, 200], raw)

        raw = exchange(port, b"HEAD /h HTTP/1.1\r\nHost: t\r\n\r\n", 0.5)
        expect("HEAD gets headers and no body", statuses(raw) == [200] and raw.endswith(b"\r\n\r\n"), raw)

        raw = exchange(port, b"POST /p HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\n\r\nhello", 0.5)
        expect("a POST body is echoed", statuses(raw) == [200] and raw.endswith(b"hello"), raw)

        raw = exchange(port, b"POST /p HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n", 0.5)
        expect("a chunked body is decoded", statuses(raw) == [200] and raw.endswith(b"abcde"), raw)

        raw = exchange(port, b"POST /p HTTP/1.1\r\nHost: t\r\nContent-Length: 5\r\nExpect: 100-continue\r\n\r\nhello", 0.5)
        expect("Expect: 100-continue is answered, then the final response", statuses(raw)[:2] == [100, 200], raw)

        raw = exchange(port, b"GET / HTTP/1.0\r\n\r\n", 0.5)
        expect("HTTP/1.0 without keep-alive is answered and closed", statuses(raw) == [200], raw)

        # Smuggling shapes: every one must be refused (400) or the connection closed without a second
        # request being parsed out of the smuggled bytes.
        smuggled = b"GET /smuggled HTTP/1.1\r\nHost: t\r\n\r\n"
        cases = {
            "Content-Length and Transfer-Encoding together": b"POST / HTTP/1.1\r\nHost: t\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n" + smuggled,
            "two different Content-Length values": b"POST / HTTP/1.1\r\nHost: t\r\nContent-Length: 4\r\nContent-Length: 5\r\n\r\nabcd" + smuggled,
            "a space before the colon": b"GET / HTTP/1.1\r\nHost : t\r\n\r\n" + smuggled,
            "a bare LF in a header line": b"GET / HTTP/1.1\nHost: t\r\n\r\n" + smuggled,
            "an obsolete line fold": b"GET / HTTP/1.1\r\nHost: t\r\nX: a\r\n b\r\n\r\n" + smuggled,
            "a chunk size that is not hex": b"POST / HTTP/1.1\r\nHost: t\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n\r\n" + smuggled,
            "a Content-Length with a sign": b"POST / HTTP/1.1\r\nHost: t\r\nContent-Length: +4\r\n\r\nabcd" + smuggled,
            "a missing Host in HTTP/1.1": b"GET / HTTP/1.1\r\n\r\n" + smuggled,
            "a NUL in a header value": b"GET / HTTP/1.1\r\nHost: t\r\nX: a\x00b\r\n\r\n" + smuggled,
        }

        for name, data in cases.items():
            raw = exchange(port, data, 0.5)
            codes = statuses(raw)
            expect(f"smuggling shape refused: {name}", b"smuggled" not in raw and (not codes or codes[0] == 400), raw)

        # Limits.
        raw = exchange(port, b"GET /" + b"a" * 20000 + b" HTTP/1.1\r\nHost: t\r\n\r\n", 0.5)
        expect("a request line over the limit gets 414", statuses(raw)[:1] == [414], raw)

        raw = exchange(port, b"GET / HTTP/1.1\r\nHost: t\r\n" + b"".join(b"X-%d: v\r\n" % i for i in range(200)) + b"\r\n", 0.5)
        expect("too many headers get 431", statuses(raw)[:1] == [431], raw)

        raw = exchange(port, b"POST / HTTP/1.1\r\nHost: t\r\nContent-Length: 999999999\r\n\r\n", 0.5)
        expect("a body over the limit gets 413 before it is read", statuses(raw)[:1] == [413], raw)

        raw = exchange(port, b"BREW / HTTP/1.1\r\nHost: t\r\n\r\n", 0.5)
        expect("an unknown method is refused with 501 or 405", statuses(raw)[:1] in ([501], [405], [400]), raw)

        raw = exchange(port, b"GET / HTTP/2.5\r\nHost: t\r\n\r\n", 0.5)
        expect("an unsupported version gets 505 or 400", statuses(raw)[:1] in ([505], [400]), raw)

        raw = exchange(port, b"OPTIONS * HTTP/1.1\r\nHost: t\r\n\r\n", 0.5)
        expect("OPTIONS * is answered", len(statuses(raw)) == 1, raw)

        raw = exchange(port, b"GET / HTTP/1.1\r\nHost: t\r\n", 0.5, shutdown_write=True)
        expect("a request cut off by the client is not answered as a success", 200 not in statuses(raw), raw)

        # Graceful shutdown: SIGINT, and the process exits 0.
        server.send_signal(signal.SIGINT)

        try:
            code = server.wait(timeout=10)
        except subprocess.TimeoutExpired:
            server.kill()
            code = None

        expect("SIGINT shuts the server down with status 0", code == 0, str(code).encode())
    finally:
        if server.poll() is None:
            server.kill()

        shutil.rmtree(work, ignore_errors=True)

    print("echo server: %s" % ("ok" if not failures else f"{len(failures)} probes failed"))

    return not failures


def llhttp_summary(jaic):
    result = subprocess.run([str(jaic), str(HTTP_TESTS / "llhttp.jai"), "-interp"], capture_output=True, text=True, cwd=ROOT)
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

    print("http-garden: Docker and a checkout are present. Build tools/http_garden/server.jai into its image "
          "(docs/stdlib/http-server.md) and run its differential tests against it; that run needs the "
          "garden's own Docker setup and is not driven from here")

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
    # Configure the parent too: runtime-harness child environment changes cannot
    # supply the following echo-server/llhttp compiler processes.
    ensure_native_libraries()
    ok = check_corpus()
    ok = run_harness(jaic, args.modes, HTTP_CONFORMANCE_CASE_IDS, args.jobs) and ok
    ok = check_server(jaic) and ok
    llhttp_summary(jaic)
    ok = external_suites(args.garden) and ok

    print("\nHTTP conformance:", "ok" if ok else "FAILED")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
