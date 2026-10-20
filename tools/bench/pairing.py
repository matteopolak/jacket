"""Fair ordering and ratio statistics for timing two binaries against each other."""
from __future__ import annotations

import random
import statistics

MIN_PAIRS = 4
BOOTSTRAPS = 2000


def interleave(names: list[str], rounds: int, seed: int, shuffle: bool = True) -> list[list[str]]:
    """The order in which the binaries run in each round. With `shuffle`, a random order for round 1, its
    reverse for round 2, a new random order for round 3, and so on, so each binary runs first as often as
    last (A B / B A / B A / A B ...). Without, the same order every round (A B A B ...)."""
    rng = random.Random(seed)
    orders: list[list[str]] = []
    while len(orders) < rounds:
        order = list(names)
        if shuffle:
            rng.shuffle(order)
        orders.append(order)
        if shuffle and len(orders) < rounds:
            orders.append(order[::-1])
    return orders


def bootstrap_interval(values: list[float], seed: int, confidence: float = 0.95) -> tuple[float, float]:
    """A percentile bootstrap interval for the median of `values`."""
    rng = random.Random(seed)
    medians = sorted(statistics.median(rng.choices(values, k=len(values))) for _ in range(BOOTSTRAPS))
    tail = (1 - confidence) / 2
    return medians[int(tail * BOOTSTRAPS)], medians[min(BOOTSTRAPS - 1, int((1 - tail) * BOOTSTRAPS))]


def ratio_summary(baseline: list[float], candidate: list[float], seed: int = 0) -> dict:
    """candidate / baseline from the paired rounds: the median of the per-round ratios, a bootstrap 95%
    interval for it, and a verdict. The interval excluding 1 is the significance test: with a quiet
    machine and enough rounds, a median ratio whose interval spans 1 is not a difference."""
    pairs = [c / b for b, c in zip(baseline, candidate) if b > 0]
    if len(pairs) < MIN_PAIRS:
        return {'pairs': len(pairs), 'ratio': statistics.median(pairs) if pairs else None, 'low': None,
                'high': None, 'verdict': f'too few rounds (need {MIN_PAIRS})'}
    low, high = bootstrap_interval(pairs, seed)
    if high < 1:
        verdict = 'candidate faster'
    elif low > 1:
        verdict = 'candidate slower'
    else:
        verdict = 'no significant difference'
    return {'pairs': len(pairs), 'ratio': statistics.median(pairs), 'low': low, 'high': high, 'verdict': verdict}
