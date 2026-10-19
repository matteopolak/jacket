"""A minimal language-server client: framed JSON-RPC over a child's pipes, standard library only."""

import json
import subprocess
import threading

__all__ = ["Lsp"]


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
        out = self.proc.stdout
        while True:
            length = None
            while True:
                line = out.readline()
                if not line:
                    return
                line = line.strip()
                if not line:
                    break
                if line.lower().startswith(b"content-length:"):
                    length = int(line.split(b":")[1])
            if length is None:
                continue
            message = json.loads(out.read(length))
            with self.cond:
                if "id" in message and "method" not in message:
                    self.responses[message["id"]] = message
                elif "id" in message:
                    self._send({"jsonrpc": "2.0", "id": message["id"], "result": None})
                else:
                    self.notifications.append(message)
                self.cond.notify_all()

    def _send(self, message):
        body = json.dumps(message).encode()
        self.proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
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
