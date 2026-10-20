"""Building one compiler against some of another tree's modules, to tell code generation apart from
library code: `--swap-modules Sort,Hash_Table` with `--baseline-modules` or `--candidate-modules DIR`.

`module_folder` makes a folder of links to the named modules of a tree, plus its `internal` folder when it
has one (this repository's modules keep their shared code there). The phase-timing metaprogram searches
that folder first, so those modules replace the compiler's own and everything else stays its own. Whole
trees don't swap: each compiler's modules use things only that compiler accepts."""
from __future__ import annotations

from pathlib import Path

SHARED = 'internal'


def module_folder(source: Path, dest: Path, names: list[str]) -> list[str]:
    """Links in `dest` to each of `names` in `source` (and its `internal` folder); returns the names linked.
    A name `source` lacks is an error: the comparison would quietly use the compiler's own module."""
    dest.mkdir(parents=True, exist_ok=True)
    missing = [name for name in names if not (source / name).exists() and not (source / f'{name}.jai').exists()]
    if missing:
        raise SystemExit(f'{source} has no module {", ".join(missing)}')
    linked = []
    for name in [*names, SHARED]:
        for entry in (source / name, source / f'{name}.jai'):
            if entry.exists() and not (dest / entry.name).exists():
                (dest / entry.name).symlink_to(entry)
                linked.append(entry.name)
    return linked
