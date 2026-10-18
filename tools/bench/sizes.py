"""How big a workload is, from the same lexer `tools/jaistats.py` uses on the corpus."""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from dataclasses import asdict, dataclass
from pathlib import Path

import jaistats


@dataclass(frozen=True)
class SourceFacts:
    files: int = 0
    lines: int = 0  # non-blank code lines, comments excluded
    procs: int = 0
    structs: int = 0  # structs, unions and enums
    bytes: int = 0

    def to_json(self) -> dict:
        return asdict(self)

    def per_kloc(self, seconds: float) -> float | None:
        """`seconds` per thousand lines of code."""
        return seconds / (self.lines / 1000) if self.lines else None


def facts_of_text(text: str) -> SourceFacts:
    blanked = jaistats.blank(text)
    lines = sum(1 for line in blanked.splitlines() if line.strip())
    return SourceFacts(files=1, lines=lines, procs=len(jaistats.PROC.findall(blanked)),
                       structs=len(jaistats.STRUCT.findall(blanked)), bytes=len(text.encode()))


def facts_of(root: Path) -> SourceFacts:
    """Facts summed over every `.jai` file under `root` (or of `root` itself when it is a file)."""
    files = [root] if root.is_file() else sorted(root.rglob('*.jai'))
    total = SourceFacts()
    for file in files:
        try:
            one = facts_of_text(file.read_text(errors='replace'))
        except OSError:
            continue
        total = SourceFacts(total.files + one.files, total.lines + one.lines, total.procs + one.procs,
                            total.structs + one.structs, total.bytes + one.bytes)
    return total
