#!/usr/bin/env python3
"""toml-test tagged decoder for the installed Python tomllib (no writer supplied).

The runner must record the Python interpreter version; TOML 1.1 requires Python 3.15 or later.
"""
from __future__ import annotations
import datetime
import json
import sys
import tomllib


def tagged(value):
    if isinstance(value, dict):
        return {key: tagged(item) for key, item in value.items()}
    if isinstance(value, list):
        return [tagged(item) for item in value]
    if isinstance(value, bool):
        kind, text = 'bool', str(value).lower()
    elif isinstance(value, int):
        kind, text = 'integer', str(value)
    elif isinstance(value, float):
        kind, text = 'float', repr(value)
    elif isinstance(value, str):
        kind, text = 'string', value
    elif isinstance(value, datetime.datetime):
        kind = 'datetime-local' if value.tzinfo is None else 'datetime'
        text = value.isoformat()
    elif isinstance(value, datetime.date):
        kind, text = 'date-local', value.isoformat()
    elif isinstance(value, datetime.time):
        kind, text = 'time-local', value.isoformat()
    else:
        raise TypeError(type(value).__name__)
    return {'type': kind, 'value': text}


def main() -> int:
    try:
        data = tomllib.loads(sys.stdin.buffer.read().decode('utf-8'))
    except (tomllib.TOMLDecodeError, UnicodeDecodeError, ValueError):
        return 1
    json.dump(tagged(data), sys.stdout, ensure_ascii=True)
    return 0


def cli_main():
    """Keep unexpected failures distinct from the protocol's expected parse rejection (1)."""
    try:
        return main()
    except Exception as error:
        print('unexpected decoder failure: ' + type(error).__name__, file=sys.stderr)
        return 70


if __name__ == '__main__':
    raise SystemExit(cli_main())
