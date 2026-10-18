"""A small LSP client for benchmarking: framing, a reader thread that timestamps messages, process sampling."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

from measure import kill_group

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



def parse_cpu_time(text: str) -> float:
    """`ps` CPU time: [DD-][HH:]MM:SS[.cc] or M:SS.cc, in seconds."""
    days, _, rest = text.rpartition('-')
    seconds = 0.0
    for part in rest.split(':'):
        seconds = seconds * 60 + float(part)
    return seconds + (int(days) * 86400 if days else 0)


def darwin_task_times():
    """A reader of (user seconds, system seconds, RSS bytes) of a process by pid on macOS, from
    `proc_pidinfo(PROC_PIDTASKINFO)`: microsecond resolution where `ps` has 10 ms. None elsewhere or when
    libproc cannot be loaded."""
    if sys.platform != 'darwin':
        return None
    try:
        import ctypes
        import ctypes.util

        libproc = ctypes.CDLL(ctypes.util.find_library('proc') or '/usr/lib/libproc.dylib')
        libc = ctypes.CDLL(ctypes.util.find_library('c') or '/usr/lib/libSystem.B.dylib')

        class TaskInfo(ctypes.Structure):
            _fields_ = [('virtual_size', ctypes.c_uint64), ('resident_size', ctypes.c_uint64),
                        ('total_user', ctypes.c_uint64), ('total_system', ctypes.c_uint64),
                        ('threads_user', ctypes.c_uint64), ('threads_system', ctypes.c_uint64),
                        ('rest', ctypes.c_int32 * 12)]

        class Timebase(ctypes.Structure):
            _fields_ = [('numer', ctypes.c_uint32), ('denom', ctypes.c_uint32)]

        timebase = Timebase()
        libc.mach_timebase_info(ctypes.byref(timebase))
        scale = timebase.numer / timebase.denom / 1e9  # mach absolute time units -> seconds
        libproc.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]

        def read(pid: int):
            info = TaskInfo()
            size = libproc.proc_pidinfo(pid, 4, 0, ctypes.byref(info), ctypes.sizeof(info))  # PROC_PIDTASKINFO
            if size < 40:
                return None
            return info.total_user * scale, info.total_system * scale, info.resident_size

        return read
    except (OSError, AttributeError, ValueError):
        return None


TASK_TIMES = darwin_task_times()


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
        'definition': {'linkSupport': False},
        'rename': {'prepareSupport': True},
        'signatureHelp': {'signatureInformation': {'documentationFormat': ['markdown', 'plaintext'],
                                                   'parameterInformation': {'labelOffsetSupport': True}}},
        'inlayHint': {},
        'formatting': {},
        'rangeFormatting': {},
        'documentSymbol': {'hierarchicalDocumentSymbolSupport': True},
        'semanticTokens': {
            'requests': {'full': {'delta': True}, 'range': True},
            'tokenTypes': ['keyword', 'string', 'number', 'variable', 'function', 'type', 'property', 'parameter',
                           'macro', 'operator', 'namespace', 'typeParameter', 'enumMember', 'decorator',
                           'formatSpecifier'],
            'tokenModifiers': ['declaration', 'readonly', 'macro'],
            'formats': ['relative'],
        },
    },
}


def capabilities(pull_diagnostics: bool) -> dict:
    """What a typical editor announces; a client that pulls diagnostics says so, one that does not gets pushes."""
    caps = json.loads(json.dumps(CLIENT_CAPABILITIES))
    publish = {'relatedInformation': True, 'versionSupport': True, 'codeDescriptionSupport': True}
    caps['textDocument']['publishDiagnostics'] = publish
    if pull_diagnostics:
        caps['textDocument']['diagnostic'] = {'dynamicRegistration': False, 'relatedDocumentSupport': True}
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
                                        stderr=self.stderr, start_new_session=True)
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
            if TASK_TIMES and (times := TASK_TIMES(pid)):
                return {'cpu_user': times[0], 'cpu_sys': times[1], 'rss_mib': times[2] / 2**20}
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
                kill_group(self.process.pid)
                _, status, usage = os.wait4(self.process.pid, 0)
                break
            time.sleep(0.005)
        self.process.returncode = os.waitstatus_to_exitcode(status)
        kill_group(self.process.pid)
        self.reader.join(2)
        self.process.stdout.close()
        self.rusage = usage
        # ru_maxrss is in bytes on macOS and KiB on Linux.
        return usage.ru_maxrss / (2**20 if sys.platform == 'darwin' else 2**10)
