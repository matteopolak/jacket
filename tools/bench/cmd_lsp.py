"""`lsp`: latency and memory of a language server over stdio, by request kind."""
from __future__ import annotations

import argparse
import shlex
import shutil
import sys
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path

from bench import common
from bench.common import Defaults
from bench.lsp_client import Client, MemoryLimit, ServerExited, Timeout, Unsupported, capabilities, file_uri
from bench.lsp_positions import delete_lines_edit, insert_edit, pick_positions
from bench.lsp_report import render
from bench.lsp_session import PROBES, SESSION_PROBES, Document, ProbeFailed, Session
from bench.lsp_workloads import WORKLOADS, Prepared
from bench.results import Report, Result, Status, checkout_info, tool_version
from bench.stats import Stat

DEFAULTS = Defaults(repeat=5, warmup=1, threshold=0.20, min_seconds=0.005, timeout=60)
LONG_SESSION_EDITS = 100
LONG_SESSION_SECONDS = 30


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--server', action='append', metavar='NAME=COMMAND', default=[],
                        help='a language server speaking LSP over stdio (repeatable; default jailsp from the build tree)')
    parser.add_argument('--diagnostics', choices=('push', 'pull'), default='push',
                        help='how the client gets diagnostics (default: %(default)s)')
    parser.add_argument('--push-wait', type=float, default=15, help='seconds to wait for a pushed diagnostics report')
    parser.add_argument('--max-rss', type=float, default=6144, help='MiB of server RSS at which a waiting request is abandoned')
    parser.add_argument('--long-edits', type=int, default=LONG_SESSION_EDITS,
                        help='edits in the long-session probe (default: %(default)s)')
    parser.add_argument('--long-seconds', type=float, default=LONG_SESSION_SECONDS,
                        help='stop the long-session probe after this many seconds (default: %(default)s)')
    common.add_common_arguments(parser, 'lsp', DEFAULTS)


# ----------------------------------------------------------------------------------- one session

@dataclass
class Outcome:
    """What one session produced for one probe."""
    status: Status = Status.OK
    reason: str = ''
    samples: list[float] = field(default_factory=list)
    cpu: float | None = None
    rss_mib: float | None = None
    extra: dict = field(default_factory=dict)


@dataclass
class SessionRecord:
    outcomes: dict[str, Outcome]
    wall: float
    cpu: float | None
    peak_rss_mib: float | None
    info: dict


class SessionRunner:
    """One fresh server and the whole script of probes. A timeout, crash or memory blow-up ends the script;
    the probes it did not reach are recorded as skipped."""

    def __init__(self, command: list[str], prepared: Prepared, args: argparse.Namespace):
        self.args = args
        self.prepared = prepared
        text = prepared.document.read_text(encoding='utf-8', errors='replace')
        self.picks = pick_positions(text)
        self.document = Document(prepared.document, text, self.picks or {}, list(prepared.also))
        self.client = Client(command, max_rss_mib=args.max_rss)
        self.session = Session(self.client, prepared.workspace, self.document, args.diagnostics == 'pull',
                               args.timeout, args.push_wait)
        self.outcomes: dict[str, Outcome] = {}
        self.alive = True
        self.dead_reason = ''

    # -------------------------------------------------------------------------------- steps

    def record(self, name: str, capability: str | None, action) -> None:
        """Run `action` (-> samples) as the probe `name`, noting CPU time and resident set around it."""
        if not self.alive:
            self.outcomes[name] = Outcome(Status.SKIPPED, self.dead_reason)
            return
        if not self.session.supports(capability):
            self.outcomes[name] = Outcome(Status.SKIPPED, f'server does not advertise {capability}')
            return
        before = self.client.sample()
        empty_before = self.session.empty_answers
        try:
            samples = action()
        except Unsupported as error:
            self.outcomes[name] = Outcome(Status.SKIPPED, str(error))
            return
        except ProbeFailed as error:
            self.outcomes[name] = Outcome(Status.FAILED, str(error))
            return
        except (Timeout, ServerExited, MemoryLimit) as error:
            self.alive = False
            self.dead_reason = f'session ended at {name}'
            self.outcomes[name] = Outcome(Status.FAILED, f'{type(error).__name__}: {error}')
            return
        after = self.client.sample()
        outcome = Outcome(samples=samples)
        outcome.extra['empty_answers'] = self.session.empty_answers - empty_before
        if before and after and samples:
            outcome.cpu = (after['cpu_user'] - before['cpu_user'] + after['cpu_sys'] - before['cpu_sys']) / len(samples)
        if after:
            outcome.rss_mib = after['rss_mib']
        self.outcomes[name] = outcome

    def start(self) -> None:
        def initialize() -> list[float]:
            number, _ = self.client.send_request('initialize', {
                'processId': None, 'clientInfo': {'name': 'jacket-bench'},
                'rootUri': file_uri(self.prepared.workspace),
                'workspaceFolders': [{'uri': file_uri(self.prepared.workspace), 'name': self.prepared.workspace.name}],
                'capabilities': capabilities(self.args.diagnostics == 'pull')})
            stamp, message = self.client.wait_response(number, self.args.timeout)
            self.session.server_capabilities = (message.get('result') or {}).get('capabilities', {})
            return [stamp - self.client.launched]

        self.record('initialize', None, initialize)
        if self.alive:
            self.client.notify('initialized')

    def index_ready(self) -> list[float]:
        """Seconds from `initialized` until workspace symbols find a procedure of the document."""
        name = self.picks['declaration']['name']
        start = time.perf_counter()
        while time.perf_counter() - start < self.args.timeout:
            _, result = self.session.request('workspace/symbol', {'query': name})
            if any(item.get('name') == name for item in result or []):
                return [time.perf_counter() - start]
            time.sleep(0.02)
        raise ProbeFailed(f'workspace symbols never found {name!r}')

    def did_open(self) -> list[float]:
        for path in self.document.also:
            self.session.open(path, path.read_text(encoding='utf-8', errors='replace'))
        seen = len(self.client.notifications)
        start = self.session.open(self.document.path, self.document.text)
        return [self.session.diagnostics_since(start, seen)]

    def file_watcher(self) -> list[float]:
        """Append a procedure to a file on disk, tell the server it changed, wait until its name is searchable."""
        path = self.prepared.watch_file
        if path is None:
            raise Unsupported('the workspace is read-only')
        original = path.read_text()
        name = f'bench_watched_{int(time.time())}'
        try:
            path.write_text(original + f'\n{name} :: () -> int {{ return 1; }}\n')
            start = self.client.notify('workspace/didChangeWatchedFiles', {'changes': [{'uri': file_uri(path), 'type': 2}]})
            while time.perf_counter() - start < self.args.timeout:
                _, result = self.session.request('workspace/symbol', {'query': name})
                if any(item.get('name') == name for item in result or []):
                    return [time.perf_counter() - start]
                time.sleep(0.02)
            raise ProbeFailed(f'{name} never became searchable')
        finally:
            path.write_text(original)

    def long_session(self) -> list[float]:
        """Many scripted edits in one session; the resident set is sampled along the way."""
        line, indent = self.picks['insert_line'], self.picks['indent']
        samples, resident = [], []
        every = max(1, self.args.long_edits // 10)
        began = time.perf_counter()
        for number in range(self.args.long_edits):
            if time.perf_counter() - began > self.args.long_seconds:
                break
            samples.append(self.session.timed_edit([insert_edit(line, 0, f'{indent}// edit {number}\n')]))
            self.session.apply([delete_lines_edit(line)])
            if number % every == 0 and (reading := self.client.sample()):
                resident.append(reading['rss_mib'])
        self.long_rss = resident
        return samples

    # ----------------------------------------------------------------------------------- run

    def run(self) -> SessionRecord:
        started = time.perf_counter()
        try:
            self.start()
            if self.alive and self.picks is None:
                self.alive = False
                self.dead_reason = 'no procedure body to pick positions in'
                self.outcomes['did_open'] = Outcome(Status.FAILED, self.dead_reason)
            self.record('index_ready', None, self.index_ready)
            self.record('did_open', None, self.did_open)
            for probe in PROBES:
                self.record(probe.name, probe.capability, lambda p=probe: p.run(self.session))
            self.record('file_watcher', None, self.file_watcher)
            self.record('long_session', None, self.long_session)
            if 'long_session' in self.outcomes and self.outcomes['long_session'].samples:
                series = getattr(self, 'long_rss', [])
                self.outcomes['long_session'].extra.update({'rss_first_mib': round(series[0], 1), 'rss_last_mib': round(series[-1], 1),
                                                       'rss_growth_mib': round(series[-1] - series[0], 1)} if series else {})
        except Exception as error:  # a bug in the harness must not lose the other workloads
            self.alive = False
            self.outcomes.setdefault('harness', Outcome(Status.FAILED, f'harness: {error!r}'))
        finally:
            peak = self.client.finish(self.alive)
        usage = self.client.rusage
        cpu = usage.ru_utime + usage.ru_stime if usage else None
        info = {'document_bytes': len(self.document.text.encode()), 'document_lines': self.document.text.count('\n') + 1}
        return SessionRecord(self.outcomes, time.perf_counter() - started, cpu, peak, info)


# ---------------------------------------------------------------------------------- aggregation

def pooled(name: str, records: list[SessionRecord]) -> Result:
    """One probe over all sessions: every session's samples together (so p50 and p95 are over all of them)."""
    outcomes = [r.outcomes[name] for r in records if name in r.outcomes]
    good = [o for o in outcomes if o.status is Status.OK]
    if not good:
        bad = next((o for o in outcomes if o.status is Status.FAILED), None) or outcomes[0]
        make = Result.failed if bad.status is Status.FAILED else Result.skipped
        return make(bad.reason)
    samples = [s for o in good for s in o.samples]
    cpus = [o.cpu for o in good if o.cpu is not None]
    rss = [o.rss_mib for o in good if o.rss_mib is not None]
    info: dict = {'samples': len(samples)}
    if len(good) < len(outcomes):
        info['failed_sessions'] = len(outcomes) - len(good)
    empty = sum(o.extra.get('empty_answers', 0) for o in good)
    if empty:
        info['empty_answers'] = empty
    for o in good:
        info.update({k: v for k, v in o.extra.items() if k != 'empty_answers'})
    return Result(wall=Stat.of(samples), cpu=Stat.of(cpus) if cpus else None, rss_mib=Stat.of(rss) if rss else None, info=info)


def session_result(records: list[SessionRecord]) -> Result:
    cpus = [r.cpu for r in records if r.cpu is not None]
    peaks = [r.peak_rss_mib for r in records if r.peak_rss_mib]
    return Result(wall=Stat.of([r.wall for r in records]), cpu=Stat.of(cpus) if cpus else None,
                  rss_mib=Stat.of(peaks) if peaks else None, info=records[0].info)


# ------------------------------------------------------------------------------------------ command

def parse_servers(specs: list[str]) -> dict[str, list[str]]:
    servers = {}
    for spec in specs or [f'jailsp={common.default_binary("jailsp")}']:
        name, separator, command = spec.partition('=')
        if not separator or not name or not command.strip():
            sys.exit(f'--server wants NAME=COMMAND, got {spec!r}')
        servers[name] = shlex.split(command)
    return servers


def server_info(name: str, command: list[str]) -> dict:
    info = {'command': command, 'version': tool_version(command)}
    if name == 'jailsp':
        info.update(checkout_info())
    return info


def chosen_workloads(only: str):
    if only:
        return [w for w in WORKLOADS if common.selected(only, w.name)]
    return [w for w in WORKLOADS if w.default]


def run(args: argparse.Namespace) -> int:
    common.check_arguments(args)
    servers = parse_servers(args.server)
    workloads = chosen_workloads(args.only)
    if args.list:
        for server in servers:
            for w in workloads:
                print(f'{server}/{w.name}')
        return 0
    for name, command in servers.items():
        if not shutil.which(command[0]) and not Path(command[0]).exists():
            sys.exit(f'{command[0]} (server {name}) not found; build it with cargo build --release -p '
                     'jai-language-server or pass --server NAME=COMMAND')
    report = Report('lsp', {'repeat': args.repeat, 'warmup': args.warmup, 'diagnostics': args.diagnostics,
                            'long_edits': args.long_edits},
                    {name: server_info(name, command) for name, command in servers.items()})
    scratch = Path(tempfile.mkdtemp(prefix='bench-lsp-'))
    try:
        for workload in workloads:
            prepared = workload.prepare(scratch)
            for server, command in servers.items():
                for _ in range(args.warmup):
                    SessionRunner(command, prepared, args).run()
                records = [SessionRunner(command, prepared, args).run() for _ in range(args.repeat)]
                prefix = f'{server}/{workload.name}'
                report.results[f'{prefix}/session'] = session_result(records)
                for name in [*SESSION_PROBES, *(p.name for p in PROBES)]:
                    report.results[f'{prefix}/{name}'] = pooled(name, records)
                p50 = report.results[f'{prefix}/did_change_keystroke'].wall
                common.print_progress(prefix, f'keystroke p50 {p50.median * 1000:.1f} ms' if p50 else 'no keystroke timing')
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    return common.finish(report, render(report), args)
