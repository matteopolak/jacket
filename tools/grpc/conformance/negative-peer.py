"""Run the pinned official HTTP/2 negative peer through a byte-preserving TLS tunnel.

Example: python negative-peer.py UPSTREAM COPY CASE CERT KEY READY STOP
The compatibility copy changes Python/API syntax and fixes consumed-request bookkeeping; its complete diff and
original/adapted hashes are written beside READY. No assertion is removed.
"""

from pathlib import Path
import difflib
import hashlib
import importlib
import json
import logging
import re
import select
import socket
import ssl
import sys
import threading

source, target, case, cert, key, ready, stop = sys.argv[1:]
source, target, ready, stop = map(Path, (source, target, ready, stop))
target.mkdir(parents=True, exist_ok=True)
identities = []
patches = []
for path in sorted(source.glob("*.py")):
    original = path.read_text()
    adapted = re.sub(r"(\w+(?:\.\w+)*)\.has_key\(([^)]+)\)", r"(\2 in \1)", original)
    adapted = adapted.replace(
        "import h2.connection",
        "import h2.connection\nimport h2.config\nimport h2.settings",
    )
    adapted = adapted.replace(
        "h2.connection.H2Connection(client_side=False)",
        "h2.connection.H2Connection(config=h2.config.H2Configuration(client_side=False))",
    )
    adapted = adapted.replace("h2.events.PingAcknowledged", "h2.events.PingAckReceived")
    adapted = adapted.replace(
        "hyperframe.frame.SettingsFrame.MAX_CONCURRENT_STREAMS",
        "h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS",
    )
    if path.name == "test_max_streams.py":
        adapted = adapted.replace(
            "import hyperframe.frame", "import hyperframe.frame\nimport h2.settings"
        )
    adapted = adapted.replace(
        'self._recv_buffer[event.stream_id] = ""',
        'self._recv_buffer[event.stream_id] = b""',
    )
    adapted = adapted.replace("response_len / 2", "response_len // 2")
    if path.name == "http2_base_server.py":
        # A legal empty END_STREAM DATA follows a consumed request. The old
        # fixture parsed that retained request twice and restarted its response.
        adapted = adapted.replace(
            "        grpc_msg_size = struct.unpack",
            "        if len(recv_buffer) < _GRPC_HEADER_SIZE:\n            return None\n        grpc_msg_size = struct.unpack",
        )
        adapted = adapted.replace(
            "        sr.ParseFromString(req_proto_str)",
            '        sr.ParseFromString(req_proto_str)\n        self._recv_buffer[stream_id] = b""',
        )
    (target / path.name).write_text(adapted, newline="\n")
    identities.append(
        {
            "path": path.name,
            "original_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "adapted_sha256": hashlib.sha256(
                (target / path.name).read_bytes()
            ).hexdigest(),
        }
    )
    patches.extend(
        difflib.unified_diff(
            original.splitlines(True),
            adapted.splitlines(True),
            fromfile=path.name,
            tofile=path.name,
        )
    )
ready.with_suffix(".compat.patch").write_text("".join(patches))
ready.with_suffix(".sources.json").write_text(json.dumps(identities, indent=2) + "\n")
sys.path.insert(0, str(target))
http2_test_server = importlib.import_module("http2_test_server")
reactor = importlib.import_module("twisted.internet.reactor")
TCP4ServerEndpoint = importlib.import_module(
    "twisted.internet.endpoints"
).TCP4ServerEndpoint
globalLogPublisher = importlib.import_module("twisted.logger").globalLogPublisher

# Twisted otherwise logs callback assertion failures without a nonzero exit.
errors = []


def observe(event):
    if event.get("isError") or event.get("log_failure"):
        errors.append(str(event))


globalLogPublisher.addObserver(observe)
logging.basicConfig(level=logging.INFO)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(cert, key)
context.set_alpn_protocols(["h2"])
listener = socket.socket()
listener.bind(("127.0.0.1", 0))
listener.listen(128)
listener.settimeout(0.2)
backend_port = None
connections = []


def tunnel(raw):
    try:
        with context.wrap_socket(raw, server_side=True) as front:
            assert front.selected_alpn_protocol() == "h2"
            with socket.create_connection(("127.0.0.1", backend_port)) as back:
                while True:
                    readable, _, _ = select.select([front, back], [], [], 0.1)
                    if front.pending() and front not in readable:
                        readable.append(front)
                    for incoming in readable:
                        data = incoming.recv(65536)
                        if not data:
                            return
                        (back if incoming is front else front).sendall(data)
    except (ConnectionResetError, BrokenPipeError, ssl.SSLEOFError):
        pass
    except Exception as error:
        errors.append(repr(error))


def accept():
    while not stop.exists():
        try:
            raw, _ = listener.accept()
        except socket.timeout:
            continue
        thread = threading.Thread(target=tunnel, args=(raw,))
        connections.append(thread)
        thread.start()
    listener.close()
    for thread in connections:
        thread.join(10)
        if thread.is_alive():
            errors.append("tunnel did not close")
    reactor.callFromThread(reactor.stop)


def started(port):
    global backend_port
    backend_port = port.getHost().port
    threading.Thread(target=accept).start()
    ready.write_text(str(listener.getsockname()[1]))


def failed(reason):
    errors.append(str(reason))
    reactor.stop()


endpoint = TCP4ServerEndpoint(reactor, 0, interface="127.0.0.1")
endpoint.listen(http2_test_server.H2Factory(case)).addCallbacks(started, failed)
reactor.run()
if errors:
    print(json.dumps({"peer_errors": errors}), flush=True)
    raise SystemExit(1)
print("canonical negative peer completed without callback failures", flush=True)
