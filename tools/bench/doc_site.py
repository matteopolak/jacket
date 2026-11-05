#!/usr/bin/env python3
"""Reproducible sequential documentation cases; validation alone takes no timings.

Execute only inside the shared heavy.py --exclusive gate after tool/source coordination:
  python3 tools/bench/doc_site.py --plan /tmp/plan.json --out /tmp/owned-doc-bench --execute
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import statistics
import subprocess
import sys
import threading
import time


def tree_rss(rows, root):
    """Sum one observed process tree, including same-group orphans, never the sampler's siblings."""
    selected = {root} | {pid for pid, _, group, _ in rows if group == root}
    previous = set()
    while previous != selected:
        previous = set(selected)
        selected.update(pid for pid, parent, _, _ in rows if parent in selected)
    owned = [(pid, rss) for pid, _, _, rss in rows if pid in selected]
    return sum(rss for _, rss in owned), len(owned)


def summarize(rows):
    """Retain failures; successful samples are not silently selected into a winning average."""
    def stat(key):
        values = [row[key] for row in rows if row.get(key) is not None]
        return {"median": statistics.median(values), "min": min(values), "max": max(values)} if values else None
    return {"allPassed": all(row["exit"] == 0 for row in rows),
            "failures": [index for index, row in enumerate(rows) if row["exit"] != 0],
            "wallSeconds": stat("wallSeconds"), "sampledTreePeakKiB": stat("sampledTreePeakKiB")}


def measure(command, cwd, env, log, timeout):
    """Use wait4 tool-tree CPU accounting and independently sampled aggregate RSS (a lower bound)."""
    if not hasattr(os, "wait4"):
        raise ValueError("documentation measurements currently require POSIX wait4 (macOS/Linux)")
    stopped = threading.Event()
    observed = {"peakKiB": 0, "samples": 0, "maxProcesses": 0, "errors": []}
    timed_out = threading.Event()
    started = time.perf_counter()
    with log.open("wb") as output:
        child = subprocess.Popen(command, cwd=cwd, env=env, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        def terminate():
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        def expire():
            timed_out.set()
            terminate()
        def sample():
            while not stopped.is_set():
                try:
                    result = subprocess.run(["ps", "-axo", "pid=,ppid=,pgid=,rss="], capture_output=True, text=True, check=True)
                    rows = [tuple(map(int, row.split())) for row in result.stdout.splitlines() if row.strip()]
                    if any(len(row) != 4 for row in rows):
                        raise ValueError("unexpected ps row")
                    rss, count = tree_rss(rows, child.pid)
                    observed["peakKiB"] = max(observed["peakKiB"], rss)
                    observed["maxProcesses"] = max(observed["maxProcesses"], count)
                    observed["samples"] += 1
                except (OSError, ValueError, subprocess.SubprocessError) as error:
                    observed["errors"].append(str(error))
                    return
                stopped.wait(0.02)
        sampler = threading.Thread(target=sample, daemon=True)
        timer = threading.Timer(timeout, expire)
        sampler.start()
        timer.start()
        try:
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
        except BaseException:
            terminate()
            os.wait4(child.pid, 0)
            raise
        finally:
            timer.cancel()
            stopped.set()
            sampler.join(timeout=5)
            terminate()
    wall = time.perf_counter() - started
    valid_memory = observed["samples"] > 0 and observed["maxProcesses"] > 0 and not observed["errors"]
    return {"exit": child.returncode, "timedOut": timed_out.is_set(), "wallSeconds": wall,
            "cpuUserSeconds": usage.ru_utime, "cpuSystemSeconds": usage.ru_stime,
            "sampledTreePeakKiB": observed["peakKiB"] if valid_memory else None,
            "kernelLargestProcessPeakKiB": usage.ru_maxrss / (1024 if sys.platform == "darwin" else 1),
            "memorySamples": observed["samples"], "maxObservedProcesses": observed["maxProcesses"],
            "memoryErrors": observed["errors"], "log": log.name}


def validate(plan):
    """A runnable plan must name real identities and repeated explicit argument vectors."""
    if plan.get("schema") != "jacket-doc-benchmark/1" or plan.get("repetitions", 0) < 3:
        raise ValueError("benchmark plan needs schema jacket-doc-benchmark/1 and at least three repetitions")
    if not plan.get("identities") or not plan.get("equivalence") or not plan.get("cases"):
        raise ValueError("record tool/source identities, equivalence settings and cases before execution")
    names = set()
    for case in plan["cases"]:
        if case["name"] in names:
            raise ValueError("duplicate benchmark case name")
        names.add(case["name"])
        if not isinstance(case.get("command"), list) or not case["command"] or any(not isinstance(arg, str) for arg in case["command"]):
            raise ValueError("benchmark commands are nonempty argument vectors, never shell strings")
        if not case.get("cwd") or not case.get("cacheState"):
            raise ValueError("every case needs a working directory and explicit cache state")


def run(plan, out):
    """Run trusted reviewed commands sequentially; only marked output-owned caches may be reset."""
    validate(plan)
    if out.exists():
        raise ValueError("--out must be absent; failure evidence is never overwritten")
    out.mkdir(parents=True)
    (out / ".jacket-doc-benchmark").write_text("1\n")
    def expand(value):
        return value.replace("{out}", str(out))
    metadata = {"schema": plan["schema"], "machine": {"platform": platform.platform(), "python": sys.version,
                "cpuCount": os.cpu_count()}, "plan": plan, "cases": []}
    (out / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    for number, case in enumerate(plan["cases"]):
        rows = []
        for repetition in range(plan["repetitions"]):
            for value in case.get("resetPaths", []):
                target = Path(expand(value)).resolve()
                if target == out or not target.is_relative_to(out):
                    raise ValueError("only benchmark-output-owned cache paths may be reset")
                if target.is_dir():
                    shutil.rmtree(target)
                elif target.exists():
                    target.unlink()
            env = {**os.environ, **{key: expand(value) for key, value in case.get("env", {}).items()}}
            cwd = Path(expand(case["cwd"]))
            for command in case.get("prepare", []):
                subprocess.run([expand(arg) for arg in command], cwd=cwd, env=env, check=True)
            log = out / f"case-{number:02}-repeat-{repetition:02}.log"
            rows.append(measure([expand(arg) for arg in case["command"]], cwd, env, log, case.get("timeoutSeconds", 1800)))
            # Write each completed sample before continuing, including failed commands.
            (out / f"case-{number:02}.json").write_text(json.dumps({"name": case["name"], "runs": rows, "summary": summarize(rows)}, indent=2) + "\n")
        metadata["cases"].append({"name": case["name"], "runs": rows, "summary": summarize(rows)})
    (out / "results.json").write_text(json.dumps(metadata, indent=2) + "\n")
    if any(not case["summary"]["allPassed"] or any(row["memoryErrors"] or row["sampledTreePeakKiB"] is None for row in case["runs"]) for case in metadata["cases"]):
        raise RuntimeError("benchmark contains failures or unavailable memory samples; inspect retained receipts")
    return metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    plan = json.loads(args.plan.read_text())
    validate(plan)
    if not args.execute:
        print(f"Validated {len(plan['cases'])} cases; no processes measured")
        return
    if not args.out:
        parser.error("--execute requires an absent owned --out directory")
    run(plan, args.out.resolve())


if __name__ == "__main__":
    main()
