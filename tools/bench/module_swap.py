"""Building one compiler against another module tree, to tell code generation apart from library code.

`module_folder` makes a folder of links to the library modules of `source`, by the names this repository's
stdlib uses, leaving out the modules a compiler is built around (its runtime, its compiler interface), which
always stay the compiler's own. The phase-timing metaprogram searches that folder first."""
from __future__ import annotations

from pathlib import Path

from results import ROOT

STDLIB = ROOT / 'stdlib'
# Modules tied to the compiler that loads them; swapping these measures nothing useful and rarely builds.
COMPILER_MODULES = frozenset({'Compiler', 'Preload', 'Runtime_Support', 'Runtime_Support_Crash_Handler',
                              'Program_Print', 'Remap_Context', 'Wasi_Numeric_Runtime'})


def module_names() -> list[str]:
    return sorted(entry.name for entry in STDLIB.iterdir() if not entry.name.startswith('.'))


def module_folder(source: Path, dest: Path) -> list[str]:
    """Links in `dest` to each library module of `source`; returns the names linked."""
    dest.mkdir(parents=True, exist_ok=True)
    linked = []
    for name in module_names():
        if Path(name).stem in COMPILER_MODULES or not (source / name).exists():
            continue
        (dest / name).symlink_to(source / name)
        linked.append(name)
    return linked
