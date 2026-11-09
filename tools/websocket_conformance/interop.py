#!/usr/bin/env python3
"""Run the pinned Python fallback in both directions and compare echo performance.

Example: PYTHONPATH=tools python3.14 tools/websocket_conformance/interop.py --binary /tmp/ws-bench --output /tmp/ws-run
Build bench/bench.jai with the prebuilt jaic before running; the runner never builds.
"""
import argparse
import asyncio
import importlib.metadata
import os
from pathlib import Path
import shutil
import ssl
import statistics
import sys
import time

from ci.pins import load
from fetch import fetch
from toolkit.repo import ROOT

sys.path.insert(0, str(fetch()))
from websockets.asyncio.client import connect
from websockets.asyncio.server import serve
from websockets.extensions.permessage_deflate import ServerPerMessageDeflateFactory

CERTS = ROOT / "stdlib/Extensions/Tls/tests/fixtures"


async def echo(connection):
    """Echo text and bytes, fragmenting large messages so both peers exercise continuation."""
    async for message in connection:
        if len(message) > 4096:
            await connection.send([message[i:i + 1024] for i in range(0, len(message), 1024)])
        else:
            await connection.send(message)


async def peer_server(args):
    context = None
    if args.tls:
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(CERTS / "server.pem", CERTS / "server.key")
        context.set_alpn_protocols(["http/1.1"])
    extensions = []
    if args.compression:
        extensions = [ServerPerMessageDeflateFactory(
            server_max_window_bits=12, client_max_window_bits=12,
            server_no_context_takeover=args.no_context,
            client_no_context_takeover=args.no_context)]
    async with serve(echo, "127.0.0.1", 0, ssl=context, compression=None,
                     extensions=extensions, subprotocols=["echo"], ping_interval=None) as server:
        args.port_file.write_text("127.0.0.1:" + str(server.sockets[0].getsockname()[1]))
        await asyncio.Future()


async def ready(process, path):
    """Read an explicit ready address; an exited process or ten-second deadline fails."""
    deadline = time.monotonic() + 10
    while not path.exists():
        if process.returncode is not None:
            raise RuntimeError(f"server exited: {process.returncode}")
        if time.monotonic() >= deadline:
            raise TimeoutError("server readiness timed out")
        await asyncio.sleep(.02)
    return path.read_text().strip()


async def peer_client(url, compression, benchmark=False):
    context = ssl.create_default_context(cafile=str(CERTS / "ca.pem")) if url.startswith("wss:") else None
    async with connect(url, ssl=context, compression="deflate" if compression else None,
                       subprotocols=["echo"], ping_interval=None, proxy=None) as connection:
        assert connection.subprotocol == "echo"
        assert bool(connection.protocol.extensions) == compression
        if benchmark:
            payload = b"x" * 64
            for _ in range(100):
                await connection.send(payload)
                assert await connection.recv() == payload
            samples = []
            start = time.perf_counter()
            for _ in range(5000):
                before = time.perf_counter_ns()
                await connection.send(payload)
                assert await connection.recv() == payload
                samples.append((time.perf_counter_ns() - before) / 1000)
            elapsed = time.perf_counter() - start
            samples.sort()
            return 5000 / elapsed, statistics.median(samples), samples[int(.99 * len(samples))]
        for message in ["hello", "日本語😀", "", b"", bytes(range(256)), "x" * 100000,
                        "x" * 100000, "x" * 100000]:
            if len(message) > 4096:
                await connection.send([message[i:i + 1024] for i in range(0, len(message), 1024)])
            else:
                await connection.send(message)
            assert await connection.recv() == message
        pong = await connection.ping(b"ping")
        await asyncio.wait_for(pong, 2)
        await connection.close(1000, "done")
        assert connection.close_code == 1000


async def run(args):
    args.output.mkdir(parents=True, exist_ok=True)
    pins = load()
    if sys.version.split()[0] != pins["python_websocket"]:
        raise RuntimeError(f"use pinned Python {pins['python_websocket']}")
    assert importlib.metadata.version("websockets") == pins["websockets"]
    print(f"Python {sys.version.split()[0]}, websockets {pins['websockets']}", flush=True)
    if not shutil.which("docker"):
        print("Autobahn not run: Docker is unavailable; using pinned Python fallback", flush=True)
    else:
        print("Autobahn not run by this Python fallback runner", flush=True)
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    # Eight direction/transport/extension combinations, plus no-context-takeover clients.
    for tls in (False, True):
        for compression in (False, True):
            for direction in ("jai-server", "jai-client"):
                contexts = (False, True) if direction == "jai-client" and compression else (False,)
                for no_context in contexts:
                    label = f"{direction}-{'wss' if tls else 'ws'}-{'deflate' if compression else 'plain'}-{'reset' if no_context else 'takeover'}"
                    port_file = args.output / (label + ".port")
                    port_file.unlink(missing_ok=True)
                    if direction == "jai-server":
                        command = [args.binary, "server", port_file, "deflate" if compression else "plain"]
                        if tls:
                            command.append("tls")
                    else:
                        command = [sys.executable, __file__, "--peer-server", "--port-file", port_file]
                        if tls:
                            command.append("--tls")
                        if compression:
                            command.append("--compression")
                        if no_context:
                            command.append("--no-context")
                    with (args.output / (label + ".log")).open("w") as log:
                        process = await asyncio.create_subprocess_exec(*map(str, command), env=env, stdout=log, stderr=log)
                        try:
                            address = await ready(process, port_file)
                            url = ("wss" if tls else "ws") + "://" + address + "/"
                            if direction == "jai-server":
                                await asyncio.wait_for(peer_client(url, compression), 30)
                            else:
                                client = await asyncio.create_subprocess_exec(str(args.binary), "client", url,
                                    "deflate" if compression else "plain", env=env, stdout=log, stderr=log)
                                assert await asyncio.wait_for(client.wait(), 30) == 0, label
                            print(label + ": PASS", flush=True)
                        finally:
                            if process.returncode is None:
                                process.terminate()
                            await asyncio.wait_for(process.wait(), 10)
    if not args.bench:
        return
    for implementation in ("jai", "python"):
        port_file = args.output / (implementation + "-bench.port")
        port_file.unlink(missing_ok=True)
        command = ([args.binary, "server", port_file, "plain"] if implementation == "jai" else
                   [sys.executable, __file__, "--peer-server", "--port-file", port_file])
        with (args.output / (implementation + "-bench.log")).open("w") as log:
            process = await asyncio.create_subprocess_exec(*map(str, command), env=env, stdout=log, stderr=log)
            try:
                address = await ready(process, port_file)
                rate, median, p99 = await peer_client("ws://" + address + "/", False, True)
                print(f"{implementation}: {rate:.0f} echoes/s, RTT p50 {median:.1f} us, p99 {p99:.1f} us (64 B, one connection, 5000 echoes, no compression)", flush=True)
            finally:
                if process.returncode is None:
                    process.terminate()
                await asyncio.wait_for(process.wait(), 10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--bench", action="store_true")
    parser.add_argument("--peer-server", action="store_true")
    parser.add_argument("--port-file", type=Path)
    parser.add_argument("--tls", action="store_true")
    parser.add_argument("--compression", action="store_true")
    parser.add_argument("--no-context", action="store_true")
    args = parser.parse_args()
    if not args.peer_server and (not args.binary or not args.output):
        parser.error("--binary and --output are required")
    asyncio.run(peer_server(args) if args.peer_server else run(args))


if __name__ == "__main__":
    main()
