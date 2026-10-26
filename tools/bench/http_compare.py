"""Compares HTTP servers under load: `benchmarks/compare/http`.

The same JSON API (`GET /items/{id}` over a 1,000-item in-memory store) is written on each server
(`servers/*.jai`), built, started, and driven by `loadgen.jai` over the loopback interface with 1, 64
and 1024 keep-alive connections. Reports requests per second, p50/p99/p99.9 latency, and the server's
memory: resident size at rest, and per connection with 1024 idle connections held open.

The community servers are third-party and unreviewed, so the build and the run of every program
(servers and load generator) go inside `bench.sandbox.Sandbox` with the loopback-only network setting:
nothing outside this machine's loopback interface is reachable, reads are limited to the system, the
jacket install, the stdlib and the upstream sources, and writes to a scratch folder.

    PYTHONPATH=tools python3 tools/bench/http_compare.py --out target/compare/http.md
    PYTHONPATH=tools python3 tools/bench/http_compare.py --servers jacket --connections 1,64
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

from bench.sandbox import environment
from bench.library_compare import ROOT, COMPARE, default_jacket, machine, native_libraries, sandbox_for, scaled

SERVERS = {
    'jacket': 'jacket.jai',
    'hyperserve': 'hyperserve.jai',
    'simplehttp': 'simplehttp.jai',
    'farzher': 'farzher.jai',
}

def build(jacket: Path, sandbox, project: Path, upstream: Path, source: str, exe: Path,
          env: dict[str, str]) -> str | None:
    """Compile `servers/source` (or the load generator) into `exe`; the compiler's output on failure."""
    command = [str(jacket), 'build', '-o', '-exe', str(exe), str(project / source),
               '-import_dir', str(upstream / '_modules'), '-import_dir', str(upstream)]
    result = subprocess.run(sandbox.wrap(command), cwd=project, env=env, capture_output=True, text=True)
    return None if result.returncode == 0 and exe.exists() else (result.stdout + result.stderr)[-3000:]


def wait_listening(port: int, process: subprocess.Popen, seconds: float = 15) -> bool:
    deadline = time.time() + seconds
    while time.time() < deadline:
        if process.poll() is not None:
            return False
        try:
            with socket.create_connection(('127.0.0.1', port), timeout=0.5):
                return True
        except OSError:
            time.sleep(0.05)
    return False


def resident_kib(pid: int) -> int:
    result = subprocess.run(['ps', '-o', 'rss=', '-p', str(pid)], capture_output=True, text=True)
    return int(result.stdout.strip() or 0)


class Sampler(threading.Thread):
    """Samples the resident size of a process until stopped. `peak` is the largest sample; `settled` the
    last one taken at least `after` seconds in (once the connections are open and quiet)."""

    def __init__(self, pid: int, after: float = 0):
        super().__init__(daemon=True)
        self.pid = pid
        self.after = after
        self.peak = 0
        self.settled = 0
        self.stop_event = threading.Event()

    def run(self):
        started = time.time()
        while not self.stop_event.is_set():
            size = resident_kib(self.pid)
            self.peak = max(self.peak, size)
            if time.time() - started >= self.after:
                self.settled = size
            self.stop_event.wait(0.2)

    def finish(self) -> 'Sampler':
        self.stop_event.set()
        self.join()
        return self


def load(sandbox, exe: Path, env: dict[str, str], port: int, connections: int, threads: int, seconds: int,
         hold_ms: int = 0, timeout: float = 120) -> dict | None:
    settings = {**env, 'BENCH_PORT': str(port), 'BENCH_CONNECTIONS': str(connections), 'BENCH_THREADS': str(threads),
                'BENCH_SECONDS': str(seconds), 'BENCH_HOLD_MS': str(hold_ms)}
    try:
        result = subprocess.run(sandbox.wrap([str(exe)]), env=settings, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None
    lines = [line for line in result.stdout.splitlines() if line.startswith('{')]
    if not lines:
        return None
    document = json.loads(lines[-1])
    document['sample'] = next((line[8:] for line in result.stderr.splitlines() if line.startswith('sample: ')), '')
    return document


class Running:
    """A server program started in the sandbox. Leaving the block stops it; `crash` says how it died
    if it died on its own."""

    def __init__(self, sandbox, exe: Path, env: dict[str, str], port: int, threads: int):
        settings = {**env, 'BENCH_PORT': str(port), 'BENCH_THREADS': str(threads)}
        self.process = subprocess.Popen(sandbox.wrap([str(exe)]), env=settings,
                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.port = port
        self.listening = wait_listening(port, self.process)
        time.sleep(0.5)
        self.crash: str | None = None

    def __enter__(self):
        return self

    def __exit__(self, *_):
        code = self.process.poll()
        if code is not None:
            self.crash = f'signal {-code}' if code < 0 else f'exit status {code}'
        else:
            self.process.terminate()
            try:
                self.process.wait(5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()


def measure(name: str, args, jacket: Path, upstream: Path, project: Path, scratch: Path,
            sandbox, env: dict[str, str], loadgen: Path, port: int) -> dict:
    """Every run starts a fresh server, so a server that dies under load costs one run, not the rest."""
    out: dict = {'name': name}
    exe = scratch / f'server-{name}'
    failure = build(jacket, sandbox, project, upstream, f'servers/{SERVERS[name]}', exe, env)
    if failure:
        out['error'] = f'does not build:\n{failure}'
        return out

    out['runs'] = {}
    out['crashes'] = {}

    for connections in args.connections:
        rounds = []
        for _ in range(args.rounds):
            with Running(sandbox, exe, env, port, args.server_threads) as server:
                if not server.listening:
                    out['error'] = 'does not start listening'
                    return out
                if 'rss_at_rest_kib' not in out:
                    out['rss_at_rest_kib'] = resident_kib(server.process.pid)
                sampler = Sampler(server.process.pid)
                sampler.start()
                result = load(sandbox, loadgen, env, port, connections, args.load_threads, args.seconds)
                peak = sampler.finish().peak
            if server.crash:
                out['crashes'].setdefault(str(connections), []).append(server.crash)
            if result is None:
                rounds.append({'error': 'the load generator did not finish'})
                continue
            result['peak_rss_kib'] = peak
            rounds.append(result)
        good = sorted((r for r in rounds if 'requests_per_second' in r), key=lambda r: r['requests_per_second'])
        out['runs'][connections] = good[len(good) // 2] if good else rounds[-1]

    # Memory per connection: 1024 connections each make a request and stay open for 4 s; the server's size
    # is read in the middle of that, against its size before the first connection.
    with Running(sandbox, exe, env, port, args.server_threads) as server:
        if server.listening:
            before = resident_kib(server.process.pid)
            sampler = Sampler(server.process.pid, after=2.5)
            sampler.start()
            held = load(sandbox, loadgen, env, port, 1024, args.load_threads, 3, hold_ms=4000)
            sampler.finish()
            out['idle'] = {'before_kib': before, 'held_kib': sampler.settled, 'served': (held or {}).get('requests', 0)}
    if server.crash:
        out['crashes'].setdefault('idle', []).append(server.crash)
    return out


def report(results: list[dict], connections: list[int]) -> str:
    lines = ['| Server | Connections | Requests/s | p50 | p99 | p99.9 | Errors |', '| --- | --- | --- | --- | --- | --- | --- |']
    for result in results:
        if 'runs' not in result:
            reason = next((line for line in result.get('error', '').splitlines() if 'error' in line), result.get('error', ''))
            lines.append(f"| {result['name']} | n/a | {reason.strip()} | | | | |")
            continue
        for count in connections:
            run = result['runs'].get(count, {})
            if 'requests_per_second' not in run:
                lines.append(f"| {result['name']} | {count} | {run.get('error', 'not run')} | | | | |")
                continue
            lines.append(f"| {result['name']} | {count} | {run['requests_per_second']:,} | {scaled(run['p50_ns'])} | "
                         f"{scaled(run['p99_ns'])} | {scaled(run['p999_ns'])} | {run['errors']} |")
    lines += ['', '| Server | Resident at rest | With 1024 idle connections | Per connection |', '| --- | --- | --- | --- |']
    for result in results:
        idle = result.get('idle')
        if not idle:
            lines.append(f"| {result['name']} | n/a | n/a | n/a |")
            continue
        rest = result['rss_at_rest_kib']
        per = (idle['held_kib'] - idle['before_kib']) / 1024
        lines.append(f"| {result['name']} | {rest / 1024:.1f} MiB | {idle['held_kib'] / 1024:.1f} MiB | {per:.1f} KiB |")
    for result in results:
        for where, how in (result.get('crashes') or {}).items():
            what = 'with 1024 idle connections' if where == 'idle' else f'under {where} connections'
            lines.append(f"\n{result['name']} died {what} in {len(how)} run(s): {', '.join(sorted(set(how)))}.")
    return '\n'.join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--jacket', default=str(default_jacket()))
    parser.add_argument('--upstream', default=str(ROOT / 'corpus' / 'upstream'))
    parser.add_argument('--servers', default='jacket,hyperserve,simplehttp,farzher')
    parser.add_argument('--connections', default='1,64,1024')
    parser.add_argument('--seconds', type=int, default=5, help='measured time of one run (after a 1 s warm-up)')
    parser.add_argument('--rounds', type=int, default=3, help='runs per connection count; the median by requests/s is reported')
    parser.add_argument('--server-threads', type=int, default=4, help='threads of the servers that have a setting')
    parser.add_argument('--load-threads', type=int, default=2, help='threads of the load generator')
    parser.add_argument('--port', type=int, default=18080)
    parser.add_argument('--json')
    parser.add_argument('--out')
    args = parser.parse_args(argv)
    args.connections = [int(c) for c in args.connections.split(',')]

    jacket = Path(args.jacket).resolve()
    upstream = Path(args.upstream).resolve()
    names = args.servers.split(',')

    with tempfile.TemporaryDirectory(prefix='http-compare-') as folder:
        scratch = Path(folder).resolve()
        project = scratch / 'project'
        shutil.copytree(COMPARE / 'http', project)

        sandbox = sandbox_for(jacket, upstream, scratch, [COMPARE / 'http'], loopback=True)
        env = native_libraries(environment(scratch))
        loadgen = scratch / 'loadgen'
        failure = build(jacket, sandbox, project, upstream, 'loadgen.jai', loadgen, env)
        if failure:
            sys.exit(f'http_compare: the load generator does not build:\n{failure}')

        results = []
        for index, name in enumerate(names):
            print(f'  {name}', file=sys.stderr)
            results.append(measure(name, args, jacket, upstream, project, scratch, sandbox, env,
                                   loadgen, args.port + index))

    if args.json:
        Path(args.json).write_text(json.dumps(results, indent=1))
    header = '\n'.join(f'- {k}: {v}' for k, v in machine().items())
    text = f'{header}\n\n{report(results, args.connections)}\n'
    if args.out:
        Path(args.out).write_text(text)
    print(text)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
