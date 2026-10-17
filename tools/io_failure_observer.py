"""Bounded numeric evidence for original Linux children; never launches a retry.

One sampler serves a copy group. Only a real failed copy and one passed control
are retained. Namespace metrics remain explicitly separate from owned sockets.
"""

from collections import Counter, deque
import hashlib
import json
from itertools import islice
import os
from pathlib import Path
import re
import subprocess
from threading import Event, Lock, Thread
import time

from check_io_burst_diagnostics import cgroup_observations

INTERVAL = 5
HISTORY = 36
MAX_TCP_ROWS = 262144
MAX_FDS = 4096
MAX_TASKS = 32
MAX_GROUPS = 32
COUNTERS = {
    "Tcp.ActiveOpens",
    "Tcp.PassiveOpens",
    "Tcp.AttemptFails",
    "Tcp.EstabResets",
    "Tcp.RetransSegs",
    "Tcp.OutRsts",
    "TcpExt.ListenOverflows",
    "TcpExt.ListenDrops",
    "TcpExt.TCPSynRetrans",
    "TcpExt.TCPTimeouts",
}
SYSCTLS = {
    "ip_local_port_range",
    "tcp_tw_reuse",
    "tcp_max_syn_backlog",
    "tcp_syn_retries",
    "tcp_synack_retries",
    "tcp_syn_linear_timeouts",
    "tcp_abort_on_overflow",
    "tcp_syncookies",
}


def read(path):
    try:
        with Path(path).open() as source:
            if str(path) in {"/proc/net/tcp", "/proc/net/tcp6"}:
                return "".join(islice(source, MAX_TCP_ROWS + 2))
            return source.read(1024 * 1024)
    except (OSError, UnicodeError):
        return None


def numeric_lines(text):
    result = {}
    for line in (text or "").splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[1].isdigit():
            result[parts[0]] = int(parts[1])
    return result


def cpu_limit(text):
    parts = text.split() if isinstance(text, str) else []
    if (
        len(parts) != 2
        or not parts[1].isdigit()
        or not (parts[0] == "max" or parts[0].isdigit())
    ):
        return {"available": False}
    return {
        "available": True,
        "unlimited": parts[0] == "max",
        "quota_us": None if parts[0] == "max" else int(parts[0]),
        "period_us": int(parts[1]),
    }


def source_identity(root, compiler):
    sources = sorted(
        p
        for p in (root / "stdlib").rglob("*")
        if p.is_file()
        and p.suffix in {".jai", ".c", ".h"}
        and not any(part.startswith("._") for part in p.relative_to(root).parts)
    ) + [root / "tools/io_stress.py", root / "tools/io_failure_observer.py"]
    manifest = {
        str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sources
    }
    return {
        "source_sha256": hashlib.sha256(
            json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
        ).hexdigest(),
        "source_files": len(manifest),
        "compiler_sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
    }


def process_metrics(pid):
    inodes = set()
    result = {"available": True}
    try:
        descriptors = list(Path(f"/proc/{pid}/fd").iterdir())
        result["fd_count"] = len(descriptors)
        result["fd_scan_truncated"] = len(descriptors) > MAX_FDS
        races = 0
        for descriptor in descriptors[:MAX_FDS]:
            try:
                link = os.readlink(descriptor)
                match = re.fullmatch(r"socket:\[(\d+)\]", link)
                if match:
                    inodes.add(int(match[1]))
            except OSError:
                races += 1
        result["fd_link_unavailable"] = races
    except OSError:
        result["available"] = False
    try:
        result["namespace_matches_observer"] = os.readlink(
            f"/proc/{pid}/ns/net"
        ) == os.readlink("/proc/self/ns/net")
    except OSError:
        result["namespace_matches_observer"] = None
    for line in (read(f"/proc/{pid}/status") or "").splitlines():
        parts = line.split()
        if parts and parts[0] in {"Threads:", "VmRSS:", "FDSize:"}:
            if len(parts) > 1 and parts[1].isdigit():
                result[parts[0][:-1]] = int(parts[1])
    sums = [0, 0, 0]
    try:
        tasks = list(Path(f"/proc/{pid}/task").iterdir())
        unavailable = 0
        for task in tasks[:MAX_TASKS]:
            text = read(task / "schedstat")
            values = (text or "").split()
            if len(values) != 3 or not all(v.isdigit() for v in values):
                unavailable += 1
            else:
                sums = [a + int(b) for a, b in zip(sums, values)]
        result["thread_schedstat"] = {
            "enumerated": len(tasks),
            "unavailable": unavailable,
            "truncated": len(tasks) > MAX_TASKS,
            "sum": sums,
            "atomic": False,
        }
    except OSError:
        result["thread_schedstat"] = {"unavailable": True}
    raw = cgroup_observations(pid)
    result["cgroup_available"] = bool(raw.get("v2_ancestors"))
    result["cgroup_ancestors"] = [
        {
            "cpu_limit": cpu_limit(row["files"].get("cpu.max")),
            "cpu_stat_available": isinstance(row["files"].get("cpu.stat"), str),
            "memory_events_available": isinstance(
                row["files"].get("memory.events"), str
            ),
            "cpu_stat": numeric_lines(
                row["files"].get("cpu.stat")
                if isinstance(row["files"].get("cpu.stat"), str)
                else None
            ),
            "memory_events": numeric_lines(
                row["files"].get("memory.events")
                if isinstance(row["files"].get("memory.events"), str)
                else None
            ),
        }
        for row in raw.get("v2_ancestors", [])
    ]
    return result, inodes


def tcp_metrics(text, ownership):
    """Parse once per namespace table; never retain endpoints or inode values."""
    states = Counter()
    owned = {
        copy: {"states": Counter(), "listener_rx": [], "syn_rto": 0}
        for copy in ownership
    }
    by_inode = {}
    for copy, inodes in ownership.items():
        for inode in inodes:
            by_inode.setdefault(inode, []).append(copy)
    malformed = 0
    lines = (text or "").splitlines()[1:]
    for line in lines[:MAX_TCP_ROWS]:
        fields = line.split()
        try:
            state = fields[3]
            rx = int(fields[4].split(":")[1], 16)
            rto = int(fields[6], 16)
            inode = int(fields[9])
            if not re.fullmatch(r"[0-9A-F]{2}", state):
                raise ValueError
        except (IndexError, ValueError):
            malformed += 1
            continue
        states[state] += 1
        for copy in by_inode.get(inode, ()):
            row = owned[copy]
            row["states"][state] += 1
            if state == "0A":
                row["listener_rx"].append(rx)
            if state == "02":
                row["syn_rto"] += rto
    return {
        "available": text is not None,
        "namespace_states": dict(states),
        "owned": owned,
        "malformed_rows": malformed,
        "truncated": len(lines) > MAX_TCP_ROWS,
    }


def namespace_metrics():
    result = {"counters": {}, "sysctls": {}, "pressure": {}}
    result["scope"] = {
        "tcp_counters": "observer network namespace; includes sibling processes",
        "pressure_softnet": "host/system; not process attribution",
    }
    for name in ("snmp", "netstat"):
        lines = (read("/proc/net/" + name) or "").splitlines()
        for header, values in zip(lines[::2], lines[1::2]):
            keys, numbers = header.split(), values.split()
            for key, value in zip(keys[1:], numbers[1:]):
                full = keys[0].rstrip(":") + "." + key
                if full in COUNTERS and value.isdigit():
                    result["counters"][full] = int(value)
    paths = {name: "/proc/sys/net/ipv4/" + name for name in SYSCTLS}
    paths.update(
        {
            "somaxconn": "/proc/sys/net/core/somaxconn",
            "nf_conntrack_count": "/proc/sys/net/netfilter/nf_conntrack_count",
            "nf_conntrack_max": "/proc/sys/net/netfilter/nf_conntrack_max",
        }
    )
    for name, path in paths.items():
        values = (read(path) or "").split()
        result["sysctls"][name] = (
            list(map(int, values))
            if values and all(v.isdigit() for v in values)
            else None
        )
    for name in ("cpu", "memory", "io"):
        entries = {}
        for line in (read("/proc/pressure/" + name) or "").splitlines():
            parts = line.split()
            if not parts or parts[0] not in {"some", "full"}:
                continue
            values = {}
            for pair in parts[1:]:
                key, _, value = pair.partition("=")
                if key in {"avg10", "avg60", "avg300", "total"}:
                    try:
                        values[key] = float(value) if key != "total" else int(value)
                    except ValueError:
                        pass
            entries[parts[0]] = values
        result["pressure"][name] = entries
    softnet = (read("/proc/net/softnet_stat") or "").splitlines()
    totals = [0, 0]
    bad = 0
    for line in softnet[:4096]:
        try:
            parts = line.split()
            totals = [totals[0] + int(parts[1], 16), totals[1] + int(parts[2], 16)]
        except (IndexError, ValueError):
            bad += 1
    result["softnet"] = {
        "available": bool(softnet),
        "dropped": totals[0],
        "time_squeeze": totals[1],
        "malformed": bad,
        "truncated": len(softnet) > 4096,
    }
    text = read("/proc/net/stat/nf_conntrack")
    lines = (text or "").splitlines()
    drops = Counter()
    malformed = 0
    if lines:
        columns = lines[0].split()
        for line in lines[1:4097]:
            values = line.split()
            try:
                if len(values) != len(columns):
                    raise ValueError
                for key, value in zip(columns, values):
                    if key in {
                        "insert_failed",
                        "drop",
                        "early_drop",
                        "error",
                        "invalid",
                    }:
                        drops[key] += int(value, 16)
            except ValueError:
                malformed += 1
    result["conntrack_stat"] = {
        "available": text is not None,
        "counters": dict(drops),
        "malformed": malformed,
        "truncated": len(lines) > 4097,
    }
    return result


def burst_numbers(stderr):
    result = {}
    match = re.search(
        r"^Io burst failure: backend=(epoll|io_uring|kqueue|iocp|ioring) "
        r"round=(\d+) elapsed_ms=(\d+) accepted_total=(\d+) handlers_finished_total=(\d+)$",
        stderr,
        re.MULTILINE,
    )
    if match:
        result["backend"] = match[1]
        result.update(
            zip(
                ("round", "elapsed_ms", "accepted", "handlers_finished"),
                map(int, match.groups()[1:]),
            )
        )
    for side, fields in (
        ("client", ("connect", "write", "read")),
        ("server", ("accept", "read", "write")),
    ):
        pattern = (
            "^Io burst "
            + side
            + ": "
            + " ".join(name + r"=(\d+) first=[A-Z_]+ code=(-?\d+)" for name in fields)
            + "$"
        )
        match = re.search(pattern, stderr, re.MULTILINE)
        if match:
            values = list(map(int, match.groups()))
            result[side] = {
                name: {"count": values[2 * i], "code": values[2 * i + 1]}
                for i, name in enumerate(fields)
            }
    return result


class FailureGroup:
    def __init__(self, report, identity):
        self.report = report
        self.identity = identity
        self.records = {}
        self.lock = Lock()
        self.stop = Event()
        self.started = time.monotonic()
        self.errors = 0
        self.thread = Thread(target=self._observe)
        self.thread.start()

    def _observe(self):
        for _ in range(HISTORY):
            if self.stop.wait(INTERVAL):
                break
            try:
                self.sample()
            except Exception:
                self.errors += 1

    def sample(self):
        started = time.monotonic()
        with self.lock:
            active = {
                i: r["pid"]
                for i, r in self.records.items()
                if r["exit_phase"] == "running"
            }
        if not active:
            return
        resources, ownership = {}, {}
        for copy, pid in active.items():
            metrics, inodes = process_metrics(pid)
            resources[copy] = metrics
            if (
                metrics.get("namespace_matches_observer") is True
                and metrics.get("available") is True
            ):
                ownership[copy] = inodes
        tables = [
            tcp_metrics(read("/proc/net/" + name), ownership)
            for name in ("tcp", "tcp6")
        ]
        namespace = namespace_metrics()
        duration = time.monotonic() - started
        with self.lock:
            for copy, metrics in resources.items():
                self.records[copy]["samples"].append(
                    {
                        "elapsed_seconds": started - self.started,
                        "sample_duration_seconds": duration,
                        "process": metrics,
                        "tcp": [
                            {k: v for k, v in table.items() if k != "owned"}
                            | {"owned": table["owned"].get(copy)}
                            for table in tables
                        ],
                        "namespace": namespace,
                    }
                )

    def run(self, copy, program, directory, timeout):
        # Same child argv/cwd/pipes/timeout as the original supervisor. The sampler
        # neither polls for a replacement launch nor kills a slow process.
        started = time.monotonic()
        try:
            child = subprocess.Popen(
                [str(program)],
                cwd=directory,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                errors="replace",
            )
        except OSError as error:
            with self.lock:
                self.records[copy] = {
                    "pid": None,
                    "exit_phase": "launch_error",
                    "exit": None,
                    "launch_errno": error.errno,
                    "samples": deque(maxlen=HISTORY),
                    "elapsed_seconds": time.monotonic() - started,
                    "supervisor_timeout_seconds": timeout,
                }
            raise
        with child as process:
            with self.lock:
                self.records[copy] = {
                    "pid": process.pid,
                    "exit_phase": "running",
                    "samples": deque(maxlen=HISTORY),
                }
            phase = "runtime_exit"
            stdout = stderr = ""
            try:
                stdout, stderr = process.communicate(timeout=timeout)
            except subprocess.TimeoutExpired as error:
                phase = "supervisor_timeout"
                process.kill()
                stdout, stderr = process.communicate()
                error.output, error.stderr = stdout, stderr
                raise
            finally:
                with self.lock:
                    self.records[copy].update(
                        {
                            "exit_phase": phase,
                            "exit": process.returncode,
                            "elapsed_seconds": time.monotonic() - started,
                            "supervisor_timeout_seconds": timeout,
                            "burst": burst_numbers(stderr),
                        }
                    )
            return subprocess.CompletedProcess(
                [str(program)], process.returncode, stdout, stderr
            )

    def finish(self, failed_copies):
        self.stop.set()
        self.thread.join()
        if not failed_copies:
            return
        failed = failed_copies[0]
        passed = next(
            (
                i
                for i, row in sorted(self.records.items())
                if i not in failed_copies and row.get("exit") == 0
            ),
            None,
        )
        selected = [failed] + ([passed] if passed is not None else [])
        self.report.mkdir(parents=True, exist_ok=True)
        path = self.report / "receipt.json"
        data = (
            json.loads(path.read_text())
            if path.exists()
            else {"version": 1, "groups": []}
        )
        if len(data["groups"]) >= MAX_GROUPS:
            data["unretained_failed_groups"] = (
                data.get("unretained_failed_groups", 0) + 1
            )
            path.write_text(json.dumps(data, indent=2) + "\n")
            return
        data["groups"].append(
            {
                "identity": self.identity,
                "failed_copies": failed_copies,
                "observer_errors": self.errors,
                "sample_interval_seconds": INTERVAL,
                "sample_limit": HISTORY,
                "namespace_attribution": "namespace only, includes sibling load",
                "copies": [
                    {
                        **self.records[i],
                        "copy": i,
                        "role": "failed" if i == failed else "passed_control",
                        "samples": list(self.records[i]["samples"]),
                    }
                    for i in selected
                ],
            }
        )
        temporary = path.with_suffix(".tmp")
        temporary.write_text(json.dumps(data, indent=2) + "\n")
        temporary.replace(path)
