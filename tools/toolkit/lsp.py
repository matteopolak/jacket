"""A minimal language-server client: framed JSON-RPC over a child's pipes, standard library only.

`encode` and `FrameDecoder` are the one implementation of the `Content-Length` framing; the benchmark client
and the PGO trainer use them too.
"""

import json
import os
import subprocess
import threading

__all__ = ["FrameDecoder", "FrameError", "Lsp", "encode"]


class FrameError(ValueError):
    pass


def encode(message: dict) -> bytes:
    """One LSP frame: `Content-Length` header, blank line, compact JSON body."""
    body = json.dumps(message, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return b"Content-Length: %d\r\n\r\n" % len(body) + body


class FrameDecoder:
    """Incremental decoder: feed it arbitrary byte chunks, get complete JSON bodies back."""

    def __init__(self) -> None:
        self.buffer = b""

    def push(self, data: bytes) -> list[dict]:
        self.buffer += data
        messages = []
        while True:
            end = self.buffer.find(b"\r\n\r\n")
            if end < 0:
                return messages
            length = None
            for line in self.buffer[:end].decode("ascii", errors="replace").split("\r\n"):
                name, _, value = line.partition(":")
                if name.strip().lower() == "content-length":
                    try:
                        length = int(value.strip())
                    except ValueError:
                        raise FrameError(f"bad Content-Length {value.strip()!r}") from None
            if length is None:
                raise FrameError("header without Content-Length")
            start = end + 4
            if len(self.buffer) < start + length:
                return messages
            messages.append(json.loads(self.buffer[start : start + length].decode("utf-8")))
            self.buffer = self.buffer[start + length :]


class Lsp:
    """One server process; the notifications it sent are kept in `notifications`."""

    def __init__(self, exe, cwd, env=None):
        self.proc = subprocess.Popen([str(exe)], cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env)
        self.next_id = 0
        self.responses = {}
        self.notifications = []
        self.cond = threading.Condition()
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        decoder = FrameDecoder()
        fd = self.proc.stdout.fileno()
        while True:
            try:
                chunk = os.read(fd, 1 << 16)
                messages = decoder.push(chunk) if chunk else None
            except (OSError, FrameError):
                return
            if messages is None:
                return
            for message in messages:
                with self.cond:
                    if "id" in message and "method" not in message:
                        self.responses[message["id"]] = message
                    elif "id" in message:
                        self._send({"jsonrpc": "2.0", "id": message["id"], "result": None})
                    else:
                        self.notifications.append(message)
                    self.cond.notify_all()

    def _send(self, message):
        self.proc.stdin.write(encode(message))
        self.proc.stdin.flush()

    def notify(self, method, params):
        self._send({"jsonrpc": "2.0", "method": method, "params": params})

    def request(self, method, params, timeout=60):
        self.next_id += 1
        ident = self.next_id
        self._send({"jsonrpc": "2.0", "id": ident, "method": method, "params": params})
        with self.cond:
            if not self.cond.wait_for(lambda: ident in self.responses, timeout):
                raise TimeoutError(method)
            return self.responses.pop(ident).get("result")

    def diagnostics(self, uri, timeout=60):
        with self.cond:
            self.cond.wait_for(lambda: any(n.get("method") == "textDocument/publishDiagnostics" and n["params"]["uri"] == uri for n in self.notifications), timeout)
            return [n["params"]["diagnostics"] for n in self.notifications if n.get("method") == "textDocument/publishDiagnostics" and n["params"]["uri"] == uri][-1:]

    def close(self):
        try:
            self.request("shutdown", None, 10)
            self.notify("exit", None)
            self.proc.wait(10)
        except Exception:
            self.proc.kill()
