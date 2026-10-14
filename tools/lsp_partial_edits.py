#!/usr/bin/env python3
"""Run source-built IDE correctness checks in timeout-isolated per-file processes.

This is a regression harness, not a benchmark. It never executes supplied binaries.
"""
import argparse
import concurrent.futures
import json
import hashlib
from pathlib import Path
import subprocess
import shutil
import sys
import uuid
import queue
import threading
import time
from collections import deque

ROOT = Path(__file__).resolve().parents[1]


def sources(paths):
    files = set()
    for path in paths:
        path = Path(path).resolve()
        if path.is_file():
            files.add(path)
        else:
            files.update(p for p in path.rglob("*.jai") if p.is_file())
    return sorted(files)


def build_worker():
    result = subprocess.run(
        ["cargo", "build", "-p", "jai-language-server", "--example", "partial_edit_coverage", "--message-format=json"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if result.returncode:
        for line in result.stdout.splitlines():
            try:
                message = json.loads(line).get("message", {})
            except ValueError:
                continue
            if message.get("rendered"):
                print(message["rendered"], file=sys.stderr)
        raise RuntimeError("source worker build failed: " + result.stderr[-4096:])
    for line in result.stdout.splitlines():
        msg = json.loads(line)
        if msg.get("reason") == "compiler-artifact" and msg.get("target", {}).get("name") == "partial_edit_coverage" and msg.get("executable"):
            # Later local builds must not change the binary midway through a sweep.
            worker_dir = ROOT / "artifacts/lsp-edits/workers"
            worker_dir.mkdir(parents=True, exist_ok=True)
            worker = worker_dir / str(uuid.uuid4())
            shutil.copy2(msg["executable"], worker)
            return str(worker)
    raise RuntimeError("cargo did not report the source-built worker")


def run_file(worker, file, args):
    command = [worker, str(file), str(args.stdlib.resolve()) if args.stdlib else "-", str(args.max_lines), "--progress"]
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", cwd=ROOT)
    events = queue.Queue()
    errors = deque(maxlen=64)
    def read_stdout():
        for line in process.stdout:
            events.put(line)
        events.put(None)
    def read_stderr():
        for line in process.stderr:
            errors.append(line[-8192:])
    readers = [threading.Thread(target=read_stdout, daemon=True), threading.Thread(target=read_stderr, daemon=True)]
    for reader in readers:
        reader.start()
    report = None
    protocol_errors = deque(maxlen=64)
    last_phase = "worker startup"
    deadline = time.monotonic() + args.timeout
    try:
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise queue.Empty
            line = events.get(timeout=remaining)
            if line is None:
                break
            try:
                message = json.loads(line)
            except ValueError:
                protocol_errors.append("invalid worker JSON: " + line[-256:].strip())
                continue
            if not isinstance(message, dict):
                protocol_errors.append("worker message must be an object")
            elif "progress" in message and isinstance(message["progress"], str) and report is None:
                last_phase = message["progress"]
                deadline = time.monotonic() + args.timeout
            elif (
                "progress" not in message
                and report is None
                and message.get("file") == str(file)
                and isinstance(message.get("failures"), list)
                and all(isinstance(failure, str) for failure in message["failures"])
                and all(type(message.get(key)) is int and message[key] >= 0 for key in ("edits", "assertions"))
            ):
                report = message
            else:
                protocol_errors.append("invalid or duplicate worker report/progress message")
        process.wait(timeout=args.timeout)
    except (queue.Empty, subprocess.TimeoutExpired):
        process.kill()
        process.wait()
        report = {"file": str(file), "failures": [f"request stalled for {args.timeout}s at {last_phase}"], "edits": None, "assertions": None}
    for reader in readers:
        reader.join(timeout=1)
    process.stdout.close()
    process.stderr.close()
    if report is None:
        report = {"file": str(file), "failures": ["worker did not return a final JSON report"], "edits": None, "assertions": None}
    if process.returncode and not report["failures"]:
        report["failures"].append(f"worker exit {process.returncode}")
    if protocol_errors:
        report["failures"].extend(protocol_errors)
        report["edits"] = None
        report["assertions"] = None
    if errors:
        report["stderr"] = "".join(errors)[-8192:]
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path)
    parser.add_argument("--retry-report", type=Path, help="rerun failed files from an earlier correctness report")
    parser.add_argument("--stdlib", type=Path, help="NEW independent stdlib, using prelude/module.jai")
    parser.add_argument("--max-lines", type=int, default=8, help="evenly sampled nonempty lines per file; 0 checks every line")
    parser.add_argument("--timeout", type=int, default=60, help="hard deadline for a stalled request/stage, reset by worker progress")
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.max_lines < 0 or args.timeout <= 0 or args.jobs <= 0:
        parser.error("limits must be nonnegative, with positive timeout/jobs")
    paths = list(args.paths)
    if args.retry_report:
        paths.extend(Path(r["file"]) for r in json.loads(args.retry_report.read_text())["files"] if r["failures"])
    files = sources(paths)
    if not files:
        parser.error("no Jai sources found")
    worker = build_worker()
    provenance = {"worker_sha256": hashlib.sha256(Path(worker).read_bytes()).hexdigest(),
                  "max_lines": args.max_lines, "stdlib": str(args.stdlib.resolve()) if args.stdlib else None,
                  "timeout_seconds": args.timeout}
    reports = []
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        pending = {pool.submit(run_file, worker, file, args): file for file in files}
        for future in concurrent.futures.as_completed(pending):
            report = future.result()
            reports.append(report)
            print(f"{len(reports)}/{len(files)} {'FAIL' if report['failures'] else 'PASS'} {pending[future]} ({report['edits']} edits)", flush=True)
            # Persist after each completed file so interruptions do not lose evidence.
            partial = args.report.with_name(args.report.name + ".partial")
            partial.write_text(json.dumps({**provenance, "files": sorted(reports, key=lambda r: r["file"])}, indent=2) + "\n")
            partial.replace(args.report)
    failed = sum(bool(r["failures"]) for r in reports)
    print(f"{len(reports)} files, {sum(r['edits'] or 0 for r in reports)} exercised edits, {sum(r['assertions'] or 0 for r in reports)} assertions, {failed} failed files (incomplete workers are not counted)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
