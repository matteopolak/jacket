"""Summary statistics for repeated measurements."""
from __future__ import annotations

import statistics
from dataclasses import asdict, dataclass


def percentile(values: list[float], fraction: float) -> float:
    """Linear-interpolated percentile of `values` (not necessarily sorted); `fraction` is 0..1."""
    if not values:
        raise ValueError('percentile of no values')
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    low = int(position)
    high = min(low + 1, len(ordered) - 1)
    return ordered[low] + (ordered[high] - ordered[low]) * (position - low)


def median_absolute_deviation(values: list[float]) -> float:
    centre = statistics.median(values)
    return statistics.median(abs(v - centre) for v in values)


@dataclass(frozen=True)
class Stat:
    """The median of `n` runs and how far they spread: min, max, median absolute deviation, p95."""
    n: int
    median: float
    min: float
    max: float
    mad: float
    p95: float

    @classmethod
    def of(cls, values: list[float]) -> Stat:
        if not values:
            raise ValueError('no values to summarize')
        return cls(n=len(values), median=statistics.median(values), min=min(values), max=max(values),
                   mad=median_absolute_deviation(values), p95=percentile(values, 0.95))

    @classmethod
    def from_json(cls, data: dict) -> Stat:
        return cls(**data)

    def to_json(self) -> dict:
        return {key: round(value, 6) if isinstance(value, float) else value for key, value in asdict(self).items()}


def ratio(a: float, b: float) -> float | None:
    """a / b, or None when b is zero."""
    return a / b if b else None
