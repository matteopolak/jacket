"""The workspaces the language server benchmark opens."""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

import benchgen
from results import ROOT


@dataclass(frozen=True)
class Prepared:
    """A workspace ready to open: the folder, the document the probes work on, the documents an editor would
    have open already, and a file the watcher probe may change on disk (None when the workspace is read-only)."""
    workspace: Path
    document: Path
    also: tuple[Path, ...]
    watch_file: Path | None


class LspWorkload(Protocol):
    name: str
    default: bool

    def prepare(self, scratch: Path) -> Prepared: ...


@dataclass(frozen=True)
class GeneratedWorkspace:
    """A tools/benchgen.py program split over `files` parts, written to a scratch directory (the program
    depends only on these numbers). The part in the middle is the document; main.jai loads the parts."""
    name: str
    lines: int
    files: int
    default: bool = True

    def prepare(self, scratch: Path) -> Prepared:
        directory = scratch / self.name
        directory.mkdir(parents=True, exist_ok=True)
        for file, text in benchgen.generate(self.lines, 1, self.files).items():
            (directory / file).write_text(text)
        parts = sorted(p for p in directory.glob('*.jai') if p.name != 'main.jai')
        if not parts:
            return Prepared(directory, directory / 'main.jai', (), None)
        document = parts[len(parts) // 2]
        return Prepared(directory, document, (directory / 'main.jai',), next(p for p in parts if p != document))


@dataclass(frozen=True)
class RepositoryWorkspace:
    """This repository, opened at its root, with one of its larger Jai files as the document."""
    name: str = 'repo'
    document: str = 'stdlib/Extensions/Jai_Format/format.jai'
    also: tuple[str, ...] = ('stdlib/Extensions/Jai_Format/module.jai',)
    default: bool = True

    def prepare(self, scratch: Path) -> Prepared:
        return Prepared(ROOT, ROOT / self.document, tuple(ROOT / p for p in self.also), None)


WORKLOADS: list[LspWorkload] = [
    GeneratedWorkspace('gen-60k', 60_000, 12),
    GeneratedWorkspace('gen-240k', 240_000, 48, default=False),
    GeneratedWorkspace('large-25k', 25_000, 1),
    GeneratedWorkspace('large-100k', 100_000, 1, default=False),
    RepositoryWorkspace(),
]
