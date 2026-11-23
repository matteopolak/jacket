"""`stress`: a long seeded editing session against a language server, watching its memory and latency."""
from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

from bench import common, procmem
from bench.lsp_client import Client, FrameError, MemoryLimit, ServerExited, Timeout, capabilities, file_uri
from bench.lsp_positions import delete_lines_edit, insert_edit, pick_positions
from bench.results import ROOT
from bench.lsp_workloads import WORKLOADS
from bench.stats import percentile
from repo_output import output_dir

WORKSPACES = ('repo', 'gen-60k', 'gen-240k')
TOLERATED = {-32800, -32801, -32802}  # cancelled, content modified, server cancelled: normal for a busy editor
MAX_OPEN = 6
OS_BLOCK = '''
#if OS == .LINUX {
    stress_linux_%(n)d :: (a: int) -> int {
        linux_value := a + 1;
        return linux_value;
    }
}

#if OS == .WINDOWS {
    stress_windows_%(n)d :: (a: int) -> int {
        windows_value := a * 2;
        return windows_value;
    }
}
'''
NEW_FILE = '''stress_file_%(n)d :: (a: int) -> int {
    local := a + %(n)d;
    return local;
}
'''


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--server', default=str(common.default_binary('jailsp')),
                        help='the language server command (default: jailsp from the build tree)')
    parser.add_argument('--workspace', choices=WORKSPACES, default='repo',
                        help='repo: a temp copy of this repository\'s Jai sources; gen-*: a generated program (default: %(default)s)')
    parser.add_argument('--seed', type=int, default=1, help='the session is a function of this (default: %(default)s)')
    parser.add_argument('--steps', type=int, default=300, help='actions in the session (default: %(default)s)')
    parser.add_argument('--minutes', type=float, default=0,
                        help='run this long instead of --steps (a long-run session: 30+ minutes shows leaks)')
    parser.add_argument('--large-pause', type=float, default=2, help='seconds a large file stays open after its request (default: %(default)s)')
    parser.add_argument('--idle-every', type=int, default=50, help='an idle gap after this many actions (default: %(default)s)')
    parser.add_argument('--idle-seconds', type=float, default=8, help='length of an idle gap (default: %(default)s)')
    parser.add_argument('--settle-seconds', type=float, default=40,
                        help='longest an idle gap lasts while memory is still falling (default: %(default)s)')
    parser.add_argument('--sample-seconds', type=float, default=1, help='memory sampling interval (default: %(default)s)')
    parser.add_argument('--diagnostics-wait', type=float, default=10,
                        help='seconds a request waits for the diagnostics of the text it asks about (default: %(default)s)')
    parser.add_argument('--timeout', type=float, default=120, help='seconds before a request counts as unanswered')
    parser.add_argument('--max-rss', type=float, default=8192, help='MiB of server RSS at which the run is abandoned')
    parser.add_argument('--max-peak', type=float, default=0, help='fail when the peak private memory passes this many MiB')
    parser.add_argument('--max-slope', type=float, default=0,
                        help='fail when memory after the idle gaps grows by more than this many MiB per gap')
    parser.add_argument('--actions', default='', help='comma-separated subset of open,close,type,fix,query,os,files,large (default: all)')
    parser.add_argument('--keep', action='store_true', help='keep the temporary workspace')
    parser.add_argument('--out', type=Path, default=output_dir('bench', 'stress.json'), help='JSON results (default: %(default)s)')


# ---------------------------------------------------------------------------------------- workspace

def make_workspace(name: str, scratch: Path) -> tuple[Path, list[Path], list[Path]]:
    """A writable workspace in `scratch`, the Jai files an editor could open in it, and the large ones (over
    100 KB: binding declarations, generated data) the `large` action opens one after another."""
    if name == 'repo':
        listed = subprocess.run(['git', 'ls-files', '-z', 'stdlib', 'tests/compiler'],
                                cwd=ROOT, capture_output=True, check=True).stdout.decode().split('\0')
        root = scratch / 'repo'
        for rel in listed:
            near = rel.startswith(('stdlib/Extensions', 'stdlib/Basic', 'tests/compiler'))
            if rel.endswith(('.jai', '.toml')) and (near or (ROOT / rel).stat().st_size > 100_000):
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / rel, root / rel)
        pool = [p for p in sorted(root.glob('stdlib/Extensions/**/*.jai')) if 2_000 < p.stat().st_size < 100_000]
        large = sorted((p for p in root.glob('**/*.jai') if p.stat().st_size > 100_000), key=lambda p: str(p))
        return root, pool, large
    workload = next(w for w in WORKLOADS if w.name == name)
    prepared = workload.prepare(scratch)
    return prepared.workspace, sorted(prepared.workspace.glob('*.jai')), []


# ---------------------------------------------------------------------------------------- the model

@dataclass
class Doc:
    path: Path
    lines: list[str]
    version: int = 1
    unpublished: bool = False  # edited since the server last published its diagnostics
    bad_line: int | None = None  # a line the session broke on purpose and has not fixed yet
    os_lines: dict[str, int] = field(default_factory=dict)  # 'linux'/'windows' -> line of the body's first statement

    @property
    def text(self) -> str:
        return '\n'.join(self.lines)

    @property
    def uri(self) -> str:
        return file_uri(self.path)


# Requests about a position: kind -> (method, capability that enables it, extra parameters).
POSITIONAL = {
    'hover': ('textDocument/hover', 'hoverProvider', {}),
    'completion': ('textDocument/completion', 'completionProvider', {}),
    'definition': ('textDocument/definition', 'definitionProvider', {}),
    'references': ('textDocument/references', 'referencesProvider', {'context': {'includeDeclaration': True}}),
    'highlight': ('textDocument/documentHighlight', 'documentHighlightProvider', {}),
    'prepare_rename': ('textDocument/prepareRename', 'renameProvider', {}),
    'rename': ('textDocument/rename', 'renameProvider', {'newName': 'stress_renamed'}),
}
# Requests about a whole document: kind -> (method, capability, function of the document giving extra parameters).
WHOLE = {
    'document_symbols': ('textDocument/documentSymbol', 'documentSymbolProvider', lambda doc: {}),
    'semantic_tokens': ('textDocument/semanticTokens/full', 'semanticTokensProvider', lambda doc: {}),
    'inlay_hints': ('textDocument/inlayHint', 'inlayHintProvider', lambda doc: {
        'range': {'start': {'line': 0, 'character': 0}, 'end': {'line': min(len(doc.lines), 200), 'character': 0}}}),
    'folding': ('textDocument/foldingRange', 'foldingRangeProvider', lambda doc: {}),
}


class Stats:
    def __init__(self) -> None:
        self.latency: dict[str, list[float]] = {}
        self.tolerated = 0
        self.actions: dict[str, int] = {}

    def add(self, kind: str, seconds: float) -> None:
        self.latency.setdefault(kind, []).append(seconds)


class Failure(Exception):
    pass


class Sampler(threading.Thread):
    """Reads the server's memory every `interval` seconds until `stop` is set."""

    def __init__(self, client: Client, interval: float, limit_mib: float):
        super().__init__(daemon=True)
        self.client, self.interval, self.limit = client, interval, limit_mib
        self.series: list[tuple[float, float, float]] = []
        self.stop = threading.Event()
        self.exceeded = ''
        self.began = time.perf_counter()

    def now(self) -> tuple[float, float] | None:
        handle = int(self.client.process._handle) if sys.platform == 'win32' else None
        return procmem.read(self.client.process.pid, handle)

    def run(self) -> None:
        while not self.stop.wait(self.interval):
            if (reading := self.now()) is None:
                continue
            self.series.append((time.perf_counter() - self.began, *reading))
            if self.limit and reading[0] > self.limit:
                self.exceeded = f'resident set {reading[0]:.0f} MiB passed {self.limit:g} MiB'


class Run:
    def __init__(self, args: argparse.Namespace, workspace: Path, pool: list[Path], large: list[Path]):
        self.args = args
        self.rng = random.Random(args.seed)
        self.workspace = workspace
        self.pool = pool
        self.large = list(large)
        self.rng.shuffle(self.large)
        self.large_next = 0
        self.client = Client(common.split_command(args.server))
        self.stats = Stats()
        self.docs: dict[Path, Doc] = {}
        self.created: list[Path] = []
        self.counter = 0
        self.positional: list[str] = []
        self.whole: list[str] = []
        self.missing_diagnostics = 0
        self.gaps: list[dict] = []
        self.sampler = Sampler(self.client, args.sample_seconds, args.max_rss)

    # ------------------------------------------------------------------ plumbing

    def send(self, method: str, params: dict) -> None:
        self.client.notify(method, params)

    def ask(self, kind: str, method: str, params: dict):
        if self.sampler.exceeded:
            raise Failure(self.sampler.exceeded)
        try:
            seconds, response = self.client.request(method, params, self.args.timeout)
        except Timeout:
            raise Failure(f'{kind}: no answer within {self.args.timeout:g} s') from None
        except ServerExited as error:
            raise Failure(f'server exited during {kind}: {error}') from None
        except MemoryLimit as error:
            raise Failure(str(error)) from None
        error = response.get('error')
        if error:
            if error.get('code') in TOLERATED:
                self.stats.tolerated += 1
            else:
                raise Failure(f"{kind}: protocol error {error.get('code')}: {error.get('message')}")
        self.stats.add(kind, seconds)
        return response.get('result')

    def count(self, action: str) -> None:
        self.stats.actions[action] = self.stats.actions.get(action, 0) + 1

    # ------------------------------------------------------------------ documents

    def open_doc(self) -> None:
        candidates = [p for p in self.pool if p not in self.docs]
        if not candidates:
            return
        if len(self.docs) >= MAX_OPEN:
            self.close_doc()
        path = self.rng.choice(candidates)
        lines = path.read_text(encoding='utf-8', errors='replace').split('\n')
        doc = Doc(path, lines)
        if self.rng.random() < 0.5:
            self.counter += 1
            first = len(lines)
            lines.extend((OS_BLOCK % {'n': self.counter}).split('\n'))
            doc.os_lines = {'linux': first + 3, 'windows': first + 10}
        self.docs[path] = doc
        doc.unpublished = True
        self.send('textDocument/didOpen', {'textDocument': {'uri': doc.uri, 'languageId': 'jai', 'version': 1, 'text': doc.text}})
        self.count('open')

    def close_doc(self) -> None:
        if not self.docs:
            return
        path = self.rng.choice(list(self.docs))
        doc = self.docs.pop(path)
        self.send('textDocument/didClose', {'textDocument': {'uri': doc.uri}})
        self.count('close')

    def change(self, doc: Doc, edit: dict) -> None:
        doc.version += 1
        doc.unpublished = True
        self.send('textDocument/didChange', {'textDocument': {'uri': doc.uri, 'version': doc.version},
                                            'contentChanges': [edit]})

    def some_doc(self) -> Doc | None:
        if not self.docs:
            self.open_doc()
        return self.rng.choice(list(self.docs.values())) if self.docs else None

    def body_position(self, doc: Doc, fraction: float | None = None) -> dict | None:
        return pick_positions(doc.text, self.rng.random() if fraction is None else fraction)

    # ------------------------------------------------------------------ actions

    def type_burst(self) -> None:
        """Keystrokes into a procedure body: a comment, or a line that does not compile and is fixed later."""
        doc = self.some_doc()
        pick = doc and self.body_position(doc)
        if not pick:
            return
        self.counter += 1
        line, indent = pick['insert_line'], pick['indent']
        broken = doc.bad_line is None and self.rng.random() < 0.5
        text = f'stress_{self.counter} := missing_{self.counter} + ;' if broken else f'// stress {self.counter}'
        doc.lines.insert(line, indent)
        self.change(doc, insert_edit(line, 0, indent + '\n'))
        for column, character in enumerate(text, len(indent)):
            doc.lines[line] = doc.lines[line][:column] + character + doc.lines[line][column:]
            self.change(doc, insert_edit(line, column, character))
            if self.rng.random() < 0.2:
                time.sleep(0.01)
        if broken:
            doc.bad_line = line
        self.count('type_burst')
        self.query_in(doc)

    def fix_errors(self) -> None:
        """Delete a broken line a past burst left, a few characters at a time then the whole line."""
        for doc in self.docs.values():
            if doc.bad_line is not None:
                line = doc.bad_line
                doc.bad_line = None
                del doc.lines[line]
                self.change(doc, delete_lines_edit(line))
                self.count('fix_errors')
                self.query_in(doc)
                return

    def settle(self, doc: Doc) -> None:
        """Like an editor that asks after typing stops: usually wait for the diagnostics of `doc`, which is
        when the server has compiled the text. Without them a request is answered from syntax alone."""
        if not doc.unpublished or self.rng.random() < 0.2:
            return
        deadline = time.perf_counter() + self.args.diagnostics_wait
        client = self.client
        with client.condition:
            while True:
                found = [n for n in client.notifications if n[1].get('method') == 'textDocument/publishDiagnostics'
                         and n[1].get('params', {}).get('uri') == doc.uri]
                # Everything seen so far is read; keep the list short whatever the server publishes.
                client.notifications.clear()
                if found:
                    doc.unpublished = False
                    return
                if client.closed:
                    raise Failure(f'server exited while waiting for diagnostics: {client.tail()}')
                left = deadline - time.perf_counter()
                if left <= 0:
                    self.missing_diagnostics += 1
                    return
                client.condition.wait(min(left, 0.5))

    def request_at(self, doc: Doc, kind: str, line: int, character: int) -> None:
        method, _, extra = POSITIONAL[kind]
        params = {'textDocument': {'uri': doc.uri}, 'position': {'line': line, 'character': character}, **extra}
        self.ask(kind, method, params)
        self.count(kind)

    def query_in(self, doc: Doc) -> None:
        """A request about a random identifier of `doc`: hover, completion, definition, references, rename,
        highlights; sometimes one about the whole document (symbols, tokens, hints)."""
        self.settle(doc)
        if self.rng.random() < 0.2 and self.whole:
            kind = self.rng.choice(self.whole)
            method, _, extra = WHOLE[kind]
            self.ask(kind, method, {'textDocument': {'uri': doc.uri}, **extra(doc)})
            self.count(kind)
            return
        pick = self.body_position(doc)
        if pick:
            self.request_at(doc, self.rng.choice(self.positional), pick['hover']['line'], pick['hover']['character'])

    def query(self) -> None:
        doc = self.some_doc()
        if doc:
            self.query_in(doc)

    def os_branch(self) -> None:
        """Edit and ask about code inside the `#if OS == .LINUX` and `#if OS == .WINDOWS` blocks of a document."""
        docs = [d for d in self.docs.values() if d.os_lines]
        if not docs:
            self.open_doc()
            docs = [d for d in self.docs.values() if d.os_lines]
            if not docs:
                return
        doc = self.rng.choice(docs)
        branch = self.rng.choice(['linux', 'windows'])
        line = next(i for i, text in enumerate(doc.lines) if f'{branch}_value :=' in text)
        if self.rng.random() < 0.5:
            column = len(doc.lines[line]) - 1
            doc.lines[line] = doc.lines[line][:column] + ' ' + doc.lines[line][column:]
            self.change(doc, insert_edit(line, column, ' '))
        column = doc.lines[line].index(f'{branch}_value') + 2
        self.settle(doc)
        kinds = [k for k in self.positional if k != 'completion'] or self.positional
        self.request_at(doc, self.rng.choice(kinds), line, column)
        self.count('os_branch')

    def open_large(self) -> None:
        """Open the next large file in turn (each one once per round), ask about it, and usually close it."""
        if not self.large:
            return
        path = self.large[self.large_next % len(self.large)]
        self.large_next += 1
        if path in self.docs:
            return
        doc = Doc(path, path.read_text(encoding='utf-8', errors='replace').split('\n'), unpublished=True)
        self.docs[path] = doc
        self.send('textDocument/didOpen', {'textDocument': {'uri': doc.uri, 'languageId': 'jai', 'version': 1, 'text': doc.text}})
        self.count('open_large')
        self.query_in(doc)
        time.sleep(self.args.large_pause)
        if self.rng.random() < 0.8:
            self.docs.pop(path)
            self.send('textDocument/didClose', {'textDocument': {'uri': doc.uri}})
        elif len(self.docs) > MAX_OPEN:
            self.close_doc()

    def watch(self, *changes: tuple[Path, int]) -> None:
        self.send('workspace/didChangeWatchedFiles', {'changes': [{'uri': file_uri(p), 'type': t} for p, t in changes]})

    def file_event(self) -> None:
        """Create, rename or delete a file in the workspace (written for real), or change one on disk."""
        roll = self.rng.random()
        folder = (self.pool[0].parent if self.pool else self.workspace)
        if roll < 0.4 or not self.created:
            self.counter += 1
            path = folder / f'stress_new_{self.counter}.jai'
            path.write_text(NEW_FILE % {'n': self.counter}, encoding='utf-8')
            self.created.append(path)
            self.watch((path, 1))
            self.count('file_create')
        elif roll < 0.7:
            old = self.created.pop(self.rng.randrange(len(self.created)))
            new = old.with_name(old.stem + '_renamed.jai')
            old.rename(new)
            self.created.append(new)
            self.watch((old, 3), (new, 1))
            self.count('file_rename')
        elif roll < 0.9:
            path = self.created.pop(self.rng.randrange(len(self.created)))
            path.unlink()
            self.watch((path, 3))
            self.count('file_delete')
        else:
            path = self.rng.choice(self.created)
            path.write_text(path.read_text(encoding='utf-8') + f'// touched {self.counter}\n', encoding='utf-8')
            self.watch((path, 2))
            self.count('file_change')
        self.ask('workspace_symbol', 'workspace/symbol', {'query': 'stress_file'})

    def idle(self) -> None:
        """Do nothing for the gap, then until the server's memory has stopped moving (it gives memory back in
        steps while idle), and record where it settled and how long that took."""
        began = time.perf_counter()
        time.sleep(self.args.idle_seconds)
        recent: list[float] = []
        reading = self.sampler.now()
        while reading and time.perf_counter() - began < self.args.settle_seconds:
            recent = [*recent[-2:], reading[1]]
            if len(recent) == 3 and max(recent) - min(recent) <= 0.01 * max(recent):
                break
            time.sleep(1)
            reading = self.sampler.now()
        if reading:
            self.gaps.append({'at_action': sum(self.stats.actions.values()), 'rss_mib': round(reading[0], 1),
                              'private_mib': round(reading[1], 1), 'settled_after_s': round(time.perf_counter() - began, 1)})
        self.count('idle')

    # ------------------------------------------------------------------ the session

    def initialize(self) -> None:
        uri = file_uri(self.workspace)
        result = self.ask('initialize', 'initialize', {
            'processId': None, 'clientInfo': {'name': 'jacket-stress'}, 'rootUri': uri,
            'workspaceFolders': [{'uri': uri, 'name': self.workspace.name}], 'capabilities': capabilities(False)})
        self.client.notify('initialized')
        advertised = (result or {}).get('capabilities', {})
        self.positional = [k for k, (_, cap, _) in POSITIONAL.items() if advertised.get(cap)]
        self.whole = [k for k, (_, cap, _) in WHOLE.items() if advertised.get(cap)]

    def run(self) -> None:
        self.sampler.start()
        self.initialize()
        self.open_doc()
        every = {'open': (self.open_doc, 3), 'close': (self.close_doc, 2), 'type': (self.type_burst, 8),
                 'fix': (self.fix_errors, 4), 'query': (self.query, 8), 'os': (self.os_branch, 4),
                 'files': (self.file_event, 4), 'large': (self.open_large, 3)}
        chosen = [n.strip() for n in self.args.actions.split(',') if n.strip()] or list(every)
        unknown = [n for n in chosen if n not in every]
        if unknown:
            raise Failure(f'--actions: unknown {unknown}; choose from {list(every)}')
        if not self.large:
            chosen = [n for n in chosen if n != 'large']
        actions = [every[n] for n in chosen]
        weights = [w for _, w in actions]
        ends = time.perf_counter() + self.args.minutes * 60
        step = 0
        while step < self.args.steps if not self.args.minutes else time.perf_counter() < ends:
            step += 1
            self.rng.choices([a for a, _ in actions], weights)[0]()
            if step % self.args.idle_every == 0:
                self.idle()
        self.idle()

    def finish(self, failure: str | None) -> dict:
        self.sampler.stop.set()
        self.sampler.join(5)
        try:
            if failure is None:
                self.client.request('shutdown', None, 10)
                self.client.notify('exit')
        except (Timeout, ServerExited, MemoryLimit):
            pass
        peak = self.client.finish(False)
        crashed = self.client.process.returncode not in (0, None, -9, -15) and failure is None
        stderr = self.client.tail()
        if crashed or 'panicked' in stderr:
            failure = failure or f'server exited {self.client.process.returncode}: {stderr}'
        return summarize(self, failure, peak)


def percentile_ms(samples: list[float], q: float) -> float:
    return round(percentile(sorted(samples), q) * 1000, 1) if samples else 0.0


def slope(points: list[tuple[float, float]]) -> float:
    """Least-squares slope of y per x."""
    n = len(points)
    if n < 2:
        return 0.0
    mx = sum(x for x, _ in points) / n
    my = sum(y for _, y in points) / n
    denominator = sum((x - mx) ** 2 for x, _ in points)
    return sum((x - mx) * (y - my) for x, y in points) / denominator if denominator else 0.0


def summarize(run: Run, failure: str | None, peak_rss: float | None) -> dict:
    series = run.sampler.series
    gaps = run.gaps
    private = [g['private_mib'] for g in gaps]
    every = [s for kind in run.stats.latency.values() for s in kind]
    result = {
        'failure': failure,
        'seed': run.args.seed, 'steps': run.args.steps, 'workspace': run.args.workspace,
        'actions': run.stats.actions,
        'tolerated_cancellations': run.stats.tolerated,
        'missing_diagnostics': run.missing_diagnostics,
        'peak_rss_mib': round(max([r for _, r, _ in series] + [peak_rss or 0]), 1),
        'peak_private_mib': round(max([p for _, _, p in series] + [0]), 1),
        'after_idle': gaps,
        'slope_per_minute_mib': round(slope([(t / 60, p) for t, _, p in series]), 1),
        # The first gap is the server warming up; the trend is judged from the gaps after it.
        'slope_second_half_per_minute_mib': round(slope([(t / 60, p) for t, _, p in series[len(series) // 2:]]), 2),
        'slope_per_gap_mib': round(slope(list(enumerate(private[1:] if len(private) > 3 else private))), 1),
        'latency_ms': {kind: {'n': len(s), 'p50': percentile_ms(s, 0.5), 'p99': percentile_ms(s, 0.99)}
                       for kind, s in sorted(run.stats.latency.items())},
        'latency_all_ms': {'n': len(every), 'p50': percentile_ms(every, 0.5), 'p99': percentile_ms(every, 0.99)},
        'samples': [[round(t, 1), round(r, 1), round(p, 1)] for t, r, p in series],
    }
    if not failure and run.args.max_peak and result['peak_private_mib'] > run.args.max_peak:
        result['failure'] = f"peak private memory {result['peak_private_mib']} MiB passed {run.args.max_peak:g} MiB"
    if not result['failure'] and run.args.max_slope and result['slope_per_gap_mib'] > run.args.max_slope:
        result['failure'] = f"memory after idle gaps grows {result['slope_per_gap_mib']} MiB per gap"
    return result


def render(result: dict) -> str:
    lines = [f"# Language server stress session (seed {result['seed']}, {result['steps']} steps, {result['workspace']})", '']
    lines.append(f"- result: {'FAILED: ' + result['failure'] if result['failure'] else 'ok'}")
    lines.append(f"- peak: {result['peak_rss_mib']} MiB resident, {result['peak_private_mib']} MiB private")
    lines.append(f"- slope: {result['slope_per_minute_mib']} MiB/min over the run ({result['slope_second_half_per_minute_mib']} in its second half), {result['slope_per_gap_mib']} MiB per idle gap")
    lines.append(f"- actions: {', '.join(f'{k} {v}' for k, v in sorted(result['actions'].items()))}")
    lines += ['', 'After each idle gap (MiB private): ' + ', '.join(str(g['private_mib']) for g in result['after_idle']), '']
    rows = [[kind, str(v['n']), f"{v['p50']:.1f}", f"{v['p99']:.1f}"] for kind, v in result['latency_ms'].items()]
    a = result['latency_all_ms']
    rows.append(['all', str(a['n']), f"{a['p50']:.1f}", f"{a['p99']:.1f}"])
    lines.append(common.table(['request', 'n', 'p50 ms', 'p99 ms'], rows))
    return '\n'.join(lines) + '\n'


def run(args: argparse.Namespace) -> int:
    command = common.split_command(args.server)
    if not shutil.which(command[0]) and not Path(command[0]).exists():
        sys.exit(f'{command[0]} not found; build it with cargo build --release -p jai-language-server or pass --server')
    os.environ.setdefault('JAIC_STDLIB', str(ROOT / 'stdlib'))
    scratch = Path(tempfile.mkdtemp(prefix='bench-stress-'))
    try:
        workspace, pool, large = make_workspace(args.workspace, scratch)
        session = Run(args, workspace, pool, large)
        failure = None
        try:
            session.run()
        except (Failure, FrameError, ServerExited, Timeout, MemoryLimit) as error:
            failure = str(error) or type(error).__name__
        result = session.finish(failure)
    finally:
        if args.keep:
            print(f'workspace kept at {scratch}', file=sys.stderr)
        else:
            shutil.rmtree(scratch, ignore_errors=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=1), encoding='utf-8')
    print(render(result))
    print(f'wrote {args.out}', file=sys.stderr)
    return 1 if result['failure'] else 0
