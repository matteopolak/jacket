"""Find regressions between two reports of the same tool."""
from __future__ import annotations

from dataclasses import dataclass

from results import Report, Result, Status

# A difference smaller than this many median absolute deviations is noise.
NOISE_FACTOR = 3.0


@dataclass(frozen=True)
class Limits:
    """When growth counts as a regression: more than `threshold` relative and more than the absolute
    floor, so tiny workloads do not trip on noise."""
    threshold: float = 0.10
    min_seconds: float = 0.10
    min_mib: float = 16.0


@dataclass(frozen=True)
class Regression:
    key: str
    what: str
    before: str
    after: str

    def __str__(self) -> str:
        return f'{self.key}: {self.what} {self.before} -> {self.after}'


def grew(before: float, after: float, floor: float, limits: Limits, noise: float) -> bool:
    return (before > 0 and after > before * (1 + limits.threshold) and after - before > floor
            and after - before > NOISE_FACTOR * noise)


def percent(before: float, after: float) -> str:
    return f'+{(after / before - 1) * 100:.0f}%'


def compare_result(key: str, old: Result, new: Result, limits: Limits) -> list[Regression]:
    if old.status is not Status.OK:
        return []
    if new.status is Status.FAILED:
        return [Regression(key, 'status', 'ok', f'failed ({new.reason})')]
    if new.status is not Status.OK:
        return []
    found = []
    for name, floor, unit in (('wall', limits.min_seconds, 's'), ('cpu', limits.min_seconds, 's'),
                              ('rss_mib', limits.min_mib, 'MiB')):
        a, b = getattr(old, name), getattr(new, name)
        if a and b and grew(a.median, b.median, floor, limits, max(a.mad, b.mad)):
            found.append(Regression(key, name, f'{a.median:g} {unit}', f'{b.median:g} {unit} ({percent(a.median, b.median)})'))
    found += instruction_regressions(key, old, new, limits)
    return found


def instruction_regressions(key: str, old: Result, new: Result, limits: Limits) -> list[Regression]:
    """Interpreted instruction counts are nearly deterministic, so no absolute floor or noise guard applies."""
    a, b = old.info.get('interp_instructions'), new.info.get('interp_instructions')
    if a and b and b > a * (1 + limits.threshold):
        return [Regression(key, 'interp_instructions', f'{a:,}', f'{b:,} ({percent(a, b)})')]
    return []


def find_regressions(old: Report, new: Report, limits: Limits) -> list[Regression]:
    """Every result present in both reports whose wall time, CPU time or peak RSS grew past `limits`,
    or that used to work and now fails."""
    found = []
    for key, result in new.results.items():
        before = old.results.get(key)
        if before:
            found += compare_result(key, before, result, limits)
    return found
