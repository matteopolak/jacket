"""Run every negative HTTP/2 case against the pinned independent peer.

Example: PYTHONPATH=tools python tools/grpc/conformance/run-negative.py --help
Supply prebuilt raw/generated clients and the pinned grpc source checkout.
The caller must acquire the shared heavy-job gate before running this suite.
"""

import argparse
import json
from pathlib import Path
import subprocess
import time
import traceback

from bench.sandbox import Sandbox, environment
from toolkit.repo import ROOT as repo

PIN = "f2f2334fb937ec78b3dc629ccc163b06913deb40"
CASES = (
    "goaway",
    "rst_after_header",
    "rst_during_data",
    "rst_after_data",
    "ping",
    "max_streams",
    "data_frame_padding",
    "no_df_padding_sanity_test",
)
parser = argparse.ArgumentParser(description=__doc__)
for name in ("upstream", "python", "raw-client", "typed-client", "scratch", "output"):
    parser.add_argument("--" + name, type=Path, required=True)
args = parser.parse_args()
args.upstream = args.upstream.resolve()
head = subprocess.check_output(
    ["git", "-C", str(args.upstream), "rev-parse", "HEAD"], text=True
).strip()
if head != PIN:
    raise SystemExit("unexpected upstream revision: " + head)
original = args.upstream / "test/http2_test"
args.scratch = args.scratch.resolve()
args.output = args.output.resolve()
args.scratch.mkdir(parents=True, exist_ok=True)
args.output.mkdir(parents=True, exist_ok=True)
python = args.python.absolute()  # preserve the virtual-environment symlink
clients = {"raw": args.raw_client.resolve(), "typed": args.typed_client.resolve()}
certs = repo / "stdlib/Extensions/Tls/tests/fixtures"
env = environment(args.scratch)
env["PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION"] = "python"
sandbox = Sandbox(
    reads=[
        repo / "tools/grpc",
        original,
        python.parent.parent,
        certs,
        *[path.parent for path in clients.values()],
    ],
    writes=[args.scratch],
    loopback=True,
)
rows = []
for mode, client in clients.items():
    for case in CASES:
        run = args.scratch / (mode + "-" + case)
        run.mkdir(exist_ok=True)
        ready, stop = run / "peer.port", run / "peer.stop"
        ready.unlink(missing_ok=True)
        stop.unlink(missing_ok=True)
        prefix = args.output / (mode + "-" + case)
        code, peer_code, outcome, detail = None, None, "ERROR", ""
        with prefix.with_suffix(".peer.log").open("w") as peerlog:
            peer = subprocess.Popen(
                sandbox.wrap(
                    [
                        str(python),
                        str(repo / "tools/grpc/conformance/negative-peer.py"),
                        str(original),
                        str(run / "compat"),
                        case,
                        str(certs / "server.pem"),
                        str(certs / "server.key"),
                        str(ready),
                        str(stop),
                    ]
                ),
                cwd=run,
                env=env,
                stdout=peerlog,
                stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 30
                while not ready.exists():
                    if peer.poll() is not None:
                        raise RuntimeError("peer exited before readiness")
                    if time.monotonic() > deadline:
                        raise TimeoutError("peer readiness")
                    time.sleep(0.02)
                with prefix.with_suffix(".client.log").open("w") as log:
                    try:
                        code = subprocess.run(
                            sandbox.wrap(
                                [
                                    str(client),
                                    "https://localhost:" + ready.read_text(),
                                    case,
                                    str(certs / "ca.pem"),
                                ]
                            ),
                            cwd=run,
                            env=env,
                            stdout=log,
                            stderr=subprocess.STDOUT,
                            timeout=120,
                        ).returncode
                        outcome = "PASS" if code == 0 else "FAIL"
                    except subprocess.TimeoutExpired:
                        outcome = "TIMEOUT"
            except Exception as error:
                # Keep paths and arbitrary diagnostic context in local logs.
                peerlog.write("harness error:\n")
                traceback.print_exception(error, file=peerlog)
                peerlog.flush()
                detail = type(error).__name__
            finally:
                stop.touch()
                try:
                    peer_code = peer.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    detail += " peer shutdown timeout"
                    peer.terminate()
                    peer_code = peer.wait(timeout=10)
        if peer_code != 0:
            outcome = "ERROR"
        rows.append(
            dict(
                mode=mode,
                case=case,
                outcome=outcome,
                client_exit=code,
                peer_exit=peer_code,
                detail=detail,
            )
        )
        (args.output / "results.json").write_text(
            json.dumps(
                {
                    "grpc_revision": head,
                    "peer": "patched Python3/h2 compatibility fixture with consumed-request correction and verified TLS byte tunnel",
                    "results": rows,
                },
                indent=2,
            )
            + "\n"
        )
        print(mode, case, outcome, code, peer_code, flush=True)
raise SystemExit(
    0 if len(rows) == 16 and all(row["outcome"] == "PASS" for row in rows) else 1
)
