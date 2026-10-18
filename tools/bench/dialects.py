"""How to ask each kind of compiler for a mode, and how to read its timing output.

`jaic` is the dialect of this repository's compiler. Any other compiler is assumed to take the
command line of the reference implementation. To support another one, subclass `Dialect`.
"""
from __future__ import annotations

import re
import shutil
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from measure import Run
import phases
from stages import JaicTimings, StageParser, Stages, with_total


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
    # Module folders to search, absolute.
    import_dirs: tuple[str, ...] = ()

    @property
    def executable(self) -> Path:
        return self.scratch / 'program'

    @property
    def entry_arg(self) -> str:
        """The entry as named on a command line run from `workdir`."""
        return self.entry.relative_to(self.workdir).as_posix()


@dataclass(frozen=True)
class Invocation:
    command: list[str]
    cwd: Path
    executable: Path | None = None  # what a successful build writes


class Dialect(ABC):
    name: str
    phase_parser: StageParser = phases.PhaseTiming()

    @abstractmethod
    def invocation(self, compiler: Path, request: Request) -> Invocation: ...

    @abstractmethod
    def source_command(self, compiler: Path, source: str) -> list[str]:
        """Compile the Jai file `source` (in the working directory) as a plain build."""

    @abstractmethod
    def import_flags(self, import_dirs: tuple[str, ...]) -> list[str]:
        """The command-line option that adds each of `import_dirs` to the module search path."""

    def detail(self, run: Run) -> Stages:
        """Seconds per bucket from a timing report only this compiler prints (empty when it prints none).
        Not comparable between compilers; `phase_invocation` is."""
        return {}

    def phase_invocation(self, compiler: Path, request: Request) -> Invocation:
        """Build the request's program through `phase_timing.jai`, the same metaprogram for every compiler.
        It links with the system linker (`link_command.jai`), so the executable is the same work to link."""
        where = request.scratch / 'phases'
        where.mkdir(parents=True, exist_ok=True)
        source = phases.program(request.entry.resolve(), where, writes_executable=request.mode.writes_executable,
                                optimized=request.mode is Mode.OPTIMIZED, run_main=request.mode is Mode.INTERP,
                                import_dirs=request.import_dirs)
        (where / phases.WRAPPER_NAME).write_text(source)
        shutil.copy(phases.LINK_COMMAND, where / phases.LINK_COMMAND.name)
        executable = where / 'program' if request.mode.writes_executable else None
        return Invocation(self.source_command(compiler, phases.WRAPPER_NAME), where, executable)

    def instruction_count(self, run: Run) -> int | None:
        """Interpreted instructions, for a compiler that can count them (run with `profiling_flags`)."""
        return None

    profiling_flags: list[str] = []


class JaicDialect(Dialect):
    name = 'jaic'
    MODE_FLAGS = {Mode.CHECK: ['-check'], Mode.BUILD: [], Mode.OPTIMIZED: ['-o'], Mode.INTERP: ['-interp']}

    def __init__(self) -> None:
        self.parser: StageParser = JaicTimings()

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        command = [str(compiler), request.entry_arg, *self.MODE_FLAGS[request.mode], '-timings',
                   *self.import_flags(request.import_dirs)]
        executable = request.executable if request.mode.writes_executable else None
        if executable and request.redirect_output:
            command += ['-exe', str(executable)]
        if request.metaprogram_args:
            command += ['-', *request.metaprogram_args]
        return Invocation(command, request.workdir, executable if request.redirect_output else None)

    def source_command(self, compiler: Path, source: str) -> list[str]:
        return [str(compiler), source]

    def import_flags(self, import_dirs: tuple[str, ...]) -> list[str]:
        return [flag for directory in import_dirs for flag in ('-I', directory)]

    def detail(self, run: Run) -> Stages:
        return with_total(self.parser.parse(run.stderr), run.wall)

    # `-profile` makes the interpreter count IR instructions. The count barely varies between runs,
    # so it shows small interpreter changes that wall time hides.
    profiling_flags = ['-profile']
    PROFILE_LINE = re.compile(r'interpreter profile: (\d+) instructions')

    def instruction_count(self, run: Run) -> int | None:
        match = self.PROFILE_LINE.search(run.stderr)
        return int(match[1]) if match else None


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

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        if request.mode in (Mode.CHECK, Mode.INTERP):
            wrapper = request.scratch / 'wrapper.jai'
            wrapper.write_text(WRAPPER.format(entry=request.entry.resolve(),
                                              run='#run main();\n' if request.mode is Mode.INTERP else ''))
            command = [str(compiler), wrapper.name, '-no_color', *self.import_flags(request.import_dirs)]
            return Invocation(command, request.scratch)
        command = [str(compiler), request.entry_arg, '-no_color', *self.import_flags(request.import_dirs)]
        if request.mode is Mode.OPTIMIZED:
            command.append('-o')
        executable = request.executable
        if request.redirect_output:
            command += ['-exe', executable.name, '-output_path', str(executable.parent)]
        if request.metaprogram_args:
            command += ['-', *request.metaprogram_args]
        return Invocation(command, request.workdir, executable if request.redirect_output else None)

    def source_command(self, compiler: Path, source: str) -> list[str]:
        return [str(compiler), source, '-no_color']

    def import_flags(self, import_dirs: tuple[str, ...]) -> list[str]:
        return [flag for directory in import_dirs for flag in ('-import_dir', directory)]


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
