#!/usr/bin/env python3
"""Measure the latency and memory of the Jai language server (jailsp) on real and generated projects.

    python3 tools/lsp_bench.py [--jailsp PATH] [--repeat 5] [--only TEXT] [--diagnostics pull,push]
                               [--timeout 60] [--out results.json] [--markdown results.md]
                               [--compare old.json] [--threshold 0.20]

Each workload starts a fresh jailsp over stdio, sends `initialize` (with a workspace folder and
realistic client capabilities), `initialized` and `didOpen`, then times a fixed script of requests:
first diagnostics, hover (first, repeated, after an edit), completion while typing a word and after
a `.`, references, document symbols and semantic tokens. One such session is one run; the first run
is reported as "cold" (the OS file cache is not primed, the server's caches are empty anyway, as
every session is a new process) and the median of the rest as "warm". Peak RSS is the server's own
maximum resident set, from wait4.

A request that gets no answer within --timeout is recorded as `timeout` (and the session ends), one that makes
the server pass --max-rss MiB as `memory`; an
error answer is `error`; a server that exits, or a document the server refuses because of its size
limits, is `limit`/`crash`. None of these stops the benchmark.

--compare flags workloads whose warm time for a metric, or peak RSS, grew by more than --threshold (and
by more than --min-seconds / --min-mib) and exits 1 when any did or when a workload failed.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import queue
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def data_root() -> Path:
    # Worktrees share the gitignored corpus/upstream with the main checkout.
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    return Path(common.stdout.strip()).parent if common.returncode == 0 else ROOT


UPSTREAM = data_root() / 'corpus' / 'upstream'

# jailsp's default `Limits` (crates/jai-language-server/src/lib.rs): used to label oversize documents.
DOCUMENT_BYTES = 32 * 1024 * 1024
MESSAGE_BYTES = 64 * 1024 * 1024

# name -> (label shown in tables, true when the metric is reported in ms)
METRICS = {
    'startup': 'launch to initialize response',
    'first_diagnostics': 'didOpen to first diagnostics',
    'hover_first': 'first hover',
    'hover_warm': 'hover again, no edit (median of 5)',
    'completion_member': 'completion after `.` (existing text)',
    'references': 'references of the procedure',
    'document_symbols': 'document symbols',
    'semantic_tokens': 'semantic tokens (full)',
    'edit_hover': 'edit, then hover (median of 3)',
    'typing_first': 'typing: first character completion',
    'typing_median': 'typing: median completion',
    'completion_member_typed': 'edit typing `name.`, then completion',
}

# workload name -> how to open it. Corpus entries: workspace folder under corpus/upstream, the document to
# measure, and other documents opened first (the project's root file, as an editor session would have).
WORKLOADS: dict[str, dict] = {
    'focus': {'project': 'focus-editor--focus', 'document': 'src/editors.jai', 'also': ['first.jai']},
    # The same document with the program's own root file open, so it is checked from `src/main.jai` and not
    # from the build metaprogram `first.jai`.
    'focus-main': {'project': 'focus-editor--focus', 'document': 'src/editors.jai', 'also': ['src/main.jai']},
    'jails': {'project': 'SogoCZE--Jails', 'document': 'server/program.jai', 'also': ['server/main.jai']},
    'chess-jai': {'project': 'danieltan1517--chess-jai', 'document': 'movegen.jai', 'also': ['build.jai']},
    # Generated (tools/benchgen.py): (lines, seed, files). main.jai loads the parts and is open too. The part in
    # the middle is the document.
    'gen-60k': {'generated': (60_000, 1, 12)},
    'gen-240k': {'generated': (240_000, 1, 48)},
    # Single files (benchgen with files=1): about 0.8 MB and 3.3 MB, the large-document cases.
    'large-25k': {'generated': (25_000, 1, 1)},
    'large-100k': {'generated': (100_000, 1, 1)},
}

KEYWORDS = frozenset('''if else then for while return break continue case switch struct enum union cast defer
using inline no_inline true false null new delete push_context remove xx it it_index context size_of type_of
type_info is_constant else if ifx operator interface'''.split())


# ---------------------------------------------------------------------------------------- framing

class FrameError(ValueError):
    pass


def encode(message: dict) -> bytes:
    """One LSP frame: `Content-Length` header, blank line, compact JSON body."""
    body = json.dumps(message, separators=(',', ':'), ensure_ascii=False).encode('utf-8')
    return b'Content-Length: %d\r\n\r\n' % len(body) + body


class FrameDecoder:
    """Incremental decoder: feed it arbitrary byte chunks, get complete JSON bodies back."""

    def __init__(self) -> None:
        self.buffer = b''

    def push(self, data: bytes) -> list[dict]:
        self.buffer += data
        messages = []
        while True:
            end = self.buffer.find(b'\r\n\r\n')
            if end < 0:
                return messages
            length = None
            for line in self.buffer[:end].decode('ascii', errors='replace').split('\r\n'):
                name, _, value = line.partition(':')
                if name.strip().lower() == 'content-length':
                    try:
                        length = int(value.strip())
                    except ValueError:
                        raise FrameError(f'bad Content-Length {value.strip()!r}') from None
            if length is None:
                raise FrameError('header without Content-Length')
            start = end + 4
            if len(self.buffer) < start + length:
                return messages
            messages.append(json.loads(self.buffer[start:start + length].decode('utf-8')))
            self.buffer = self.buffer[start + length:]


# ------------------------------------------------------------------------------ position picking

def mask_line(line: str, block_depth: int = 0) -> tuple[str, int]:
    """`line` with comments and string contents replaced by spaces (same length), and the block comment depth
    left open at its end. Good enough for choosing positions; it does not try to be a lexer."""
    out = []
    i = 0
    in_string = False
    while i < len(line):
        two = line[i:i + 2]
        if block_depth:
            if two == '/*':
                block_depth += 1
                out.append('  ')
                i += 2
            elif two == '*/':
                block_depth -= 1
                out.append('  ')
                i += 2
            else:
                out.append(' ')
                i += 1
        elif in_string:
            if line[i] == '\\':
                out.append('  ')
                i += 2
                continue
            if line[i] == '"':
                in_string = False
                out.append('"')
            else:
                out.append(' ')
            i += 1
        elif two == '//':
            out.append(' ' * (len(line) - i))
            break
        elif two == '/*':
            block_depth = 1
            out.append('  ')
            i += 2
        elif line[i] == '"':
            in_string = True
            out.append('"')
            i += 1
        else:
            out.append(line[i])
            i += 1
    return ''.join(out), block_depth


PROC_START = re.compile(r'^([A-Za-z_]\w*)\s*::\s*\(.*\{\s*$')
IDENT = re.compile(r'(?<![\w.#$@])[A-Za-z_]\w*')
MEMBER = re.compile(r'(?<![\w.$])([A-Za-z_]\w*)\.([A-Za-z_]\w*)')


def utf16_col(line: str, col: int) -> int:
    """LSP column (UTF-16 code units) of the character index `col` in `line`."""
    return len(line[:col].encode('utf-16-le')) // 2


def procedure_bodies(masked: list[str]) -> list[tuple[int, int, str, int]]:
    """(first line, closing line, name, name column) of each top-level procedure whose `{` ends its header
    line and whose closing `}` is alone at column 0."""
    bodies = []
    start = None
    for number, line in enumerate(masked):
        if start is None:
            match = PROC_START.match(line)
            if match:
                start = (number, match.group(1))
        elif line.rstrip() in ('}', '};'):
            bodies.append((start[0], number, start[1], 0))
            start = None
        elif PROC_START.match(line) is None and line[:1] not in (' ', '\t', '') and line.strip():
            # A new top-level declaration before any closer: the header was not a body.
            start = None
            match = PROC_START.match(line)
            if match:
                start = (number, match.group(1))
    return bodies


def pick_positions(text: str, fraction: float = 0.6) -> dict | None:
    """Deterministic positions inside procedure bodies, `fraction` of the way through the candidates:

    hover: an identifier that is not a keyword, a member or a directive (line, utf-16 column, name);
    insert_line/indent: where to add a line inside the same body (the line after the hover's);
    declaration: the name of the procedure that body belongs to (for references);
    member: a `base.field` use inside a body, column just after the dot (None when there is none).
    """
    lines = text.split('\n')
    masked = []
    depth = 0
    for line in lines:
        m, depth = mask_line(line.rstrip('\r'), depth)
        masked.append(m)
    bodies = procedure_bodies(masked)
    identifiers = []
    members = []
    for first, last, name, _ in bodies:
        for number in range(first + 1, last):
            line = masked[number]
            for m in IDENT.finditer(line):
                word = m.group(0)
                if len(word) >= 3 and word not in KEYWORDS and not line[m.end():].lstrip().startswith('::'):
                    identifiers.append((number, m.start(), word, (first, last, name)))
            for m in MEMBER.finditer(line):
                if m.group(1) not in KEYWORDS:
                    members.append((number, m.start(2), m.group(1), (first, last, name)))
    if not identifiers:
        return None
    number, col, word, body = identifiers[min(len(identifiers) - 1, int(len(identifiers) * fraction))]
    raw = lines[number].rstrip('\r')
    indent = raw[:len(raw) - len(raw.lstrip(' \t'))]
    pick = {
        'hover': {'line': number, 'character': utf16_col(raw, col), 'name': word},
        'insert_line': number + 1,
        'indent': indent,
        'declaration': {'line': body[0], 'character': 0, 'name': body[2]},
        'member': None,
    }
    if members:
        mnumber, mcol, base, _ = members[min(len(members) - 1, int(len(members) * (fraction - 0.1)))]
        pick['member'] = {'line': mnumber, 'character': utf16_col(lines[mnumber].rstrip('\r'), mcol), 'base': base}
    return pick


def insert_edit(line: int, character: int, text: str) -> dict:
    """An incremental `contentChanges` entry that inserts `text` at a position."""
    return {'range': {'start': {'line': line, 'character': character},
                      'end': {'line': line, 'character': character}}, 'text': text}


def delete_lines_edit(first: int, count: int = 1) -> dict:
    return {'range': {'start': {'line': first, 'character': 0}, 'end': {'line': first + count, 'character': 0}},
            'text': ''}


def typing_script(indent: str, word: str) -> list[tuple[int, str]]:
    """What typing `word` one character at a time inserts: (column to insert at, character), after the
    indentation is already in place."""
    return [(len(indent) + i, ch) for i, ch in enumerate(word)]


# ------------------------------------------------------------------------------------- statistics

def parse_cpu_time(text: str) -> float:
    """`ps` CPU time: [DD-][HH:]MM:SS[.cc] or M:SS.cc, in seconds."""
    days, _, rest = text.rpartition('-')
    seconds = 0.0
    for part in rest.split(':'):
        seconds = seconds * 60 + float(part)
    return seconds + (int(days) * 86400 if days else 0)


def median(values: list[float]) -> float:
    return statistics.median(values)


def cpu_of(run: dict) -> float | None:
    return round(run['cpu_user'] + run['cpu_sys'], 4) if 'cpu_user' in run and 'cpu_sys' in run else None


def warm_median(values: list) -> float | None:
    """Median of the values after the first (of all when there is only one); None entries are skipped."""
    rest = [v for v in values[1:] if v is not None] or [v for v in values if v is not None]
    return round(median(rest), 5) if rest else None


def summarize_metric(runs: list[dict]) -> dict:
    """One metric over the sessions. Each run is {'seconds': x} or {'status': s, 'message': m}, with the
    server's CPU seconds and RSS around the step when known. The first session is cold, the median of the
    rest warm (of all when there is only one)."""
    ok = [r['seconds'] if 'seconds' in r else None for r in runs]
    statuses = list(dict.fromkeys(r['status'] for r in runs if 'status' in r))
    numbers = [v for v in ok if v is not None]
    cpu = [cpu_of(r) for r in runs]
    return {
        'cold': round(ok[0], 5) if ok[0] is not None else None,
        'warm': warm_median(ok),
        'min': round(min(numbers), 5) if numbers else None,
        'max': round(max(numbers), 5) if numbers else None,
        'runs': [round(v, 5) if v is not None else None for v in ok],
        'cpu_cold': cpu[0],
        'cpu_warm': warm_median(cpu),
        'rss_cold': runs[0].get('rss_mib'),
        'rss_warm': warm_median([r.get('rss_mib') for r in runs]),
        'status': 'ok' if not statuses else '/'.join(statuses),
        'message': next((r['message'] for r in runs if r.get('message')), ''),
    }


def summarize(sessions: list[dict]) -> dict:
    names = list(dict.fromkeys(name for s in sessions for name in s['metrics']))
    metrics = {name: summarize_metric([s['metrics'].get(name, {'status': 'skipped'}) for s in sessions])
               for name in names}
    rss = [s['peak_rss_mib'] for s in sessions if s.get('peak_rss_mib')]
    cpu = [(s['cpu_user_s'] + s['cpu_sys_s']) if s.get('cpu_user_s') is not None else None for s in sessions]
    user = [s.get('cpu_user_s') for s in sessions]
    system = [s.get('cpu_sys_s') for s in sessions]
    return {
        'metrics': metrics,
        'peak_rss_mib': round(max(rss), 1) if rss else None,
        'warm_rss_mib': warm_median(rss),
        'cpu_seconds_cold': round(cpu[0], 3) if cpu and cpu[0] is not None else None,
        'cpu_seconds': warm_median(cpu),
        'cpu_user_seconds': warm_median(user),
        'cpu_sys_seconds': warm_median(system),
        'info': sessions[0].get('info', {}),
    }


def compare(old: dict, new: dict, threshold: float, min_seconds: float, min_mib: float) -> list[str]:
    """Lines describing every regression of `new` against `old` (same key and metric only): warm wall time,
    warm CPU time and RSS after the step per metric, and total CPU time and peak RSS per workload. A metric
    that was a number and is now a failure counts too (unsupported and skipped ones do not)."""
    problems = []

    def grew(a, b, floor):
        return a and b is not None and b > a * (1 + threshold) and b - a > floor

    for key, row in new.get('results', {}).items():
        before = old.get('results', {}).get(key)
        if not before:
            continue
        for name, metric in row['metrics'].items():
            was = before['metrics'].get(name)
            if not was or was.get('warm') is None:
                continue
            if metric.get('warm') is None:
                if metric['status'] not in ('unsupported', 'skipped'):
                    problems.append(f'{key}: {name} {was["warm"] * 1000:g} ms -> {metric["status"]}')
                continue
            a, b = was['warm'], metric['warm']
            if grew(a, b, min_seconds):
                problems.append(f'{key}: {name} {a * 1000:g} -> {b * 1000:g} ms (+{(b / a - 1) * 100:.0f}%)')
            a, b = was.get('cpu_warm'), metric.get('cpu_warm')
            if grew(a, b, max(min_seconds, 0.02)):
                problems.append(f'{key}: {name} cpu {a * 1000:g} -> {b * 1000:g} ms (+{(b / a - 1) * 100:.0f}%)')
            a, b = was.get('rss_warm'), metric.get('rss_warm')
            if grew(a, b, min_mib):
                problems.append(f'{key}: {name} rss {a:g} -> {b:g} MiB (+{(b / a - 1) * 100:.0f}%)')
        for field, unit, floor in (('cpu_seconds', 's', max(min_seconds, 0.02)), ('peak_rss_mib', 'MiB', min_mib)):
            a, b = before.get(field), row.get(field)
            if grew(a, b, floor):
                problems.append(f'{key}: {field} {a:g} -> {b:g} {unit} (+{(b / a - 1) * 100:.0f}%)')
    return problems


# ---------------------------------------------------------------------------------------- client

class Timeout(Exception):
    pass


class ServerExited(Exception):
    pass


class MemoryLimit(Exception):
    pass


class Unsupported(Exception):
    pass


CLIENT_CAPABILITIES = {
    'general': {'positionEncodings': ['utf-16']},
    'workspace': {'workspaceFolders': True, 'configuration': True, 'didChangeConfiguration': {},
                  'didChangeWatchedFiles': {}, 'diagnostics': {'refreshSupport': True}},
    'textDocument': {
        'synchronization': {'didSave': True},
        'hover': {'contentFormat': ['markdown', 'plaintext']},
        'completion': {'completionItem': {'snippetSupport': True, 'documentationFormat': ['markdown', 'plaintext'],
                                          'labelDetailsSupport': True, 'insertReplaceSupport': True,
                                          'resolveSupport': {'properties': ['documentation', 'detail']}},
                       'contextSupport': True},
        'references': {},
        'documentSymbol': {'hierarchicalDocumentSymbolSupport': True},
        'semanticTokens': {
            'requests': {'full': {'delta': False}, 'range': True},
            'tokenTypes': ['keyword', 'string', 'number', 'variable', 'function', 'type', 'property', 'parameter',
                           'macro', 'operator', 'namespace', 'typeParameter', 'enumMember', 'decorator',
                           'formatSpecifier'],
            'tokenModifiers': ['declaration', 'readonly', 'macro'],
            'formats': ['relative'],
        },
    },
}


def capabilities(mode: str) -> dict:
    caps = json.loads(json.dumps(CLIENT_CAPABILITIES))
    if mode == 'pull':
        caps['textDocument']['diagnostic'] = {'dynamicRegistration': False, 'relatedDocumentSupport': True}
        # A client that pulls does not also take pushes in the way a push-only client does.
        caps['textDocument']['publishDiagnostics'] = {'relatedInformation': True, 'versionSupport': True}
    else:
        caps['textDocument']['publishDiagnostics'] = {'relatedInformation': True, 'versionSupport': True,
                                                      'codeDescriptionSupport': True}
    return caps


def file_uri(path: Path) -> str:
    return path.resolve().as_uri()


class Client:
    """A jailsp process and the JSON-RPC conversation with it. A reader thread timestamps every message the
    moment it is decoded, so waiting in the main thread does not add polling delay."""

    def __init__(self, command: list[str], cwd: Path | None = None, max_rss_mib: float = 0):
        self.max_rss_mib = max_rss_mib
        self.stderr = tempfile.TemporaryFile()
        self.launched = time.perf_counter()
        self.process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.stderr)
        self.next_id = 1
        self.responses: dict[int, tuple[float, dict]] = {}
        self.notifications: list[tuple[float, dict]] = []
        self.condition = threading.Condition()
        self.closed = False
        self.write_lock = threading.Lock()
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()
        self.rusage = None

    def _read(self) -> None:
        decoder = FrameDecoder()
        fd = self.process.stdout.fileno()
        try:
            while True:
                chunk = os.read(fd, 1 << 16)
                if not chunk:
                    break
                for message in decoder.push(chunk):
                    stamp = time.perf_counter()
                    self._dispatch(stamp, message)
        except (OSError, FrameError, ValueError):
            pass
        with self.condition:
            self.closed = True
            self.condition.notify_all()

    def _dispatch(self, stamp: float, message: dict) -> None:
        if 'method' in message and 'id' in message:
            # A request from the server: answer it as an editor with no special settings would.
            params = message.get('params') or {}
            result = [None] * len(params.get('items', [])) if message['method'] == 'workspace/configuration' else None
            self._send({'jsonrpc': '2.0', 'id': message['id'], 'result': result})
            return
        with self.condition:
            if 'id' in message:
                self.responses[message['id']] = (stamp, message)
            else:
                self.notifications.append((stamp, message))
            self.condition.notify_all()

    def _send(self, message: dict) -> None:
        data = encode(message)
        with self.write_lock:
            try:
                self.process.stdin.write(data)
                self.process.stdin.flush()
            except (BrokenPipeError, OSError, ValueError):
                pass

    def notify(self, method: str, params: dict | None = None) -> float:
        start = time.perf_counter()
        self._send({'jsonrpc': '2.0', 'method': method, 'params': params or {}})
        return start

    def send_request(self, method: str, params: dict | None = None) -> tuple[int, float]:
        number = self.next_id
        self.next_id += 1
        start = time.perf_counter()
        self._send({'jsonrpc': '2.0', 'id': number, 'method': method, 'params': params or {}})
        return number, start

    def wait_response(self, number: int, timeout: float) -> tuple[float, dict]:
        deadline = time.perf_counter() + timeout
        with self.condition:
            while number not in self.responses:
                if self.closed:
                    raise ServerExited(self.tail())
                left = deadline - time.perf_counter()
                if left <= 0:
                    raise Timeout(f'no answer within {timeout:g} s')
                self.condition.wait(min(left, 0.5))
                self.check_memory()
            return self.responses.pop(number)

    def request(self, method: str, params: dict | None, timeout: float) -> tuple[float, dict]:
        """(seconds until the answer, the whole response message)."""
        number, start = self.send_request(method, params)
        stamp, message = self.wait_response(number, timeout)
        return stamp - start, message

    def wait_notification(self, method: str, uri: str, start_time: float, timeout: float, since: int = 0) -> float:
        """Seconds from `start_time` until a `method` notification for `uri` arrives (index `since` on)."""
        deadline = time.perf_counter() + timeout
        seen = since
        with self.condition:
            while True:
                while seen < len(self.notifications):
                    stamp, message = self.notifications[seen]
                    seen += 1
                    if message.get('method') == method and message.get('params', {}).get('uri') == uri:
                        return stamp - start_time
                if self.closed:
                    raise ServerExited(self.tail())
                left = deadline - time.perf_counter()
                if left <= 0:
                    raise Timeout(f'no {method} within {timeout:g} s')
                self.condition.wait(min(left, 0.5))
                self.check_memory()

    def tail(self) -> str:
        try:
            self.stderr.seek(0)
            text = self.stderr.read().decode(errors='replace').strip().splitlines()
        except (OSError, ValueError):
            return 'server exited'
        return ' | '.join(text[-3:]) or 'server exited'

    def check_memory(self) -> None:
        """Give up on a server that is eating the machine (a runaway request on a 16 GiB laptop)."""
        if self.max_rss_mib and (rss := self.rss_mib()) and rss > self.max_rss_mib:
            raise MemoryLimit(f'resident set passed {self.max_rss_mib:g} MiB ({rss:.0f} MiB) while waiting')

    def rss_mib(self) -> float | None:
        out = subprocess.run(['ps', '-o', 'rss=', '-p', str(self.process.pid)], capture_output=True, text=True)
        return int(out.stdout) / 1024 if out.stdout.strip().isdigit() else None

    def sample(self) -> dict | None:
        """The server's CPU seconds (user, system) and RSS now, or None when they cannot be read."""
        pid = self.process.pid
        stat = Path(f'/proc/{pid}/stat')
        try:
            if stat.exists():
                fields = stat.read_text().rsplit(')', 1)[1].split()
                ticks = os.sysconf('SC_CLK_TCK')
                return {'cpu_user': int(fields[11]) / ticks, 'cpu_sys': int(fields[12]) / ticks,
                        'rss_mib': int(fields[21]) * os.sysconf('SC_PAGE_SIZE') / 2**20}
            out = subprocess.run(['ps', '-o', 'utime=,stime=,rss=', '-p', str(pid)], capture_output=True, text=True)
            user, system, rss = out.stdout.split()
            return {'cpu_user': parse_cpu_time(user), 'cpu_sys': parse_cpu_time(system), 'rss_mib': int(rss) / 1024}
        except (OSError, ValueError, IndexError):
            return None

    def finish(self, graceful: bool) -> float | None:
        """Shut the server down (politely when it still answers), reap it and return its peak RSS in MiB."""
        if graceful and not self.closed:
            try:
                self.request('shutdown', None, 10)
                self.notify('exit')
            except (Timeout, ServerExited, MemoryLimit):
                pass
        try:
            self.process.stdin.close()
        except OSError:
            pass
        deadline = time.perf_counter() + 10
        status = usage = None
        while True:
            pid, status, usage = os.wait4(self.process.pid, os.WNOHANG)
            if pid:
                break
            if time.perf_counter() > deadline:
                self.process.kill()
                _, status, usage = os.wait4(self.process.pid, 0)
                break
            time.sleep(0.005)
        self.process.returncode = os.waitstatus_to_exitcode(status)
        self.reader.join(2)
        self.process.stdout.close()
        self.rusage = usage
        # ru_maxrss is in bytes on macOS and KiB on Linux.
        return usage.ru_maxrss / (2**20 if sys.platform == 'darwin' else 2**10)


# --------------------------------------------------------------------------------------- session

def failure(error: Exception, oversize: bool) -> dict:
    if isinstance(error, MemoryLimit):
        return {'status': 'memory', 'message': str(error)}
    if isinstance(error, Timeout):
        return {'status': 'timeout', 'message': str(error)}
    if isinstance(error, ServerExited):
        return {'status': 'limit' if oversize else 'crash', 'message': str(error)}
    return {'status': 'error', 'message': str(error)}


def record_diagnostics(info: dict, items: list[dict]) -> None:
    """How many diagnostics the first report had, by code: `jai-limit` means the server gave up on the
    document's syntax layer, `jai-check` is a type-checker error (the facts may be incomplete)."""
    codes: dict[str, int] = {}
    for item in items:
        code = str(item.get('code'))
        codes[code] = codes.get(code, 0) + 1
    info['diagnostics_count'] = len(items)
    info['diagnostic_codes'] = codes


def answer(response: dict):
    if 'error' in response:
        if response['error'].get('code') == -32601:
            raise Unsupported(f"method not found: {response['error'].get('message')}")
        raise RuntimeError(f"{response['error'].get('code')}: {response['error'].get('message')}")
    return response.get('result')


def count_of(result) -> int:
    if result is None:
        return 0
    if isinstance(result, dict):
        return len(result.get('items') or result.get('data') or [])
    return len(result)


# Server capability (in the `initialize` result) each metric's request needs; a missing one makes the
# metric "unsupported", as does a "method not found" answer.
NEEDS = {'hover_first': 'hoverProvider', 'hover_warm': 'hoverProvider', 'edit_hover': 'hoverProvider',
         'completion_member': 'completionProvider', 'typing_first': 'completionProvider',
         'typing_median': 'completionProvider', 'completion_member_typed': 'completionProvider',
         'references': 'referencesProvider', 'document_symbols': 'documentSymbolProvider',
         'semantic_tokens': 'semanticTokensProvider'}


def run_session(command: list[str], workload: dict, mode: str, timeout: float, max_rss_mib: float = 0,
                push_wait: float = 15) -> dict:
    """One fresh server and the whole script of measurements. `workload` has: workspace (Path), document
    (Path), also (list of Paths opened before the document). Every metric is {'seconds': x} or
    {'status': ...}, plus the server's CPU seconds and RSS around the step that produced it."""
    workspace: Path = workload['workspace']
    document: Path = workload['document']
    text = document.read_text(encoding='utf-8', errors='replace')
    uri = file_uri(document)
    oversize = len(text.encode('utf-8')) > DOCUMENT_BYTES
    metrics: dict[str, dict] = {}
    info: dict = {'document_bytes': len(text.encode('utf-8')), 'document_lines': text.count('\n') + 1,
                  'diagnostics': mode}
    client = Client(command, max_rss_mib=max_rss_mib)
    alive = True
    stopped = 'skipped'  # the status of what a dead session did not get to
    server_caps: dict = {}
    version = 1
    try:
        try:
            number, _ = client.send_request('initialize', {
                'processId': os.getpid(), 'clientInfo': {'name': 'lsp_bench'},
                'rootUri': file_uri(workspace),
                'workspaceFolders': [{'uri': file_uri(workspace), 'name': workspace.name}],
                'capabilities': capabilities(mode)})
            stamp, message = client.wait_response(number, timeout)
            server_caps = (answer(message) or {}).get('capabilities', {})
            metrics['startup'] = {'seconds': stamp - client.launched, **(client.sample() or {})}
            client.notify('initialized')
        except (Timeout, ServerExited, MemoryLimit, RuntimeError, Unsupported) as error:
            metrics['startup'] = failure(error, False)
            alive = False

        def document_params(**extra) -> dict:
            return {'textDocument': {'uri': uri}, **extra}

        def step(name: str, action, needs: str | None = None) -> None:
            """Run `action` -> seconds (or None to record nothing); failures become a status, and the CPU
            seconds and RSS around the step are recorded (sampled outside the timed window)."""
            nonlocal alive, stopped
            if not alive:
                metrics[name] = {'status': stopped, 'message': ''}
                return
            if needs and not server_caps.get(needs):
                metrics[name] = {'status': 'unsupported', 'message': f'server does not advertise {needs}'}
                return
            before = client.sample()
            try:
                seconds = action()
                if seconds is not None:
                    metrics[name] = {'seconds': seconds}
            except (Timeout, ServerExited, MemoryLimit) as error:
                metrics[name] = failure(error, oversize)
                alive = False
                stopped = metrics[name]['status'] if metrics[name]['status'] == 'limit' else 'skipped'
            except Unsupported as error:
                metrics[name] = {'status': 'unsupported', 'message': str(error)}
            except RuntimeError as error:
                metrics[name] = {'status': 'error', 'message': str(error)}
                if 'not open in this session' in str(error):
                    # The server dropped the document at didOpen (a size limit): nothing else can work.
                    metrics[name]['status'] = 'limit'
                    alive = False
                    stopped = 'limit'
                    if metrics.get('first_diagnostics', {}).get('seconds') is not None:
                        metrics['first_diagnostics'] = {'status': 'limit', 'message': str(error)}
            if name in metrics and alive:
                after = client.sample()
                if before and after:
                    metrics[name]['cpu_user'] = round(after['cpu_user'] - before['cpu_user'], 3)
                    metrics[name]['cpu_sys'] = round(after['cpu_sys'] - before['cpu_sys'], 3)
                if after:
                    metrics[name]['rss_mib'] = round(after['rss_mib'], 1)

        def timed(method: str, params: dict, key: str | None = None):
            seconds, message = client.request(method, params, timeout)
            result = answer(message)
            if key:
                info[key] = count_of(result) if not isinstance(result, dict) or 'contents' not in result else 1
            return seconds, result

        def change(edits: list[dict]) -> float:
            nonlocal version
            version += 1
            return client.notify('textDocument/didChange', {
                'textDocument': {'uri': uri, 'version': version}, 'contentChanges': edits})

        def open_documents() -> None:
            for other in workload.get('also', []):
                client.notify('textDocument/didOpen', {'textDocument': {
                    'uri': file_uri(other), 'languageId': 'jai', 'version': 1,
                    'text': other.read_text(encoding='utf-8', errors='replace')}})

        picks = pick_positions(text)
        info['picks'] = picks

        def first_diagnostics() -> float | None:
            open_documents()
            since = len(client.notifications)
            start = client.notify('textDocument/didOpen', {'textDocument': {
                'uri': uri, 'languageId': 'jai', 'version': 1, 'text': text}})
            if mode == 'pull':
                seconds, message = client.request('textDocument/diagnostic', document_params(), timeout)
                record_diagnostics(info, (answer(message) or {}).get('items', []))
                return seconds
            try:
                seconds = client.wait_notification('textDocument/publishDiagnostics', uri, start, push_wait, since)
            except Timeout:
                # A server may publish nothing for a clean file.
                metrics['first_diagnostics'] = {'status': 'none', 'message': f'no publishDiagnostics within {push_wait:g} s'}
                return None
            for _, note in client.notifications[since:]:
                if note.get('method') == 'textDocument/publishDiagnostics' and note['params'].get('uri') == uri:
                    record_diagnostics(info, note['params'].get('diagnostics', []))
                    break
            return seconds

        step('first_diagnostics', first_diagnostics, 'diagnosticProvider' if mode == 'pull' else None)
        if alive and picks is None:
            for name in METRICS:
                metrics.setdefault(name, {'status': 'error', 'message': 'no procedure body to pick positions in'})
            alive = False

        if picks:
            hover = picks['hover']
            decl = picks['declaration']
            hover_params = document_params(position={'line': hover['line'], 'character': hover['character']})

            def hover_first() -> float:
                seconds, result = timed('textDocument/hover', hover_params)
                info['hover_result'] = result is not None
                return seconds

            step('hover_first', hover_first, NEEDS['hover_first'])
            step('hover_warm', lambda: median([timed('textDocument/hover', hover_params)[0] for _ in range(5)]),
                 NEEDS['hover_warm'])

            def completion_member() -> float | None:
                if not picks['member']:
                    metrics['completion_member'] = {'status': 'skipped', 'message': 'no member access found'}
                    return None
                member = picks['member']
                return timed('textDocument/completion', document_params(
                    position={'line': member['line'], 'character': member['character']},
                    context={'triggerKind': 2, 'triggerCharacter': '.'}), 'member_items')[0]

            step('completion_member', completion_member, NEEDS['completion_member'])
            step('references', lambda: timed('textDocument/references', document_params(
                position={'line': decl['line'], 'character': decl['character']},
                context={'includeDeclaration': True}), 'references_count')[0], NEEDS['references'])
            step('document_symbols',
                 lambda: timed('textDocument/documentSymbol', document_params(), 'symbols_count')[0],
                 NEEDS['document_symbols'])
            step('semantic_tokens',
                 lambda: timed('textDocument/semanticTokens/full', document_params(), 'tokens_count')[0],
                 NEEDS['semantic_tokens'])

            line_no = picks['insert_line']
            indent = picks['indent']

            def edit_hover() -> float:
                samples = []
                for k in range(3):
                    start = change([insert_edit(line_no, 0, f'{indent}// lsp_bench edit {k}\n')])
                    number, _ = client.send_request('textDocument/hover', hover_params)
                    stamp, message = client.wait_response(number, timeout)
                    answer(message)
                    samples.append(stamp - start)
                info['edit_hover_samples'] = [round(x, 5) for x in samples]
                return median(samples)

            step('edit_hover', edit_hover, NEEDS['edit_hover'])

            def typing() -> float:
                word = hover['name'][:8]
                change([insert_edit(line_no, 0, indent + '\n')])
                times = []
                for column, ch in typing_script(indent, word):
                    start = change([insert_edit(line_no, column, ch)])
                    number, _ = client.send_request('textDocument/completion', document_params(
                        position={'line': line_no, 'character': column + 1}, context={'triggerKind': 1}))
                    stamp, message = client.wait_response(number, timeout)
                    info['typing_items'] = count_of(answer(message))
                    times.append(stamp - start)
                info['typing_word'] = word
                info['typing_samples'] = [round(t, 5) for t in times]
                metrics['typing_median'] = {'seconds': median(times)}
                change([delete_lines_edit(line_no)])
                return times[0]

            step('typing_first', typing, NEEDS['typing_first'])
            if 'seconds' not in metrics.get('typing_first', {}):
                metrics['typing_median'] = dict(metrics['typing_first'])

            def member_typed() -> float | None:
                member = picks['member']
                if not member:
                    metrics['completion_member_typed'] = {'status': 'skipped', 'message': 'no member access found'}
                    return None
                change([insert_edit(line_no, 0, f"{indent}{member['base']}.\n")])
                number, start = client.send_request('textDocument/completion', document_params(
                    position={'line': line_no, 'character': len(indent) + len(member['base']) + 1},
                    context={'triggerKind': 2, 'triggerCharacter': '.'}))
                stamp, message = client.wait_response(number, timeout)
                info['member_typed_items'] = count_of(answer(message))
                change([delete_lines_edit(line_no)])
                return stamp - start

            step('completion_member_typed', member_typed, NEEDS['completion_member_typed'])
    except Exception as error:  # a bug in the harness must not lose the other workloads
        for name in METRICS:
            metrics.setdefault(name, {'status': 'error', 'message': f'harness: {error!r}'})
    finally:
        peak = client.finish(alive)
    for name in METRICS:
        metrics.setdefault(name, {'status': stopped, 'message': ''})
    usage = client.rusage
    return {'metrics': {name: metrics[name] for name in METRICS}, 'peak_rss_mib': peak, 'info': info,
            'cpu_user_s': round(usage.ru_utime, 3) if usage else None,
            'cpu_sys_s': round(usage.ru_stime, 3) if usage else None}


# ------------------------------------------------------------------------------------ workloads

def prepare(name: str, tmp: str, cache: dict) -> dict:
    """The workspace, document and extra documents of a workload (generated ones are written once)."""
    spec = WORKLOADS[name]
    if 'project' in spec:
        workspace = UPSTREAM / spec['project']
        if not workspace.is_dir():
            raise FileNotFoundError(f'{workspace} is missing; run python3 tools/fetch_upstreams.py')
        document = workspace / spec['document']
        also = [workspace / p for p in spec.get('also', [])]
        if not document.is_file():
            raise FileNotFoundError(f'{document} is missing')
        return {'workspace': workspace, 'document': document, 'also': [p for p in also if p.is_file()]}
    if name not in cache:
        sys.path.insert(0, str(ROOT / 'tools'))
        import benchgen
        lines, seed, files = spec['generated']
        directory = Path(tmp) / name
        directory.mkdir()
        for file, content in benchgen.generate(lines, seed, files).items():
            (directory / file).write_text(content)
        parts = sorted(p for p in directory.glob('*.jai') if p.name != 'main.jai')
        if parts:
            cache[name] = {'workspace': directory, 'document': parts[len(parts) // 2],
                           'also': [directory / 'main.jai']}
        else:
            cache[name] = {'workspace': directory, 'document': directory / 'main.jai', 'also': []}
    return cache[name]


def default_jailsp() -> Path:
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
    return target / 'release' / 'jailsp'


def sh(*command: str) -> str:
    try:
        return subprocess.run(command, capture_output=True, text=True, timeout=30).stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return ''


def machine_info() -> dict:
    info = {'system': platform.system(), 'release': platform.release(), 'machine': platform.machine(),
            'python': platform.python_version(), 'cpus': os.cpu_count()}
    if platform.system() == 'Darwin':
        info['cpu'] = sh('sysctl', '-n', 'machdep.cpu.brand_string')
        memory = sh('sysctl', '-n', 'hw.memsize')
        info['macos'] = sh('sw_vers', '-productVersion')
    else:
        info['cpu'] = next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                            if line.startswith('model name')), '') if Path('/proc/cpuinfo').exists() else ''
        memory = next((str(int(line.split()[1]) * 1024) for line in Path('/proc/meminfo').read_text().splitlines()
                       if line.startswith('MemTotal')), '') if Path('/proc/meminfo').exists() else ''
    info['memory_gib'] = round(int(memory) / 2**30, 1) if memory.isdigit() else None
    return info


def server_info(command: list[str]) -> dict:
    commit = sh('git', '-C', str(ROOT), 'rev-parse', 'HEAD')
    dirty = bool(sh('git', '-C', str(ROOT), 'status', '--porcelain', '--untracked-files=no'))
    return {'command': command, 'version': sh(*command, '--version'), 'commit': commit, 'dirty': dirty,
            'rustc': sh('rustc', '--version')}


# ------------------------------------------------------------------------------------- markdown

def cell(metric: dict | None, field: str, scale: float = 1000) -> str:
    if not metric:
        return '-'
    value = metric.get(field)
    if value is None:
        return metric['status'] if metric['status'] != 'ok' else '-'
    text = f'{value * scale:.0f}' if value * scale >= 10 or scale == 1 else f'{value * scale:.1f}'
    return text if metric['status'] == 'ok' else f"{text} ({metric['status']})"


TABLES = (('Warm wall time, ms (median of the sessions after the first)', 'warm', 1000),
          ('Cold wall time, ms (first session)', 'cold', 1000),
          ('Warm CPU time, ms (user + system of the server during the step)', 'cpu_warm', 1000),
          ('Warm resident set after the step, MiB', 'rss_warm', 1))


def markdown(report: dict, baseline: dict | None = None) -> str:
    m = report['machine']
    lines = [f"# Language server latency and memory benchmark ({report['date'][:10]})", '',
             f"Machine: {m.get('cpu') or m['machine']}, {m['cpus']} cores, {m.get('memory_gib')} GiB, "
             f"{m['system']} {m.get('macos') or m['release']}.  "]
    for name, s in report['servers'].items():
        lines.append(f"{name}: {s.get('version') or '?'}, checkout `{s['commit'][:12]}`"
                     f"{' (dirty)' if s['dirty'] else ''}, {s['rustc']}.  ")
    lines += [f"Sessions per workload: {report['settings']['repeat']} (cold = first session, warm = median of the "
              'rest). A cell is a status when there is no number: `limit` (the server refused or died on a '
              'size limit), `timeout`, `memory` (RSS cap), `crash`, `error`, `none` (no diagnostics pushed), '
              '`unsupported`, `skipped`.', '']
    names = list(METRICS)
    for title, field, scale in TABLES:
        extra = field == 'warm'
        lines += [f'## {title}', '',
                  '| workload | doc KiB | ' + ' | '.join(names) + (' | total CPU s | peak RSS MiB |' if extra else ' |')
                  + (' vs baseline |' if baseline and extra else ''),
                  '|---|---:|' + '---:|' * len(names) + ('---:|---:|' if extra else '') + ('---|' if baseline and extra else '')]
        for key, row in report['results'].items():
            cells = [key, f"{row['info'].get('document_bytes', 0) / 1024:.0f}"]
            cells += [cell(row['metrics'].get(n), field, scale) for n in names]
            if extra:
                cells.append(f"{row['cpu_seconds']:.2f}" if row.get('cpu_seconds') is not None else '-')
                cells.append(f"{row['peak_rss_mib']:.0f}" if row.get('peak_rss_mib') else '-')
            if baseline and extra:
                old = baseline.get('results', {}).get(key)
                hover = row['metrics'].get('hover_first', {}).get('warm')
                before = (old or {}).get('metrics', {}).get('hover_first', {}).get('warm')
                cells.append(f'hover {hover / before:.2f}x' if old and hover and before else 'new' if not old else '-')
            lines.append('| ' + ' | '.join(cells) + ' |')
        lines.append('')
    lines += ['Metrics:', ''] + [f'- `{n}`: {label}' for n, label in METRICS.items()]
    lines += ['', 'What the first diagnostics said (diagnostics by code; `jai-limit` = the syntax layer gave up on the '
              'document, `jai-check` = type-checker errors, so later facts may be incomplete; hover null = no fact):', '']
    for key, row in report['results'].items():
        info = row['info']
        if 'diagnostic_codes' in info:
            lines.append(f"- `{key}`: {info['diagnostics_count']} diagnostics {info['diagnostic_codes']}, "
                         f"hover {'answered' if info.get('hover_result') else 'null'}")
    bad = [(key, n, metric) for key, row in report['results'].items() for n, metric in row['metrics'].items()
           if metric['status'] not in ('ok', 'skipped') and metric.get('message')]
    seen = set()
    if bad:
        lines += ['', 'Failures:', '']
        for key, n, metric in bad:
            if (key, metric['status']) in seen:
                continue
            seen.add((key, metric['status']))
            lines.append(f"- `{key}` {n}: {metric['status']}: {metric['message']}")
    if report.get('failures'):
        lines += ['', 'Failed workloads:', '']
        lines += [f'- `{key}`: {message}' for key, message in report['failures'].items()]
    return '\n'.join(lines) + '\n'


# ----------------------------------------------------------------------------------------- main

def parse_servers(specs: list[str]) -> dict[str, list[str]]:
    """`--server NAME=COMMAND...` (repeatable) -> {name: argv}; the default is jailsp from the build tree."""
    servers = {}
    for spec in specs or [f'jailsp={default_jailsp()}']:
        name, sep, command = spec.partition('=')
        if not sep or not name or not command.strip():
            raise SystemExit(f'--server wants NAME=COMMAND, got {spec!r}')
        servers[name] = shlex.split(command)
    return servers


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--server', action='append', metavar='NAME=CMD',
                    help='a language server to run (repeatable; default jailsp=target/release/jailsp); it must '
                         'speak LSP over stdio')
    ap.add_argument('--repeat', type=int, default=5, help='sessions per workload (the first is the cold one)')
    ap.add_argument('--only', default='', help='substring of "server/workload/mode" keys to run')
    ap.add_argument('--diagnostics', default='pull,push', help='client styles to run: pull, push or pull,push')
    ap.add_argument('--timeout', type=float, default=60, help='seconds a single request may take')
    ap.add_argument('--push-wait', type=float, default=15, help='seconds to wait for pushed diagnostics before '
                    'recording "none"')
    ap.add_argument('--max-rss', type=float, default=6144, help='MiB of server RSS at which a waiting request is abandoned')
    ap.add_argument('--out', type=Path, help='write the results as JSON')
    ap.add_argument('--markdown', type=Path, help='write the results as a Markdown table')
    ap.add_argument('--compare', type=Path, help='JSON from an earlier --out to check for regressions')
    ap.add_argument('--threshold', type=float, default=0.20, help='relative growth that counts as a regression')
    ap.add_argument('--min-seconds', type=float, default=0.005, help='ignore smaller absolute time growth')
    ap.add_argument('--min-mib', type=float, default=16, help='ignore smaller absolute RSS growth')
    ap.add_argument('--list', action='store_true', help='print the workloads and exit')
    a = ap.parse_args()
    modes = [m for m in a.diagnostics.split(',') if m]
    if any(m not in ('pull', 'push') for m in modes):
        sys.exit('--diagnostics takes pull, push or pull,push')
    servers = parse_servers(a.server)
    todo = [(f'{server}/{name}/{mode}', server, name, mode) for server in servers for name in WORKLOADS
            for mode in modes if a.only in f'{server}/{name}/{mode}']
    if a.list:
        for key, _, name, _ in todo:
            spec = WORKLOADS[name]
            where = f"{spec['project']}/{spec['document']}" if 'project' in spec else 'generated (lines, seed, files) = %s' % (spec['generated'],)
            print(f'{key:30} {where}')
        return
    for name, command in servers.items():
        if not shutil.which(command[0]) and not Path(command[0]).exists():
            sys.exit(f'{command[0]} (server {name}) not found; build jailsp with cargo build --release -p '
                     'jai-language-server or pass --server NAME=COMMAND')
    baseline = json.loads(a.compare.read_text()) if a.compare else None
    report = {'format': 2, 'date': datetime.now(timezone.utc).isoformat(timespec='seconds'),
              'machine': machine_info(), 'servers': {n: server_info(c) for n, c in servers.items()},
              'settings': {'repeat': a.repeat, 'timeout': a.timeout, 'diagnostics': modes},
              'results': {}, 'failures': {}}
    cache: dict = {}
    with tempfile.TemporaryDirectory(prefix='lsp-bench-') as tmp:
        for key, server, name, mode in todo:
            try:
                workload = prepare(name, tmp, cache)
            except FileNotFoundError as error:
                report['failures'][key] = str(error)
                print(f'{key:30} FAILED {error}', flush=True)
                continue
            sessions = [run_session(servers[server], workload, mode, a.timeout, a.max_rss, a.push_wait)
                        for _ in range(max(1, a.repeat))]
            row = report['results'][key] = summarize(sessions)
            row['server'] = server
            m = row['metrics']

            def show(metric: str) -> str:
                return cell(m.get(metric), 'warm')
            print(f"{key:30} diag {show('first_diagnostics'):>10}  hover1 {show('hover_first'):>8}  "
                  f"warm {show('hover_warm'):>8}  edit {show('edit_hover'):>8}  type1 {show('typing_first'):>8}  "
                  f"cpu {row['cpu_seconds'] or 0:6.2f} s  peak {row['peak_rss_mib'] or 0:6.0f} MiB  (ms)", flush=True)
    text = markdown(report, baseline)
    if a.out:
        a.out.write_text(json.dumps(report, indent=2) + '\n')
    if a.markdown:
        a.markdown.write_text(text)
    print()
    print(text)
    regressions = compare(baseline, report, a.threshold, a.min_seconds, a.min_mib) if baseline else []
    for line in regressions:
        print(f'REGRESSION {line}')
    sys.exit(1 if regressions or report['failures'] else 0)


if __name__ == '__main__':
    main()
