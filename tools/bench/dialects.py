"""How to ask each compiler binary for a mode, and how to read its timing output.

Two dialects, chosen by probing the binary's `-help` (`detect_dialect`), never by its file name:

- `jaic`: a binary with jaic's own mode and reporting flags (`-check`, `-interp`, `-timings`, `-profile`).
- `plain`: any other binary. It gets only the portable command line every Jai compiler takes (the file,
  `-o`, `-exe`, `-output_path`, `-import_dir`, `-no_color`). Check and interpreter modes compile the
  program from a wrapper file whose build options turn output off (plus `#run main();` for the
  interpreter), so they measure the front end and compile-time execution of the real program. It is
  timed from the outside only.

To support a different command line, subclass `Dialect`.
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from bench.measure import Run
from bench import phases
from bench.stages import JaicTimings, StageParser, Stages, with_total


# Executables carry this suffix on Windows. Compilers add it to a name that has none (`-exe program`),
# so command lines name the program without it and paths to the file are built with it.
EXE_SUFFIX = '.exe' if os.name == 'nt' else ''


def executable_file(folder: Path, name: str = 'program') -> Path:
    return folder / (name + EXE_SUFFIX)


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
        return executable_file(self.scratch)

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
        executable = executable_file(where) if request.mode.writes_executable else None
        return Invocation(self.source_command(compiler, phases.WRAPPER_NAME), where, executable)

    def instruction_count(self, run: Run) -> int | None:
        """Interpreted instructions, for a compiler that can count them (run with `profiling_flags`)."""
        return None

    profiling_flags: list[str] = []


class JaicDialect(Dialect):
    name = 'jaic'
    MODE_FLAGS = {Mode.CHECK: ['-check'], Mode.BUILD: [], Mode.OPTIMIZED: ['-o'], Mode.INTERP: ['-interp']}

    REPORT_FLAGS = ['-timings']

    def __init__(self) -> None:
        self.parser: StageParser = JaicTimings()

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        command = [str(compiler), request.entry_arg, *self.MODE_FLAGS[request.mode], *self.REPORT_FLAGS,
                   *self.import_flags(request.import_dirs)]
        executable = request.executable if request.mode.writes_executable else None
        if executable and request.redirect_output:
            command += ['-exe', str(executable.with_suffix(''))]
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


WRAPPER = '''#import "Compiler";
#run set_build_options_dc(.{{do_output=false}});
#load "{entry}";
{run}'''


class PlainDialect(Dialect):
    """A binary without jaic's own flags: only the portable command line, no per-phase detail or
    instruction count, only what is measured from outside."""
    name = 'plain'

    def invocation(self, compiler: Path, request: Request) -> Invocation:
        if request.mode in (Mode.CHECK, Mode.INTERP):
            wrapper = request.scratch / 'wrapper.jai'
            wrapper.parent.mkdir(parents=True, exist_ok=True)
            wrapper.write_text(WRAPPER.format(entry=request.entry.resolve().as_posix(),
                                              run='#run main();\n' if request.mode is Mode.INTERP else ''))
            command = [str(compiler), wrapper.name, '-no_color', *self.import_flags(request.import_dirs)]
            return Invocation(command, request.scratch)
        command = [str(compiler), request.entry_arg, '-no_color', *self.import_flags(request.import_dirs)]
        if request.mode is Mode.OPTIMIZED:
            command.append('-o')
        executable = request.executable
        if request.redirect_output:
            command += ['-exe', executable.stem, '-output_path', str(executable.parent)]
        if request.metaprogram_args:
            command += ['-', *request.metaprogram_args]
        return Invocation(command, request.workdir, executable if request.redirect_output else None)

    def source_command(self, compiler: Path, source: str) -> list[str]:
        return [str(compiler), source, '-no_color']

    def import_flags(self, import_dirs: tuple[str, ...]) -> list[str]:
        return [flag for directory in import_dirs for flag in ('-import_dir', directory)]

    def detail(self, run: Run) -> Stages:
        return {}

    def instruction_count(self, run: Run) -> int | None:
        return None


@dataclass(frozen=True)
class Compiler:
    name: str
    path: Path
    dialect: Dialect

    def describe(self) -> dict:
        return {'path': str(self.path), 'dialect': self.dialect.name}


def supports_reporting(binary: Path) -> bool:
    """Whether `binary -help` lists the `-timings` flag. A binary that cannot be run or prints no help
    is treated as plain."""
    try:
        done = subprocess.run([str(binary), '-help'], capture_output=True, text=True, timeout=30,
                              stdin=subprocess.DEVNULL)
    except (OSError, subprocess.TimeoutExpired):
        return False
    return '-timings' in done.stdout + done.stderr


def detect_dialect(binary: Path) -> Dialect:
    return JaicDialect() if supports_reporting(binary) else PlainDialect()


def parse_compiler(spec: str) -> Compiler:
    """`NAME=PATH`; the dialect comes from probing the binary (`detect_dialect`), not from its name."""
    name, separator, path = spec.partition('=')
    if not separator or not name or not path:
        raise ValueError(f'--compiler wants NAME=PATH, got {spec!r}')
    binary = Path(path).expanduser().resolve()
    return Compiler(name, binary, detect_dialect(binary) if binary.exists() else PlainDialect())
