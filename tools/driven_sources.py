"""Find the sources a tool compiles with its own flags: `stdlib/**/tests/driven/`.

A module's `tests/` directory holds the programs `stdlib_runtime.py` runs as they are. What
a checker builds with special flags, inputs or native oracles (code generators' headers and
expected outputs, debugger and profiler fixtures, process helpers) lives in `tests/driven/`
beside them and is never discovered as a case. File names in these directories are unique
across the stdlib, so a tool names the file and not the module that owns it.
"""

from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@lru_cache(maxsize=1)
def _index():
    index = {}
    for directory in sorted((ROOT / "stdlib").glob("**/tests/driven")):
        for path in sorted(p for p in directory.rglob("*") if p.is_file()):
            relative = path.relative_to(directory).as_posix()
            if relative in index:
                raise RuntimeError(f"driven file name is not unique: {relative}")
            index[relative] = path
    return index


def driven(name):
    """The path of the driven source `name`, relative to its `tests/driven` directory."""
    try:
        return _index()[name]
    except KeyError:
        raise FileNotFoundError(f"no stdlib/**/tests/driven/{name}") from None


def driven_matching(pattern):
    """Every driven file directly in a `tests/driven` directory whose name matches the glob
    `pattern`, in a stable order. Files in sub-directories are named in full by `driven`."""
    from fnmatch import fnmatch

    return [
        path
        for name, path in sorted(_index().items())
        if "/" not in name and fnmatch(name, pattern)
    ]
