#!/usr/bin/env python3
"""Run one benchmark suite in a fresh resource-accounting subprocess."""
import json
import os
from pathlib import Path
import platform
import resource
import signal
import subprocess
import sys


def peak_rss_bytes(value, system):
    if system == 'Darwin':
        return int(value)
    if system == 'Linux':
        return int(value) * 1024
    raise ValueError(f'unsupported peak RSS unit on {system}')


def main():
    if len(sys.argv) < 4 or sys.argv[2] != '--':
        raise SystemExit('usage: benchmark_resources.py output.json -- executable [arguments]')
    destination = Path(sys.argv[1])
    timeout = os.environ.get('JAI_BENCH_TIMEOUT_SECONDS')
    child = subprocess.Popen(sys.argv[3:], start_new_session=True)
    try:
        exit_code = child.wait(timeout=float(timeout) if timeout else None)
        timed_out = False
    except subprocess.TimeoutExpired:
        # The fresh session belongs only to this invocation and its own children.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
        exit_code = 124
        timed_out = True
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    record = {
        'exit_code': exit_code,
        'timed_out': timed_out,
        'maximum_child_peak_rss_bytes': peak_rss_bytes(usage.ru_maxrss, platform.system()),
        'user_cpu_seconds': usage.ru_utime,
        'system_cpu_seconds': usage.ru_stime,
        'scope': 'entire suite including preparation; maximum individual waited child RSS, not simultaneous process-tree memory',
        'includes_native_compiler_preparation': True,
    }
    temporary = destination.with_suffix('.tmp')
    temporary.write_text(json.dumps(record, indent=2) + '\n')
    temporary.replace(destination)
    raise SystemExit(exit_code)


if __name__ == '__main__':
    main()
