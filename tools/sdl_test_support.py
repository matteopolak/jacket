"""Explicit scratch and canonical source locations for SDL Python tests."""

import os
from pathlib import Path
import tempfile


def repository_root():
    return Path(
        os.environ.get("SDL_PYTHON_TEST_ROOT", Path(__file__).resolve().parents[1])
    ).resolve(strict=True)


def scratch_root():
    value = os.environ.get("SDL_TEST_TMPDIR")
    if not value:
        raise RuntimeError(
            "SDL_TEST_TMPDIR is required: run tools/prepare_sdl_python_tests.py before discovery"
        )
    root = Path(value)
    if not root.is_absolute():
        raise ValueError("SDL_TEST_TMPDIR must be an absolute owned scratch path")
    root.mkdir(parents=True, exist_ok=True)
    return root


def temporary_directory():
    return tempfile.TemporaryDirectory(dir=scratch_root())


def source_path(key, relative):
    """Overrides are explicit; ordinary checkout discovery needs no helper keys."""
    return Path(os.environ.get(key, repository_root() / relative)).resolve(strict=True)


def remove_fixture_metadata(root):
    """Remove verified macOS AppleDouble metadata from owned synthetic fixtures.

    Native SDK/source inventories are never passed here. Unknown dot-underscore
    files remain fatal, so no malformed fixture header is silently ignored.
    """
    root = Path(root).resolve(strict=True)
    if scratch_root().resolve() not in root.parents:
        raise ValueError("Fixture metadata cleanup requires a child of SDL_TEST_TMPDIR")
    for path in root.rglob("._*"):
        if not path.is_file() or path.read_bytes()[:8] != bytes.fromhex(
            "0005160700020000"
        ):
            raise ValueError("Unrecognized fixture sidecar: " + str(path))
        original = path.with_name(path.name[2:])
        if not original.exists():
            raise ValueError("Orphan fixture metadata: " + str(path))
        path.unlink()
