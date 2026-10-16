#!/usr/bin/env python3
"""Source-only local checks, or one Linux execution with bounded resource evidence.

Does not alter concurrency, deadlines, backend selection, or a program's result.
Linux collection is for the root-owned CI gate; it refuses runtime on macOS.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / "stdlib/Extensions/Io/tests/lib/suite.jai"
ENTRY = ROOT / "tests/extensions/io-accept-burst-diagnostics.jai"
KERNEL_FILES = [
    "/proc/net/sockstat",
    "/proc/net/sockstat6",
    "/proc/net/snmp",
    "/proc/net/netstat",
    "/proc/net/dev",
    "/proc/net/softnet_stat",
    "/proc/net/stat/nf_conntrack",
    "/proc/sys/net/ipv4/ip_local_port_range",
    "/proc/sys/net/ipv4/tcp_tw_reuse",
    "/proc/sys/net/ipv4/tcp_max_syn_backlog",
    "/proc/sys/net/ipv4/tcp_syn_retries",
    "/proc/sys/net/ipv4/tcp_synack_retries",
    "/proc/sys/net/ipv4/tcp_syn_linear_timeouts",
    "/proc/sys/net/ipv4/tcp_abort_on_overflow",
    "/proc/sys/net/ipv4/tcp_syncookies",
    "/proc/sys/net/core/somaxconn",
    "/proc/sys/net/core/netdev_max_backlog",
    "/proc/sys/net/netfilter/nf_conntrack_count",
    "/proc/sys/net/netfilter/nf_conntrack_max",
    "/proc/sys/fs/file-nr",
    "/proc/loadavg",
    "/proc/pressure/cpu",
    "/proc/pressure/memory",
    "/proc/pressure/io",
    "/proc/sys/kernel/osrelease",
]


def sha(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def read_optional(path):
    try:
        return Path(path).read_text()
    except (OSError, UnicodeError) as error:
        return {"unavailable": str(error)}


def tcp_observations(text, socket_inodes):
    if not isinstance(text, str):
        return text
    result = {
        "namespace_states": {},
        "owned_socket_identity_available": socket_inodes is not None,
        "owned_states": {},
        "owned_listeners": [],
        "owned_syn_sent_examples": [],
        "owned_syn_sent_rto_total": 0,
        "malformed_rows": 0,
    }
    for line in text.splitlines()[1:]:
        fields = line.split()
        try:
            state = fields[3]
            tx, rx = (int(value, 16) for value in fields[4].split(":"))
            timer, expires = (int(value, 16) for value in fields[5].split(":"))
            rto_count = int(fields[6], 16)
            inode = int(fields[9])
        except (IndexError, ValueError):
            result["malformed_rows"] += 1
            continue
        counts = result["namespace_states"]
        counts[state] = counts.get(state, 0) + 1
        if socket_inodes is None or inode not in socket_inodes:
            continue
        counts = result["owned_states"]
        counts[state] = counts.get(state, 0) + 1
        row = {
            "inode": inode,
            "local_hex": fields[1],
            "remote_hex": fields[2],
            "tx_queue_raw": tx,
            "rx_queue_raw": rx,
            "timer_active": timer,
            "timer_expires_jiffies_raw": expires,
            "unrecovered_rto_count": rto_count,
        }
        if state == "0A":
            result["owned_listeners"].append(row)
        if state == "02":
            result["owned_syn_sent_rto_total"] += rto_count
            if len(result["owned_syn_sent_examples"]) < 16:
                result["owned_syn_sent_examples"].append(row)
    result["syn_sent_examples_truncated"] = result["owned_states"].get("02", 0) > 16
    return result


def cgroup_observations(pid):
    membership = read_optional(f"/proc/{pid}/cgroup")
    if not isinstance(membership, str):
        return membership
    result = {"membership": membership, "v2_ancestors": []}
    for line in membership.splitlines():
        if not line.startswith("0::"):
            continue
        relative = line[3:]
        if not relative.startswith("/") or ".." in relative.split("/"):
            result["unavailable"] = "unrepresentable cgroup v2 path"
            return result
        mount = Path("/sys/fs/cgroup")
        folder = mount / relative.lstrip("/")
        for _ in range(16):
            result["v2_ancestors"].append(
                {
                    "path": str(folder),
                    "files": {
                        name: read_optional(folder / name)
                        for name in [
                            "cpu.max",
                            "cpu.stat",
                            "cpu.stat.local",
                            "cpu.pressure",
                            "memory.current",
                            "memory.max",
                            "memory.events",
                        ]
                    },
                }
            )
            if folder == mount:
                return result
            folder = folder.parent
        result["ancestors_truncated"] = True
        return result
    result["unavailable"] = "no unified cgroup v2 membership; v1 not sampled"
    return result


def sample(pid):
    started = time.monotonic()
    result = {"pid": pid, "elapsed_clock": started, "unix_time_ns": time.time_ns()}
    result["namespace_kernel"] = {name: read_optional(name) for name in KERNEL_FILES}
    namespaces = {}
    for label, path in [
        ("observer", "/proc/self/ns/net"),
        ("child", f"/proc/{pid}/ns/net"),
    ]:
        try:
            namespaces[label] = os.readlink(path)
        except OSError as error:
            namespaces[label] = {"unavailable": str(error)}
    result["network_namespaces"] = namespaces
    result["namespace_kernel_matches_child"] = (
        namespaces["observer"] == namespaces["child"]
        if all(isinstance(value, str) for value in namespaces.values())
        else None
    )
    for name in ["limits", "status", "schedstat", "stat", "wchan"]:
        result[name] = read_optional(f"/proc/{pid}/{name}")
    socket_inodes = None
    try:
        descriptors = list(Path(f"/proc/{pid}/fd").iterdir())
        result["open_descriptors"] = len(descriptors)
        socket_inodes = set()
        raced = 0
        for descriptor in descriptors:
            try:
                target = os.readlink(descriptor)
                if target.startswith("socket:[") and target.endswith("]"):
                    socket_inodes.add(int(target[8:-1]))
            except (OSError, ValueError):
                raced += 1
        result["descriptor_link_reads_unavailable"] = raced
    except OSError as error:
        result["open_descriptors"] = {"unavailable": str(error)}
    # These tables cover the network namespace, not just this process. Preserve
    # that scope: TIME_WAIT sockets no longer have a live process descriptor.
    result["tcp"] = {
        name: tcp_observations(read_optional(name), socket_inodes)
        for name in [f"/proc/{pid}/net/tcp", f"/proc/{pid}/net/tcp6"]
    }
    result["cgroup"] = cgroup_observations(pid)
    result["sample_duration_seconds"] = time.monotonic() - started
    return result


def observe_process(process, receipt, save):
    started = time.monotonic()
    while process.poll() is None:
        receipt["samples"].append(sample(process.pid))
        save()
        if time.monotonic() - started >= 180:
            # Bound observation volume, not the original program's lifetime.
            # The existing CI supervisor retains its original timeout policy.
            receipt["observation_window_exhausted"] = True
            save()
            break
        time.sleep(1)
    result = process.wait()
    receipt["elapsed_seconds"] = time.monotonic() - started
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--program", type=Path)
    parser.add_argument("--jaic", type=Path)
    parser.add_argument("--jailint", type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    if args.program and platform.system() != "Linux":
        parser.error("Runtime evidence collection requires the actual Linux CI host")
    if args.check and (not args.jaic or not args.jailint):
        parser.error("Source checks require --jaic and --jailint")
    output = args.report.resolve()
    output.mkdir(parents=True, exist_ok=False)
    sources = list((ROOT / "stdlib").rglob("*.jai")) + [
        ENTRY,
        Path(__file__),
        ROOT / "tests/tools/test_io_burst_diagnostics.py",
    ]
    before = {str(p.relative_to(ROOT)): sha(p) for p in sorted(sources)}
    receipt = {
        "status": "collecting",
        "host": platform.platform(),
        "cwd": str(Path.cwd()),
        "head": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "source_sha256": before,
        "mode": "source_only" if args.check else "single_linux_diagnostic_execution",
        "commands": [],
        "samples": [],
        "environment": {
            name: os.environ.get(name)
            for name in ["JAI_IO_BACKEND", "JAI_IO_FILE_INLINE", "JAI_IO_FILE_THREADS"]
        },
    }

    def save():
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")

    save()
    if args.check:
        environment = dict(
            os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), TMPDIR=str(output)
        )
        programs = [
            ROOT / "stdlib/Extensions/Io/tests/backends.jai",
            ROOT / "stdlib/Extensions/Io/tests/tcp.jai",
            ENTRY,
        ]
        commands = []
        for target_os, cpu in [
            ("linux", "x64"),
            ("linux", "arm64"),
            ("macos", "arm64"),
        ]:
            for source in programs:
                label = f"{source.stem}-{target_os}-{cpu}"
                commands.append(
                    (
                        label,
                        [args.jaic, "-check", "-os", target_os, "-cpu", cpu, source],
                    )
                )
        commands.append(
            (
                "strict-lint",
                [
                    args.jailint,
                    "-D",
                    "warnings",
                    "--manifest",
                    output / "lint.json",
                    ROOT / "stdlib/Extensions/Io/module.jai",
                    ROOT / "stdlib/Extensions/Io/tests/backends.jai",
                    ROOT / "stdlib/Extensions/Io/tests/tcp.jai",
                    ENTRY,
                ],
            )
        )
        receipt["compiler_sha256"] = sha(args.jaic.resolve())
        receipt["linter_sha256"] = sha(args.jailint.resolve())
        for label, command in commands:
            argv = list(map(str, command))
            result = subprocess.run(
                argv,
                cwd=output,
                env=environment,
                capture_output=True,
                text=True,
                timeout=180,
            )
            (output / f"{label}.stdout").write_text(result.stdout)
            (output / f"{label}.stderr").write_text(result.stderr)
            receipt["commands"].append(
                {"label": label, "argv": argv, "exit": result.returncode}
            )
            save()
            print(f"{label}: {result.returncode}", flush=True)
        failed = any(row["exit"] != 0 for row in receipt["commands"])
    else:
        program = args.program.resolve(strict=True)
        receipt["program"] = str(program)
        receipt["program_sha256"] = sha(program)
        receipt["samples"].append(sample(os.getpid()))
        with (
            (output / "program.stdout").open("wb") as stdout,
            (output / "program.stderr").open("wb") as stderr,
        ):
            process = subprocess.Popen([str(program)], stdout=stdout, stderr=stderr)
            receipt["pid"] = process.pid
            result = observe_process(process, receipt, save)
        receipt["exit"] = result
        receipt["samples"].append(sample(os.getpid()))
        failed = result != 0
    receipt["source_changes"] = [
        name for name, expected in before.items() if sha(ROOT / name) != expected
    ]
    receipt["status"] = (
        "invalidated_source_drift"
        if receipt["source_changes"]
        else "failed"
        if failed
        else "passed_source_only"
        if args.check
        else "passed_single_linux_execution"
    )
    save()
    raise SystemExit(1 if failed or receipt["source_changes"] else 0)


if __name__ == "__main__":
    main()
