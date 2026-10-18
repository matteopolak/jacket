"""How to ask each kind of compiler for a mode, and how to read its timing output.

`jaic` is the dialect of this repository's compiler. Any other compiler is assumed to take the
command line of the reference implementation. To support another one, subclass `Dialect`.
"""
from __future__ import annotations

import re
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from measure import Run
from stages import Bucket, JaicTimings, LabelledSeconds, StageParser, Stages, with_total


class Mode(str, Enum):
    CHECK = 'check'          # parse, check and run compile-time code; write nothing
    BUILD = 'build'          # the default build: an unoptimized executable
    OPTIMIZED = 'optimized'  # `-o`
    INTERP = 'interp'        # run the program inside the compiler

    @property
    def writes_executable(self) -> bool:
        return self in (Mode.BUILD, Mode.OPTIMIZED)


@dataclass(frozen=True)
class Request:
    """What to compile: `entry` (a file in `workdir`), the mode, extra arguments for the build metaprogram
    (after a lone `-`), and where scratch files and the executable go."""
    entry: Path
    workdir: Path
    mode: Mode
    metaprogram_args: tuple[str, ...]
    scratch: Path
    redirect_output: bool = True

    @property
    def executable(self) -> Path:
        return self.scratch / 'program'


@dataclass(frozen=True)
class Invocation:
    command: list[str]
    cwd: Path
    executable: Path | None = None  # what a successful build writes


class Dialect(ABC):
    name: str

    @abstractmethod
    def invocation(self, compiler: Path, request: Request) -> Invocation: ...

    @abstractmethod
    def stages(self, run: Run) -> Stages:
        """Seconds per bucket from one finished run (always including the total)."""

    def instruction_count(self, run: Run) -> int | None:
        """Interpreted instructions, for a compiler that can count them (run with `profiling_env`)."""
        return None

    profiling_env: dict[str, str] = {}


class JaicDialect(Dialect):
    name = 'jaic'
    MODE_FLAGS = {Mode.CHECK: ['-check'], Mode.BUILD: [], Mode.OPTIMIZED: ['-o'], Mode.INTERP: ['-interp']}

    def __init__(self) -> None:
        self.parser: StageParser = JaicTimings()

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        command = [str(compiler), request.entry.name, *self.MODE_FLAGS[request.mode], '-timings']
        executable = request.executable if request.mode.writes_executable else None
        if executable and request.redirect_output:
            command += ['-exe', str(executable)]
        if request.metaprogram_args:
            command += ['-', *request.metaprogram_args]
        return Invocation(command, request.workdir, executable if request.redirect_output else None)

    def stages(self, run: Run) -> Stages:
        return with_total(self.parser.parse(run.stderr), run.wall)

    # `JAIC_PROFILE=1` makes the interpreter count IR instructions. The count barely varies between runs,
    # so it shows small interpreter changes that wall time hides.
    profiling_env = {'JAIC_PROFILE': '1'}
    PROFILE_LINE = re.compile(r'interpreter profile: (\d+) instructions')

    def instruction_count(self, run: Run) -> int | None:
        match = self.PROFILE_LINE.search(run.stderr)
        return int(match[1]) if match else None


# What the reference prints about its own timing, mapped onto buckets (substring of the label, bucket).
REFERENCE_LABELS = [
    ('front', Bucket.FRONT_END), ('parse', Bucket.FRONT_END), ('check', Bucket.FRONT_END),
    ('run', Bucket.COMPILE_TIME), ('meta', Bucket.COMPILE_TIME),
    ('llvm', Bucket.CODEGEN), ('gener', Bucket.CODEGEN), ('codegen', Bucket.CODEGEN),
    ('link', Bucket.LINK),
]

# Compiles `entry` without writing anything; in interpreter mode also runs `main` at compile time.
WRAPPER = '''#import "Compiler";
#run set_build_options_dc(.{{do_output=false}});
#load "{entry}";
{run}'''


class ReferenceDialect(Dialect):
    """The reference compiler has no check or interpreter mode. Check mode compiles the program inside a
    wrapper file whose build options turn output off, and interpreter mode adds `#run main();`, so both
    measure the front end and compile-time execution of the real program."""
    name = 'reference'

    def __init__(self) -> None:
        self.parser: StageParser = LabelledSeconds(REFERENCE_LABELS)

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        if request.mode in (Mode.CHECK, Mode.INTERP):
            wrapper = request.scratch / 'wrapper.jai'
            wrapper.write_text(WRAPPER.format(entry=request.entry.resolve(),
                                              run='#run main();\n' if request.mode is Mode.INTERP else ''))
            return Invocation([str(compiler), wrapper.name, '-no_color'], request.scratch)
        command = [str(compiler), request.entry.name, '-no_color']
        if request.mode is Mode.OPTIMIZED:
            command.append('-o')
        executable = request.executable
        if request.redirect_output:
            command += ['-exe', executable.name, '-output_path', str(executable.parent)]
        if request.metaprogram_args:
            command += ['-', *request.metaprogram_args]
        return Invocation(command, request.workdir, executable if request.redirect_output else None)

    def stages(self, run: Run) -> Stages:
        return with_total(self.parser.parse(run.stdout + '\n' + run.stderr), run.wall)


@dataclass(frozen=True)
class Compiler:
    name: str
    path: Path
    dialect: Dialect

    def describe(self) -> dict:
        return {'path': str(self.path), 'dialect': self.dialect.name}


def parse_compiler(spec: str) -> Compiler:
    """`NAME=PATH`. A binary called `jaic*` speaks the jaic dialect, anything else the reference's."""
    name, separator, path = spec.partition('=')
    if not separator or not name or not path:
        raise ValueError(f'--compiler wants NAME=PATH, got {spec!r}')
    binary = Path(path).expanduser().resolve()
    dialect: Dialect = JaicDialect() if binary.name.startswith('jaic') else ReferenceDialect()
    return Compiler(name, binary, dialect)
