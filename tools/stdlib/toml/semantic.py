"""Exact comparison on the Python-representable shared benchmark/settings profile.

This is not a lossless general TOML model: benchmark temporal inputs use millisecond fractions
and exclude negative-zero offsets and leap seconds. The separate raw feature probes and core
regressions cover those stronger contracts without conversion through Python datetime.
"""
import datetime
import math
import struct
import tomllib


def semantic_equal(a, b):
    """Compare types, float bits, civil fields and offset, without instant normalization.

        assert not semantic_equal(1, True)
        assert not semantic_equal(-0.0, 0.0)
    """
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return a.keys() == b.keys() and all(semantic_equal(a[key], b[key]) for key in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(semantic_equal(x, y) for x, y in zip(a, b))
    if isinstance(a, float):
        return (math.isnan(a) and math.isnan(b)) or struct.pack('>d', a) == struct.pack('>d', b)
    if isinstance(a, datetime.datetime):
        return (a.year, a.month, a.day, a.hour, a.minute, a.second, a.microsecond, a.utcoffset()) == (b.year, b.month, b.day, b.hour, b.minute, b.second, b.microsecond, b.utcoffset())
    if isinstance(a, datetime.time):
        return (a.hour, a.minute, a.second, a.microsecond, a.utcoffset()) == (b.hour, b.minute, b.second, b.microsecond, b.utcoffset())
    return a == b


def settings_projection(value, rust):
    """Normalize override literals and the shared owned settings scope, preserving types.

        value = settings_projection(json.loads(output), rust=False)
    """
    for group in ('resolved', 'overrides'):
        value[group] = {key: tomllib.loads('value = ' + text)['value'] for key, text in value[group].items()}
    if rust:
        value['unknown'] = [key.removeprefix('format.') if key.startswith('format.') else key for key in value['unknown'] if key.startswith(('format.', 'doc.format.'))]
    return value
