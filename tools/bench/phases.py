"""Stage timing that works the same for every compiler: build the program from a small metaprogram
(`phase_timing.jai`) that timestamps the compiler's phase messages, and read those timestamps back.

What the messages can tell apart, in order: the start of the build, `typechecked` (the last
TYPECHECKED_ALL_WE_CAN), `built` (target code generated), `pre_write` and `post_write` (around writing
the object files and linking), and `complete`. Parsing, type checking and compile-time execution all
happen before the last `typechecked` message and nothing in between marks them off from each other, so
they stay one `front end` bucket and `compile-time execution` is not reported. Writing objects and
linking sit between `pre_write` and `post_write` and cannot be told apart either; that span is `link`.
"""
from __future__ import annotations

import re
from pathlib import Path

from stages import Bucket, Stages

TEMPLATE = Path(__file__).with_name('phase_timing.jai')
WRAPPER_NAME = 'phase_timing.jai'
LINE = re.compile(r'^bench-phase: (?P<name>\w+) (?P<seconds>\d+(?:\.\d+)?(?:e[-+]?\d+)?)\s*$', re.MULTILINE)


def jai_string(text: str) -> str:
    """`text` escaped for the inside of a Jai string literal."""
    return text.replace('\\', '\\\\').replace('"', '\\"')


SETTINGS = re.compile(r'// settings:.*?// end settings\n', re.DOTALL)


def program(entry: Path, output_dir: Path, *, writes_executable: bool, optimized: bool, run_main: bool) -> str:
    """The metaprogram's source for building `entry`: no output (check), no output plus `#run main();`
    (interp), or an executable in `output_dir`."""
    settings = (f'// settings: rewritten by phases.py\nENTRY :: "{jai_string(str(entry))}";\n'
                f'OUTPUT_PATH :: "{jai_string(str(output_dir))}";\n'
                f'WRITES_EXECUTABLE :: {jai_bool(writes_executable)};\nOPTIMIZED :: {jai_bool(optimized)};\n'
                f'RUN_MAIN :: {jai_bool(run_main)};\n// end settings\n')
    return SETTINGS.sub(lambda _: settings, TEMPLATE.read_text(), count=1)


def jai_bool(value: bool) -> str:
    return 'true' if value else 'false'


class PhaseTiming:
    """Reads `bench-phase: <name> <seconds>` lines into buckets. The other bucket and the total come
    from `stages.with_total`, which knows the wall time of the run."""

    def parse(self, output: str) -> Stages:
        marks: dict[str, float] = {}
        for match in LINE.finditer(output):
            marks[match['name']] = float(match['seconds'])  # a repeated phase counts at its last message
        if 'complete' not in marks:
            return {}
        front_end_ends = marks.get('typechecked', marks.get('parsed', 0.0))
        write_starts = marks.get('pre_write')
        code_ends = write_starts if write_starts is not None else marks.get('built', front_end_ends)
        stages = {Bucket.FRONT_END: front_end_ends, Bucket.CODEGEN: max(0.0, code_ends - front_end_ends)}
        if write_starts is not None and 'post_write' in marks:
            stages[Bucket.LINK] = max(0.0, marks['post_write'] - write_starts)
        return stages
