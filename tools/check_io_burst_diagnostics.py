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
    "/proc/sys/net/ipv4/ip_local_port_range",
    "/proc/sys/net/ipv4/tcp_tw_reuse",
    "/proc/sys/net/ipv4/tcp_max_syn_backlog",
    "/proc/sys/net/core/somaxconn",
    "/proc/sys/fs/file-nr",
    "/proc/loadavg",
    "/proc/pressure/cpu",
    "/proc/pressure/memory",
    "/proc/pressure/io",
]


def sha(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def read_optional(path):
    try:
        return Path(path).read_text()
    except OSError as error:
        return {"unavailable": str(error)}


def sample(pid):
    result = {"elapsed_clock": time.monotonic()}
    result["namespace_kernel"] = {name: read_optional(name) for name in KERNEL_FILES}
    for name in ["limits", "status", "schedstat", "stat"]:
        result[name] = read_optional(f"/proc/{pid}/{name}")
    try:
        result["open_descriptors"] = len(list(Path(f"/proc/{pid}/fd").iterdir()))
    except OSError as error:
        result["open_descriptors"] = {"unavailable": str(error)}
    # These tables cover the network namespace, not just this process. Preserve
    # that scope: TIME_WAIT sockets no longer have a live process descriptor.
    states = {}
    for filename in ["/proc/net/tcp", "/proc/net/tcp6"]:
        text = read_optional(filename)
        if not isinstance(text, str):
            states[filename] = text
            continue
        counts = {}
        for line in text.splitlines()[1:]:
            fields = line.split()
            if len(fields) >= 4:
                state = fields[3]
                counts[state] = counts.get(state, 0) + 1
        states[filename] = counts
    result["namespace_tcp_state_counts"] = states
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
    sources = list((ROOT / "stdlib").rglob("*.jai")) + [ENTRY, Path(__file__)]
    before = {str(p.relative_to(ROOT)): sha(p) for p in sorted(sources)}
    receipt = {
        "host": platform.platform(),
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
            started = time.monotonic()
            while process.poll() is None:
                receipt["samples"].append(sample(process.pid))
                save()
                if time.monotonic() - started >= 180:
                    receipt["diagnostic_timeout"] = True
                    process.kill()
                    break
                time.sleep(1)
            result = process.wait()
        receipt["exit"] = result
        receipt["elapsed_seconds"] = time.monotonic() - started
        receipt["samples"].append(sample(os.getpid()))
        failed = result != 0 or receipt.get("diagnostic_timeout", False)
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
