"""Fixed-batch Python TOML reader; input loading is outside the measured interval.

    python3 python_batch.py input.toml 100

Python tomllib supplies no writer. The full materialized value is produced each time and
released before timing stops; there is no substitute serialization task.
"""
import json
from pathlib import Path
import sys
import time
import tomllib

data = Path(sys.argv[1]).read_bytes()
iterations = int(sys.argv[2])
assert iterations > 0
started = time.perf_counter_ns()
for iteration in range(0 if 'cold' in sys.argv[3:] else -3, iterations):
    if iteration == 0:
        started = time.perf_counter_ns()
    text = data.decode('utf-8')
    if 'reject' in sys.argv[3:]:
        try:
            tomllib.loads(text)
        except tomllib.TOMLDecodeError:
            pass
        else:
            raise AssertionError('invalid benchmark input was accepted')
    else:
        value = tomllib.loads(text)
        del value
    del text
elapsed = time.perf_counter_ns() - started
print(json.dumps(dict(nanoseconds=elapsed, iterations=iterations, output_bytes=0, checksum=0)))
