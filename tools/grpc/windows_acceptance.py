"""Run required Windows Io/TLS and HTTP/gRPC cases with verified execution IDs.

Example: PYTHONPATH=tools python tools/grpc/windows_acceptance.py --jacket target/debug/jacket.exe --output target/grpc-windows
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from toolkit.repo import ROOT

PREREQUISITES = (
    "Io::tcp",
    "Io::backends",
    "Io::timeouts",
    "Io::fibers-context",
    "Tls::tls",
)
HTTP_GRPC_REQUIRED = (
    "Http::client",
    "Http::server-bodies",
    "Http::h2-client",
    "Http::h2-header-limits",
    "Http::h2-current-profile",
    "Http::h2-padding",
    "Grpc::calls",
    "Grpc::generated",
    "Grpc::compression",
    "Grpc::server-scope",
)
TERMINAL = {"passed", "failed", "skipped", "timed_out", "xfail", "xpass", "flaky"}


def checked_results(text, required):
    """Require one actual passing terminal event per required case, never a list/skip.

    checked_results(JSONL, ("Io::tcp",)) rejects a successful but empty runner.
    """
    cases = {}
    started = set()
    run_started = None
    run_finished = None
    for line in text.splitlines():
        if not line.strip():
            continue
        event = json.loads(line)
        if not isinstance(event, dict):
            raise ValueError("runner event must be an object")
        kind = event.get("type")
        phase = event.get("event")
        if kind == "run" and phase == "started":
            if run_started is not None or run_finished is not None:
                raise ValueError("duplicate or out-of-order run start")
            run_started = event
        elif kind == "run" and phase == "finished":
            if run_started is None or run_finished is not None:
                raise ValueError("missing start or duplicate run finish")
            run_finished = event
        elif kind == "test" and (phase == "started" or phase in TERMINAL):
            if run_started is None or run_finished is not None:
                raise ValueError("test event outside an active run")
            name = event.get("name")
            if not isinstance(name, str) or not name:
                raise ValueError("missing case identity")
            if phase == "started":
                if name in started or name in cases:
                    raise ValueError("duplicate or out-of-order case start")
                started.add(name)
            else:
                if name in cases:
                    raise ValueError("duplicate terminal case identity")
                if phase != "skipped" and name not in started:
                    raise ValueError("executed case has no start event")
                cases[name] = phase
    if run_started is None or run_finished is None:
        raise ValueError("need one complete runner report")
    selected = run_started.get("tests")
    if type(selected) is not int or selected != len(cases):
        raise ValueError("selected and terminal case counts differ")
    if started - cases.keys():
        raise ValueError("started case has no terminal event")
    if type(run_finished.get("not_run")) is not int or run_finished["not_run"] != 0:
        raise ValueError("runner left selected cases unexecuted")
    for status in TERMINAL:
        reported = run_finished.get(status)
        actual = sum(value == status for value in cases.values())
        if type(reported) is not int or reported != actual:
            raise ValueError("terminal summary count differs: " + status)
    missing = [name for name in required if cases.get(name) != "passed"]
    if missing:
        raise ValueError(
            "required cases did not execute and pass: " + ", ".join(missing)
        )
    bad = [
        name for name, status in cases.items() if status not in {"passed", "skipped"}
    ]
    if bad:
        raise ValueError("runner reported failed cases: " + ", ".join(bad))
    return cases


def commands(jacket):
    """Select module folders and exact prerequisite names; full RPC roots follow."""
    base = [str(jacket), "test", "-j", "2", "--format", "json", "--timeout", "120"]
    return (
        (
            "prerequisites",
            [
                *base,
                "--path",
                "stdlib/Extensions/Io",
                "--path",
                "stdlib/Extensions/Tls",
                "--exact",
                *PREREQUISITES,
            ],
            PREREQUISITES,
        ),
        (
            "http-grpc",
            [
                *base,
                "--path",
                "stdlib/Extensions/Http",
                "--path",
                "stdlib/Extensions/Grpc",
            ],
            HTTP_GRPC_REQUIRED,
        ),
    )


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jacket", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args(argv)
    jacket = args.jacket.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    report = {
        "jacket_sha256": hashlib.sha256(jacket.read_bytes()).hexdigest(),
        "suites": [],
    }
    for name, command, required in commands(jacket):
        raw = args.output / (name + ".jsonl")
        diagnostic = args.output / (name + ".stderr.log")
        with (
            raw.open("w", encoding="utf-8") as output,
            diagnostic.open("w", encoding="utf-8") as error,
        ):
            result = subprocess.run(
                command, cwd=ROOT, stdout=output, stderr=error, check=False
            )
        row = {
            "suite": name,
            "runner_exit": result.returncode,
            "required": list(required),
            "passed": False,
        }
        try:
            row["cases"] = checked_results(raw.read_text(encoding="utf-8"), required)
            row["passed"] = result.returncode == 0
        except (ValueError, OSError) as error:
            with diagnostic.open("a", encoding="utf-8") as log:
                log.write("\nacceptance report error: " + str(error) + "\n")
            row["report_error"] = type(error).__name__
        report["suites"].append(row)
        (args.output / "result.json").write_text(
            json.dumps(report, indent=2) + "\n", encoding="utf-8"
        )
        print(
            name,
            "PASS" if row["passed"] else "FAIL",
            "runner_exit",
            result.returncode,
            flush=True,
        )
    return 0 if all(row["passed"] for row in report["suites"]) else 1


if __name__ == "__main__":
    raise SystemExit(main())
