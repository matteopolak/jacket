"""Compiler stages: map whatever a compiler prints about its own timing onto common buckets."""
from __future__ import annotations

import re
from enum import Enum
from typing import Protocol


class Bucket(str, Enum):
    FRONT_END = 'front end'
    COMPILE_TIME = 'compile-time execution'
    CODEGEN = 'codegen'
    LINK = 'link'
    OTHER = 'other'
    TOTAL = 'total'


Stages = dict[Bucket, float]


class StageParser(Protocol):
    """Turns the text a compiler printed into seconds per bucket. A parser that finds nothing returns {}."""

    def parse(self, output: str) -> Stages: ...


JAIC_LINE = re.compile(r'^jaic-timing: (?P<phase>.+) (?P<seconds>\d+(?:\.\d+)?) (?P<calls>\d+)$', re.MULTILINE)


class JaicTimings:
    """`jaic -timings`: one `jaic-timing: <phase> <seconds> <calls>` line per phase.

    `front end` wraps parsing, checking and (for a metaprogram) everything the metaprogram builds, and
    `interp` is the time compile-time code ran, which overlaps it. So the front end bucket is
    `front end - interp`, and compile-time execution is `interp + run` (`run` is the program itself under
    `-interp`). `workspaces` is the time a metaprogram's other workspaces took after its own front end, which
    for a check is checking them, so it counts with the front end. The backend phases (`prepare output`, `codegen`, `debug info`) are codegen. For a
    metaprogram that builds workspaces, backend time is also inside its `front end` and `interp` phases, so
    buckets may add up to more than the total; `other` never goes negative."""

    CODEGEN_PHASES = ('prepare output', 'codegen', 'debug info')

    def parse(self, output: str) -> Stages:
        phases = {m['phase']: float(m['seconds']) for m in JAIC_LINE.finditer(output)}
        if 'total' not in phases:
            return {}
        interp = phases.get('interp', 0.0)
        stages = {
            Bucket.FRONT_END: max(0.0, phases.get('front end', 0.0) + phases.get('workspaces', 0.0) - interp),
            Bucket.COMPILE_TIME: interp + phases.get('run', 0.0),
            Bucket.CODEGEN: sum(phases.get(name, 0.0) for name in self.CODEGEN_PHASES),
            Bucket.LINK: phases.get('link', 0.0),
        }
        stages[Bucket.TOTAL] = phases['total']
        stages[Bucket.OTHER] = max(0.0, phases['total'] - sum(v for k, v in stages.items() if k is not Bucket.TOTAL))
        return stages


def with_total(stages: Stages, wall: float) -> Stages:
    """`stages` with the total set to the wall time of the whole process, the same yardstick for every
    compiler, and `other` as what no bucket accounts for (including process start-up and exit)."""
    if not stages:
        return {Bucket.TOTAL: wall}
    result = {bucket: seconds for bucket, seconds in stages.items() if bucket not in (Bucket.TOTAL, Bucket.OTHER)}
    result[Bucket.OTHER] = max(0.0, wall - sum(result.values()))
    result[Bucket.TOTAL] = wall
    return result
