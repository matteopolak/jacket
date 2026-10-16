"""Canonical locations for retained SDK fixtures relocated into module tests.

This explicit migration inventory is shared by source checks, runtime checks
and private-unit placement. Missing canonical files are fatal; no old-path
fallback can replace the reviewed ABI assertions.
"""

from pathlib import Path

MODULE_LOCAL_FIXTURES = {
    name: Path("stdlib/SDL/tests") / name
    for name in ("sdl-layout.jai", "sdl-rwops.jai", "sdl-wm.jai")
}


def fixture_source(root, filename):
    if filename in MODULE_LOCAL_FIXTURES:
        return (root / MODULE_LOCAL_FIXTURES[filename]).resolve(strict=True)
    return root / "tests/native-bindings" / filename
