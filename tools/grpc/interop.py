#!/usr/bin/env python3
"""Run required TLS interop in both directions, with raw and generated services.

Install requirements.txt into a Python 3.14.3 environment first. Example:
PYTHONPATH=tools python tools/grpc/interop.py --jaic target/debug/jaic --peer-python target/grpc-peer/bin/python
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

from toolkit.repo import ROOT, cargo_target_dir
FIXTURES = ROOT / "tools/grpc/fixtures"
CERTS = ROOT / "stdlib/Extensions/Tls/tests/fixtures"


def run(command, log, env, timeout=90):
    """Capture exact commands and failures; no retry or skipped acceptance cases."""
    log.write("$ " + " ".join(map(str, command)) + "\n")
    log.flush()
    subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=log,
                   stderr=subprocess.STDOUT, timeout=timeout, check=True)


def await_port(process, path, timeout):
    """Wait for the fixture's explicit ready file, bounded by timeout."""
    deadline = time.monotonic() + timeout
    while not path.exists():
        if process.poll() is not None:
            raise RuntimeError(f"server exited before ready: {process.returncode}")
        if time.monotonic() >= deadline:
            raise TimeoutError("server did not publish its ready port")
        time.sleep(.02)
    return path.read_text().strip()


def stop(process):
    """Bound process teardown; timeout is a failure rather than a hidden kill/retry."""
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()
        raise


def peer_metadata(peer, jaic):
    """Return reproducible peer identity without raw build, path or host context."""
    versions = subprocess.check_output([str(peer), "-c", "import sys,json,importlib.metadata as m; print(json.dumps({'python':sys.version,'grpcio':m.version('grpcio'),'grpcio-tools':m.version('grpcio-tools'),'protobuf':m.version('protobuf')}))"], text=True)
    identity = json.loads(versions)
    metadata = {name: identity[name] for name in ("python", "grpcio", "grpcio-tools", "protobuf")}
    for name, version in {"grpcio": "1.78.0", "grpcio-tools": "1.78.0", "protobuf": "6.33.5"}.items():
        if metadata[name] != version:
            raise RuntimeError(f"peer {name} must be {version}, found {metadata[name]}")
    if metadata["python"].split()[0] != "3.14.3":
        raise RuntimeError("interop peer requires pinned Python 3.14.3")
    protoc = subprocess.check_output([str(peer), "-m", "grpc_tools.protoc", "--version"], text=True).strip()
    if protoc != "libprotoc 31.1":
        raise RuntimeError(f"bundled protoc must be 31.1, found {protoc}")
    metadata["python"] = metadata["python"].split()[0]
    metadata["protoc"] = protoc
    metadata["source_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    metadata["working_tree_clean"] = not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip()
    metadata.update(platform={"os": platform.system(), "architecture": platform.machine()}, compiler_sha256=hashlib.sha256(jaic.read_bytes()).hexdigest(),
                    schema_sha256=hashlib.sha256((FIXTURES / "echo.proto").read_bytes()).hexdigest(), transport="TLS, ALPN h2 required")
    return metadata


def matrix(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / "result.json").unlink(missing_ok=True)
    # Preserve a venv executable symlink: resolving it selects the base interpreter.
    peer = args.peer_python.absolute()
    jaic = args.jaic.resolve()
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    env["PYTHONPATH"] = str(output) + os.pathsep + env.get("PYTHONPATH", "")
    metadata = peer_metadata(peer, jaic)
    with (output / "setup.log").open("w") as log:
        run([peer, "-m", "grpc_tools.protoc", "--version"], log, env)
        run([peer, "-m", "grpc_tools.protoc", f"--proto_path={FIXTURES}", f"--python_out={output}", FIXTURES / "echo.proto"], log, env)
    results = []
    for typed in (False, True):
        prefix = "typed-" if typed else ""
        options = ["--protobuf"] if typed else []
        suffix = ".exe" if sys.platform == "win32" else ""
        client = output / (prefix + "client" + suffix)
        server = output / (prefix + "server" + suffix)
        with (output / (prefix + "compile.log")).open("w") as log:
            for name, executable in (("client", client), ("server", server)):
                run([jaic, FIXTURES / (prefix + name + ".jai"), "-exe", executable], log, env)
        for direction in ("jai-client", "jai-server"):
            port_file = output / (prefix + direction + ".port")
            port_file.unlink(missing_ok=True)
            server_command = ([peer, ROOT / "tools/grpc/peer.py", "server", *options, "--cert", CERTS / "server.pem", "--key", CERTS / "server.key", "--port-file", port_file]
                              if direction == "jai-client" else [server, port_file])
            with (output / (prefix + direction + "-server.log")).open("w") as server_log:
                process = subprocess.Popen(list(map(str, server_command)), cwd=ROOT, env=env, stdout=server_log, stderr=subprocess.STDOUT)
                try:
                    address = await_port(process, port_file, args.timeout)
                    command = ([client, "https://127.0.0.1:" + address] if direction == "jai-client" else
                               [peer, ROOT / "tools/grpc/peer.py", "client", *options, "--ca", CERTS / "ca.pem", "--target", address])
                    with (output / (prefix + direction + ".log")).open("w") as log:
                        run(command, log, env, args.timeout)
                    results.append({"generated": typed, "direction": direction, "modes": ["unary", "server-streaming", "client-streaming", "bidi"], "deadline_cancel": True, "unary_metadata_errors": True, "passed": True})
                    print(prefix + direction + ": all four modes passed", flush=True)
                finally:
                    stop(process)
    return {"environment": metadata, "results": results}


def run_report(args):
    """Retain raw diagnostics locally; publish only acceptance or a failure class."""
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    try:
        report = {**matrix(args), "passed": True}
    except Exception as error:
        (output / "result.json").write_text(json.dumps({"passed": False, "failure_kind": type(error).__name__}, indent=2) + "\n")
        raise
    (output / "result.json").write_text(json.dumps(report, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--peer-python", type=Path, default=Path(sys.executable))
    parser.add_argument("--output", type=Path, default=cargo_target_dir() / "grpc-interop")
    parser.add_argument("--timeout", type=float, default=30)
    run_report(parser.parse_args())


if __name__ == "__main__":
    main()
