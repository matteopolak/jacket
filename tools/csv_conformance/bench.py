"""Compare the CSV benchmark input with Python csv: PYTHONPATH=tools python3 tools/csv_conformance/bench.py."""
import csv
import io
import statistics
import timeit
from dataclasses import dataclass

from repo_output import output_dir


@dataclass
class Row:
    id: int
    name: str
    value: float
    active: bool
    note: str


def records(text: str) -> int:
    reader = csv.reader(io.StringIO(text, newline=""))
    next(reader)
    return sum(len(row) for row in reader)


def typed(text: str) -> int:
    reader = csv.reader(io.StringIO(text, newline=""))
    next(reader)
    total = 0
    for row in reader:
        value = Row(int(row[0]), row[1], float(row[2]), row[3] == "true", row[4])
        total += value.id
    return total


if __name__ == "__main__":
    data = output_dir("csv-conformance", "bench.csv").read_bytes()
    text = data.decode("utf-8")
    for function in (records, typed):
        seconds = statistics.median(timeit.repeat(lambda: function(text), number=1, repeat=5))
        print(f"{function.__name__}: {len(data) / seconds / 1e6:.2f} MB/s ({seconds:.3f}s; {len(data)} bytes)")
