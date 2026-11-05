#!/usr/bin/env python3
"""Black-box community-library interoperability adapter, not a pure parser/writer result.

Decoder path: community parse+write -> Python tagged conversion. Encoder path: own semantic
builder/writer -> community parse+write. The independent runner supplies expected values.
All community execution is sandboxed. Keep failures attributable to this complete pipeline.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tomllib

from bench.sandbox import Sandbox, environment
from python_decoder import tagged

REJECT_MARKER = b'TOML_READ_REJECTED\n'
HERE = Path(__file__).resolve().parent


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--community', type=Path, required=True)
    parser.add_argument('--own-adapter', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--encode', action='store_true')
    parser.add_argument('--float-mode', choices=('default', 'shortest', 'scientific', 'scientific-compact'), default='default')
    args = parser.parse_args()
    if sys.platform not in ('darwin', 'linux'):
        parser.error('community subprocess comparison requires the macOS/Linux bench Sandbox launcher')
    data = sys.stdin.buffer.read()
    if args.encode:
        result = subprocess.run([str(args.own_adapter), 'encode'], input=data, capture_output=True, timeout=10)
        if result.returncode:
            return 70
        data = result.stdout
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    executable = args.community.resolve()
    sandbox = Sandbox(reads=[executable.parent], writes=[scratch])
    command = [str(executable), *([args.float_mode] if args.float_mode != 'default' else [])]
    result = subprocess.run(sandbox.wrap(command), input=data, capture_output=True, env=environment(scratch), timeout=10)
    if result.returncode:
        # Diagnostics might contain implementation source: do not expose them.
        if result.returncode == 1:
            return 1 if result.stderr.endswith(REJECT_MARKER) else 70
        return result.returncode if result.returncode > 0 else 128 - result.returncode
    if args.encode:
        sys.stdout.buffer.write(result.stdout)
        return 0
    try:
        value = tomllib.loads(result.stdout.decode('utf-8'))
    except (tomllib.TOMLDecodeError, UnicodeDecodeError, ValueError):
        return 70
    json.dump(tagged(value), sys.stdout, ensure_ascii=True)
    return 0


def cli_main():
    """Unexpected failures/timeouts never masquerade as the protocol's read rejection (1)."""
    try:
        return main()
    except Exception as error:
        print('unexpected interoperability failure: ' + type(error).__name__, file=sys.stderr)
        return 70


if __name__ == '__main__':
    raise SystemExit(cli_main())
