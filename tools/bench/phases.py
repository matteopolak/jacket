"""Stage timing that works the same for every compiler: build the program from a small metaprogram
(`phase_timing.jai`) that timestamps the compiler's phase messages, and read those timestamps back.

What the messages can tell apart, in order: the start of the build, `typechecked` (the last
TYPECHECKED_ALL_WE_CAN), `built`, `ready_link` (the objects are written and the compiler waits for the
link), `link` (a duration: the system linker the metaprogram itself ran, the same command for every
compiler), and `complete`. Parsing, type checking and compile-time execution all happen before the last
`typechecked` message and nothing in between marks them off from each other, so they stay one `front end`
bucket and `compile-time execution` is not reported. `codegen` runs from there to `ready_link` and includes
writing the objects. A build without a custom link command (Windows) falls back to the span between
`pre_write` and `post_write`.
"""
from __future__ import annotations

import re
from pathlib import Path

from stages import Bucket, Stages

TEMPLATE = Path(__file__).with_name('phase_timing.jai')
WRAPPER_NAME = 'phase_timing.jai'
LINK_COMMAND = Path(__file__).with_name('link_command.jai')
LINE = re.compile(r'^bench-phase: (?P<name>\w+) (?P<seconds>\d+(?:\.\d+)?(?:e[-+]?\d+)?)\s*$', re.MULTILINE)


def jai_string(text: str) -> str:
    """`text` escaped for the inside of a Jai string literal."""
    return text.replace('\\', '\\\\').replace('"', '\\"')


SETTINGS = re.compile(r'// settings:.*?// end settings\n', re.DOTALL)


def program(entry: Path, output_dir: Path, *, writes_executable: bool, optimized: bool, run_main: bool,
            import_dirs: tuple[str, ...] = ()) -> str:
    """The metaprogram's source for building `entry`: no output (check), no output plus `#run main();`
    (interp), or an executable in `output_dir`. `import_dirs` join the end of the nested workspace's module
    search path."""
    dirs = ', '.join(f'"{jai_string(directory)}"' for directory in import_dirs)
    settings = (f'// settings: rewritten by phases.py\nENTRY :: "{jai_string(str(entry))}";\n'
                f'OUTPUT_PATH :: "{jai_string(str(output_dir))}";\n'
                f'WRITES_EXECUTABLE :: {jai_bool(writes_executable)};\nOPTIMIZED :: {jai_bool(optimized)};\n'
                f'RUN_MAIN :: {jai_bool(run_main)};\n'
                f'IMPORT_DIRS :: string.[{dirs}];\n// end settings\n')
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
        code_ends = marks.get('ready_link', marks.get('pre_write', marks.get('built', front_end_ends)))
        stages = {Bucket.FRONT_END: front_end_ends, Bucket.CODEGEN: max(0.0, code_ends - front_end_ends)}
        if 'link' in marks:
            stages[Bucket.LINK] = marks['link']
        elif 'pre_write' in marks and 'post_write' in marks:
            stages[Bucket.LINK] = max(0.0, marks['post_write'] - marks['pre_write'])
        return stages
