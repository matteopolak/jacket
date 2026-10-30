"""The compile benchmark's workloads: what they exercise, where their sources come from, how to build them."""
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Protocol

import benchgen
from bench.dialects import Mode
from repo_output import host_platform, main_checkout, shared_output_dir
from bench.results import ROOT

CORPUS = main_checkout() / 'corpus' / 'upstream'
COMPILE_PROGRAMS = ROOT / 'benchmarks' / 'compile'
MICRO_PROGRAMS = ROOT / 'benchmarks'
RUNTIME_PROGRAMS = ROOT / 'benchmarks' / 'runtime'
# Entry files that make an upstream library do real work; each is copied into its project.
PROJECT_DRIVERS = ROOT / 'benchmarks' / 'runtime-projects'


# Programs worth looking for inside a `sh -c` setup script.
SCRIPT_TOOLS = ('clang', 'gcc', 'cc', 'ar', 'make', 'cmake')


class Category(str, Enum):
    STARTUP = 'startup'
    LOTS_OF_CODE = 'lots of code'
    REAL_PROJECT = 'real projects'
    COMPILE_TIME = 'compile-time heavy'
    STDLIB = 'stdlib heavy'
    INTERPRETER = 'interpreter'
    GENERATED_CODE = 'generated-code runtime'


# ----------------------------------------------------------------------------------------- sources

class Source(Protocol):
    def materialize(self, scratch: Path) -> Path:
        """The directory to compile in (writing generated files under `scratch` first)."""

    def available(self) -> str:
        """'' when the sources exist, otherwise why not."""

    def measured(self, directory: Path, entry: str) -> Path:
        """What to count lines in: the part of `directory` that is the workload's own code."""


def is_link(path: Path) -> bool:
    """A symlink, or a Windows directory junction."""
    return path.is_symlink() or (hasattr(os.path, 'isjunction') and os.path.isjunction(path))


def link_target(path: Path) -> Path:
    """The folder or file a link points to. A link recorded with an absolute path from another machine or
    mapping (a corpus checkout made on the host, read in a VM) dangles; its tail after `corpus/upstream` is
    looked up in this checkout's corpus instead."""
    target = Path(os.path.realpath(path))
    if target.exists():
        return target
    raw = os.readlink(path)
    parts = [part for part in re.split(r'[\\/]+', raw.removeprefix('\\\\?\\')) if part]
    for index in range(len(parts) - 1):
        if parts[index] == 'corpus' and parts[index + 1] == 'upstream':
            local = CORPUS.joinpath(*parts[index + 2:])
            if local.exists():
                return local
    raise FileNotFoundError(f'{path} links to {raw}, which does not exist here; '
                            'run python3 tools/fetch_upstreams.py to recreate the links')


def copy_tree(source: Path, destination: Path, active: tuple[Path, ...] = ()) -> None:
    """Copy `source` to `destination`, except `.build` and `.git`; links are copied as what they point to."""
    destination.mkdir(parents=True, exist_ok=True)
    real = Path(os.path.realpath(source))
    for entry in sorted(source.iterdir()):
        if entry.name in ('.build', '.git'):
            continue
        origin = link_target(entry) if is_link(entry) else entry
        if origin.is_dir():
            if Path(os.path.realpath(origin)) not in (*active, real):
                copy_tree(origin, destination / entry.name, (*active, real))
        else:
            shutil.copy2(origin, destination / entry.name)


def copy_in(directory: Path, scratch: Path, root: Path | None = None) -> Path:
    """A copy under `scratch` of `root` (default `directory`), made once, and where `directory` landed in
    it: the sandboxed compilers may write only there (a compiler writes `.build/` next to what it
    compiles). A project subfolder copies its whole project, which it may import from (`../module.jai`),
    and links are copied as what they point to (a corpus project links its sibling projects as modules)."""
    root = root or directory
    copy = scratch / 'sources' / root.name
    if not copy.exists():
        copy_tree(root, copy)
    return copy / directory.relative_to(root)


@dataclass(frozen=True)
class RepoDirectory:
    """A directory of this repository (benchmarks/compile)."""
    directory: Path

    def materialize(self, scratch: Path) -> Path:
        return copy_in(self.directory, scratch)

    def available(self) -> str:
        return '' if self.directory.is_dir() else f'{self.directory} is missing'

    def measured(self, directory: Path, entry: str) -> Path:
        return directory / entry  # the directory holds many unrelated programs


@dataclass(frozen=True)
class CorpusProject:
    """A pinned upstream project (`python3 tools/fetch_upstreams.py`), optionally with setup commands from
    tools/upstream-cases.json that prepare vendored libraries first."""
    project: str
    setup_ids: tuple[str, ...] = ()
    # A file copied into the project's folder (a driver from PROJECT_DRIVERS), for a library with no
    # program of its own that does enough work.
    driver: Path | None = None
    # The project links third-party libraries from `target/native-libs` (built by tools/native/build_native_libs.py).
    native_libs: bool = False

    @property
    def directory(self) -> Path:
        return CORPUS / self.project

    def materialize(self, scratch: Path) -> Path:
        for case in self.setup_cases():
            for command in case.get('setup', []):
                try:
                    subprocess.run(command, cwd=(CORPUS / case['path']).parent, capture_output=True,
                                   stdin=subprocess.DEVNULL)
                except OSError:
                    pass  # `available()` reports a missing tool; a setup that cannot start changes nothing
        copy = copy_in(self.directory, scratch, CORPUS / Path(self.project).parts[0])
        if self.driver:
            shutil.copy(self.driver, copy / self.driver.name)
        return copy

    def setup_cases(self) -> list[dict]:
        if not self.setup_ids:
            return []
        cases = json.loads((ROOT / 'tools' / 'upstream-cases.json').read_text(encoding='utf-8'))
        return [case for case in cases if case['id'] in self.setup_ids]

    def missing_tools(self) -> list[str]:
        """Programs the setup commands need that are not installed: each command's own program, and for a
        `sh -c` script the compiler and archiver it calls."""
        needed: list[str] = []
        for case in self.setup_cases():
            for command in case.get('setup', []):
                needed.append(command[0])
                if command[0] in ('sh', 'bash') and '-c' in command:
                    script = command[command.index('-c') + 1]
                    needed += [tool for tool in SCRIPT_TOOLS if re.search(rf'(?<![\w.-]){tool}(?![\w.-])', script)]
        return sorted({tool for tool in needed if not shutil.which(tool)})

    def available(self) -> str:
        if not self.directory.is_dir():
            return f'{self.directory} is missing; run python3 tools/fetch_upstreams.py'
        tools = self.missing_tools()
        if tools:
            return f'setup needs {", ".join(tools)}, which {"is" if len(tools) == 1 else "are"} not installed'
        libs = shared_output_dir('native-libs') / host_platform()
        if self.native_libs and not libs.is_dir():
            return f'{libs} is missing; run python3 tools/native/build_native_libs.py'
        return ''

    def measured(self, directory: Path, entry: str) -> Path:
        return directory


@dataclass(frozen=True)
class Generated:
    """A program from tools/benchgen.py; the text depends only on these numbers."""
    lines: int
    seed: int = 1
    files: int = 1

    def materialize(self, scratch: Path) -> Path:
        directory = scratch / f'generated-{self.lines}-{self.seed}-{self.files}'
        if not directory.exists():
            directory.mkdir(parents=True)
            for name, text in benchgen.generate(self.lines, self.seed, self.files).items():
                (directory / name).write_text(text, encoding='utf-8')
        return directory

    def available(self) -> str:
        return ''

    def measured(self, directory: Path, entry: str) -> Path:
        return directory


# --------------------------------------------------------------------------------------- workloads

@dataclass(frozen=True)
class Plan:
    """How one mode builds a workload: the file to name on the command line, arguments for the project's
    build metaprogram (after a lone `-`), and whether the harness may redirect the executable. A project
    whose metaprogram chooses its own output keeps `redirect_output=False`. `import_dirs` are module
    folders (relative to the project) every compiler is told to search, for a project whose own build
    metaprogram would otherwise add them."""
    entry: str = 'main.jai'
    metaprogram_args: tuple[str, ...] = ()
    redirect_output: bool = True
    import_dirs: tuple[str, ...] = ()


@dataclass(frozen=True)
class Workload:
    name: str
    category: Category
    source: Source
    plans: dict[Mode, Plan]
    # The program's output is deterministic, so compilers must agree on it before being timed.
    compare_output: bool = False
    # Also run the built executable and time that (the generated-code runtime).
    runtime: bool = False

    @property
    def modes(self) -> list[Mode]:
        return list(self.plans)


def single(entry: str, *modes: Mode) -> dict[Mode, Plan]:
    return {mode: Plan(entry) for mode in modes}


CHECK_BUILD = (Mode.CHECK, Mode.BUILD)
ALL_BUILD_MODES = (Mode.CHECK, Mode.BUILD, Mode.OPTIMIZED)


def program(name: str, category: Category, directory: Path, modes: tuple[Mode, ...] = CHECK_BUILD,
            **options) -> Workload:
    return Workload(name, category, RepoDirectory(directory), single(f'{name}.jai', *modes),
                    compare_output=True, **options)


def generated(lines: int, modes: tuple[Mode, ...], runtime: bool = False) -> Workload:
    return Workload(f'gen-{lines // 1000}k', Category.LOTS_OF_CODE, Generated(lines, 1, max(1, lines // 5000)),
                    single('main.jai', *modes), compare_output=True, runtime=runtime)


def project(name: str, directory: str, plans: dict[Mode, Plan], setup: tuple[str, ...] = (),
            native_libs: bool = False) -> Workload:
    return Workload(name, Category.REAL_PROJECT, CorpusProject(directory, setup, native_libs=native_libs), plans)


def interpreter_programs() -> list[Workload]:
    names = sorted(path.stem for path in MICRO_PROGRAMS.glob('interp-*.jai'))
    return [program(name, Category.INTERPRETER, MICRO_PROGRAMS, (Mode.INTERP,)) for name in names]


def runtime_programs() -> list[Workload]:
    """One program per kind of work, built and run: the library is timed by `jacket bench`, see `benchmarks/bench`."""
    return [program(path.stem, Category.GENERATED_CODE, RUNTIME_PROGRAMS, (Mode.BUILD, Mode.OPTIMIZED), runtime=True)
            for path in sorted(RUNTIME_PROGRAMS.glob('*.jai'))]


def project_drivers() -> list[Workload]:
    """Upstream libraries built and run through a driver named after the project's repository."""
    projects = {'jai-crypto': 'smari--jai-crypto', 'jaison': 'rluba--jaison', 'jai-xml': 'smari--jai-xml'}
    return [Workload(f'{name}-run', Category.REAL_PROJECT,
                     CorpusProject(projects[name], driver=PROJECT_DRIVERS / f'{name}.jai'),
                     single(f'{name}.jai', Mode.BUILD, Mode.OPTIMIZED), compare_output=True, runtime=True)
            for name in sorted(projects)]


def own_output(entry: str, *metaprogram_args: str) -> Plan:
    """A build driven by the project's metaprogram, which decides where the executable goes."""
    return Plan(entry, metaprogram_args, redirect_output=False)


def all_workloads() -> list[Workload]:
    compile_time = [program(name, Category.COMPILE_TIME, COMPILE_PROGRAMS)
                    for name in ('run-loops', 'insert-meta', 'polymorph-explosion', 'constant-tables')]
    return [
        program('hello', Category.STARTUP, COMPILE_PROGRAMS, (Mode.CHECK, Mode.BUILD, Mode.INTERP)),
        generated(2_000, (Mode.CHECK, Mode.BUILD, Mode.INTERP)),
        generated(10_000, (Mode.CHECK, Mode.BUILD, Mode.OPTIMIZED), runtime=True),
        generated(60_000, CHECK_BUILD),
        project('focus', 'focus-editor--focus', {
            Mode.CHECK: Plan('first.jai'), Mode.BUILD: own_output('first.jai'), Mode.OPTIMIZED: own_output('first.jai', 'release')},
                native_libs=True),
        # Jails's build.jai only creates a workspace for server/main.jai and adds ./modules to its import
        # path, and links with the compiler's own linker. The server itself is built here, with the module
        # folder on the command line, so every compiler does the same work and links the same way.
        project('jails', 'SogoCZE--Jails',
                {mode: Plan('server/main.jai', import_dirs=('modules',)) for mode in ALL_BUILD_MODES}),
        project('sgpu-examples', 'roeyb1--sgpu/examples', {Mode.CHECK: Plan('build.jai')}),
        project('jaison-tests', 'rluba--jaison', single('tests.jai', *CHECK_BUILD, Mode.OPTIMIZED)),
        project('open-jai-drawing-board', 'withlang-dev--open-jai/examples/33',
                single('33.13_drawing_board.jai', *CHECK_BUILD, Mode.OPTIMIZED)),
        project('forbear', 'gabrielmfern--forbear', {
            Mode.CHECK: Plan('build.jai'), Mode.BUILD: own_output('build.jai')},
            setup=('forbear-build',)),
        *compile_time,
        program('compile-time', Category.COMPILE_TIME, MICRO_PROGRAMS, CHECK_BUILD),
        program('stdlib-imports', Category.STDLIB, COMPILE_PROGRAMS),
        *interpreter_programs(),
        *runtime_programs(),
        *project_drivers(),
    ]
