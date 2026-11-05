"""Reproducible authored-value and byte-mutation differential checks on the shared profile.

    python3 differential.py --own /path/decoder --rust /path/toml-oracle --report /tmp/differential.json

Every input and outcome is retained, including disagreement or crashes. Temporal cases use
millisecond fractions, so this shared-profile check does not erase our stronger exact-digit
contract. No third-party Jai implementation is used by this check.
"""
from __future__ import annotations
import argparse
import datetime
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent


def normalized(value):
    if isinstance(value, list):
        return [normalized(item) for item in value]
    if not isinstance(value, dict):
        raise ValueError('invalid tagged value')
    if set(value) == {'type', 'value'} and isinstance(value['type'], str):
        kind, text = value['type'], value['value']
        if kind == 'float':
            number = float(text)
            return [kind, 'nan' if math.isnan(number) else struct.pack('>d', number).hex()]
        if kind == 'integer':
            return [kind, int(text)]
        if kind == 'datetime':
            text = datetime.datetime.fromisoformat(text.replace('Z', '+00:00').replace('z', '+00:00')).isoformat()
        elif kind == 'datetime-local':
            text = datetime.datetime.fromisoformat(text).isoformat()
        elif kind == 'time-local':
            text = datetime.time.fromisoformat(text).isoformat()
        return [kind, text]
    return {key: normalized(item) for key, item in value.items()}


def render(value):
    if isinstance(value, bool):
        return 'true' if value else 'false'
    if isinstance(value, (int, float)):
        return repr(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False).replace(chr(127), r'\u007f')
    if isinstance(value, list):
        return '[' + ', '.join(render(item) for item in value) + ']'
    return '{' + ', '.join(render(key) + ' = ' + render(item) for key, item in value.items()) + '}'


def generated(rng, depth=0):
    choice = rng.randrange(7 if depth < 3 else 5)
    if choice == 0:
        return rng.randrange(-(1 << 63), 1 << 63)
    if choice == 1:
        while True:
            number = struct.unpack('>d', rng.getrandbits(64).to_bytes(8, 'big'))[0]
            if math.isfinite(number):
                return number
    if choice == 2:
        return rng.choice([True, False])
    if choice == 3:
        return rng.choice(['', 'plain', 'λ😀', 'a\nb\t"\\', 'é\x00\x7f'])
    if choice == 4:
        return rng.choice([0.0, -0.0, 1.0, 1e20])
    if choice == 5:
        return [generated(rng, depth + 1) for _ in range(rng.randrange(5))]
    return {f'key_{i}': generated(rng, depth + 1) for i in range(rng.randrange(5))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--own', type=Path, required=True)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if sys.version_info < (3, 15):
        parser.error('shared TOML 1.1 differential checks require Python 3.15 or later')
    rng = random.Random(0x544f4d4c)
    valid = [b'', b'[a.b]\nx=1\n[a]\ny=2\n', b'[[rows]]\nx=1\n[[rows]]\nx=2\n', b'local = 12:34\nat = 2024-02-29T12:34:56.123+01:30\n']
    for _ in range(64):
        valid.append(('\n'.join(f'key_{i} = {render(generated(rng))}' for i in range(rng.randrange(1, 8))) + '\n').encode())
    cases = [(data, True) for data in valid]
    for _ in range(256):
        data = bytearray(rng.choice(valid[1:]))
        for _ in range(rng.randrange(1, 5)):
            data[rng.randrange(len(data))] = rng.randrange(256)
        if rng.randrange(2):
            del data[rng.randrange(len(data) + 1):]
        cases.append((bytes(data), False))
    jobs = {'own': [str(args.own.resolve())], 'rust': [str(args.rust.resolve())], 'python': [sys.executable, str(HERE / 'python_decoder.py')]}
    report = dict(seed=0x544f4d4c, own_sha256=hashlib.sha256(args.own.read_bytes()).hexdigest(), rust_sha256=hashlib.sha256(args.rust.read_bytes()).hexdigest(), python=sys.version, cases=[])
    failed = 0
    for index, (data, authored_valid) in enumerate(cases):
        row = dict(index=index, input_hex=data.hex(), authored_valid=authored_valid, outcomes={})
        for name, command in jobs.items():
            result = subprocess.run(command, input=data, capture_output=True, timeout=10)
            outcome = dict(exit=result.returncode)
            if result.returncode == 0:
                outcome['value'] = normalized(json.loads(result.stdout))
            elif result.returncode != 1:
                outcome['crash'] = True
            row['outcomes'][name] = outcome
        outcomes = list(row['outcomes'].values())
        row['agree'] = all(outcome == outcomes[0] for outcome in outcomes) and not any(outcome.get('crash') for outcome in outcomes)
        if authored_valid and outcomes[0]['exit'] != 0:
            row['authored_valid_failure'] = True
        failed += not row['agree'] or row.get('authored_valid_failure', False)
        report['cases'].append(row)
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2, ensure_ascii=True) + '\n')
    print(json.dumps(dict(cases=len(cases), disagreements=sum(not row['agree'] for row in report['cases']), authored_valid_failures=sum(row.get('authored_valid_failure', False) for row in report['cases']), exclusions=0)))
    return int(failed != 0)


if __name__ == '__main__':
    raise SystemExit(main())
