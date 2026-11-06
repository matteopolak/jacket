"""Measure equivalent verified-TLS unary tasks with one optimized Jai client.

Build all three providers first, then run under the shared exclusive quiet gate.
Each timed sample uses the same 271828-byte request and 314159-byte zero response,
a reused channel, identical payload/status checks and a nanosecond client timer.
Only recorded per-call elapsed time is scored; first calls are retained separately.
Example: PYTHONPATH=tools python tools/grpc/conformance/compare-unary.py --help
"""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time

from bench.sandbox import Sandbox, environment
from toolkit.repo import ROOT as repo

parser = argparse.ArgumentParser(description=__doc__)
for name in ("client", "jai-server", "go-server", "scratch", "output"):
    parser.add_argument("--" + name, required=True, type=Path)
parser.add_argument("--repetitions", type=int, default=4)
parser.add_argument("--iterations", type=int, default=1010)
parser.add_argument("--warmup", type=int, default=10)
args = parser.parse_args()
if args.repetitions < 2 or args.warmup < 1 or args.iterations <= args.warmup:
    parser.error("need two repetitions and at least one retained and warmup call")
certs = repo / "stdlib/Extensions/Tls/tests/fixtures"
args.scratch = args.scratch.resolve()
args.output = args.output.resolve()
args.scratch.mkdir(parents=True, exist_ok=True)
args.output.mkdir(parents=True, exist_ok=True)
providers = {"jai": args.jai_server.resolve(), "go": args.go_server.resolve()}
client = args.client.resolve()
env = environment(args.scratch)
env["GOMAXPROCS"] = "2"
sandbox = Sandbox(
    reads=[client.parent, certs, *[path.parent for path in providers.values()]],
    writes=[args.scratch],
    loopback=True,
)
report = {
    "task": "identity unary, 271828 request bytes / 314159 zero response bytes; verified TLS/ALPN h2; one reused channel",
    "timer": "owning optimized Jai client monotonic nanoseconds, including message allocation/encoding/decoding/checking/cleanup; per-call log writes excluded",
    "host": {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
    },
    "GOMAXPROCS": 2,
    "iterations": args.iterations,
    "warmup": args.warmup,
    "client_sha256": hashlib.sha256(client.read_bytes()).hexdigest(),
    "server_sha256": {
        name: hashlib.sha256(path.read_bytes()).hexdigest()
        for name, path in providers.items()
    },
    "binary_size_scope": "executable file only; excludes shared libraries and debug sidecars; not deployment footprint",
    "binary_bytes": {
        "client": client.stat().st_size,
        **{name: path.stat().st_size for name, path in providers.items()},
    },
    "samples": [],
    "allocation_count": None,
    "peak_rss": None,
}
for repetition in range(args.repetitions):
    order = ["jai", "go"] if repetition % 2 == 0 else ["go", "jai"]
    for name in order:
        run = args.scratch / (str(repetition) + "-" + name)
        run.mkdir(exist_ok=True)
        ready, stop = run / "port", run / "stop"
        ready.unlink(missing_ok=True)
        stop.unlink(missing_ok=True)
        command = (
            [str(providers[name]), "raw", str(ready), str(stop)]
            if name == "jai"
            else [
                str(providers[name]),
                str(certs / "server.pem"),
                str(certs / "server.key"),
                str(ready),
                str(stop),
            ]
        )
        prefix = args.output / (str(repetition) + "-" + name)
        with prefix.with_suffix(".server.log").open("w") as log:
            server = subprocess.Popen(
                sandbox.wrap(command),
                cwd=run,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 30
                while not ready.exists():
                    if server.poll() is not None:
                        raise RuntimeError("server exited before readiness")
                    if time.monotonic() > deadline:
                        raise TimeoutError("server readiness")
                    time.sleep(0.02)
                port = ready.read_text().strip().rsplit(":", 1)[-1]
                with prefix.with_suffix(".client.log").open("w") as output:
                    subprocess.run(
                        sandbox.wrap(
                            [
                                str(client),
                                "https://localhost:" + port,
                                "rpc_soak",
                                str(certs / "ca.pem"),
                                str(args.iterations),
                                "1000",
                                "1200",
                            ]
                        ),
                        cwd=run,
                        env=env,
                        stdout=output,
                        stderr=subprocess.STDOUT,
                        timeout=1210,
                        check=True,
                    )
            finally:
                stop.touch()
                server.wait(timeout=20)
        if server.returncode != 0:
            raise RuntimeError("server failed: " + str(server.returncode))
        text = prefix.with_suffix(".client.log").read_text()
        matches = re.findall(
            r"soak iteration: (\d+).* (succeeded|failed) elapsed_ns: (\d+)", text
        )
        if len(matches) != args.iterations or any(
            int(i) != n or result != "succeeded"
            for n, (i, result, _) in enumerate(matches)
        ):
            raise RuntimeError("incomplete or failed measured task")
        elapsed = [int(value) for _, _, value in matches]
        values = sorted(elapsed[args.warmup :])

        def quantile(p):
            return values[round((len(values) - 1) * p)] / 1e6

        row = {
            "provider": name,
            "repetition": repetition,
            "cold_first_ms": elapsed[0] / 1e6,
            "warmup_ns": elapsed[: args.warmup],
            "measured_ns": elapsed[args.warmup :],
            "p50_ms": quantile(0.5),
            "p95_ms": quantile(0.95),
            "p99_ms": quantile(0.99),
            "mean_ms": statistics.mean(values) / 1e6,
            "tasks_per_second": len(values) * 1e9 / sum(values),
        }
        report["samples"].append(row)
        (args.output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
        print(
            name,
            repetition,
            "p50_ms",
            row["p50_ms"],
            "tasks/s",
            row["tasks_per_second"],
            flush=True,
        )
