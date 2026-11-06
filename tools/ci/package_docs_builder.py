#!/usr/bin/env python3
"""Copy the locked Astro builder's source closure into an installed toolchain.

    python3 tools/ci/package_docs_builder.py --output jacket-platform/docs-builder

Node and npm dependencies are deliberately absent. Installed users run `npm ci`
once in docs-builder; rendering never downloads or installs packages implicitly.
"""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tempfile

from toolkit.repo import ROOT

# These roots constitute the installed source package, rather than the checkout's
# unbounded contents. Keep additions explicit when the builder gains new inputs.
FILES = ("package.json", "package-lock.json", "astro.config.mjs", "tsconfig.json", "tsconfig.worker.json", ".node-version",
         "scripts/build.mjs", "scripts/check-site.mjs", "scripts/generate-types.mjs")
TREES = ("src", "schema", "public")
REQUIRED = FILES + ("src/generated/doc-site.d.ts", "schema/doc-site.schema.json", "scripts/build.mjs")
EXCLUDED = frozenset({"node_modules", ".astro", "dist", "export", "tests", "__tests__", ".git", "coverage"})


def inventory(source):
    """Validate and list portable source paths, e.g. inventory(ROOT / 'website/docs')."""
    if source.is_symlink():
        raise ValueError("builder source must not be a symlink")
    for name in REQUIRED:
        path = source / name
        if not path.is_file():
            raise ValueError(f"builder source missing {name}")
    try:
        package = json.loads((source / "package.json").read_text(encoding="utf-8"))
        lock = json.loads((source / "package-lock.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise ValueError(f"builder package metadata: {error}") from error
    if not all(isinstance(package.get(key), str) and package[key] for key in ("name", "version")):
        raise ValueError("builder package metadata needs name and version")
    if lock.get("lockfileVersion") != 3 or any(lock.get(key) != package[key] for key in ("name", "version")):
        raise ValueError("builder lockfile must use format 3 and match package name/version")
    locked_root = lock.get("packages", {}).get("", {})
    if any(locked_root.get(key) != package[key] for key in ("name", "version")):
        raise ValueError("builder lockfile root package must match package name/version")
    for key in ("dependencies", "devDependencies", "optionalDependencies", "engines"):
        if locked_root.get(key, {}) != package.get(key, {}):
            raise ValueError(f"builder lockfile root {key} must match package metadata")
    paths = []

    def add(path):
        if path.is_symlink():
            raise ValueError(f"builder source contains symlink: {path.relative_to(source).as_posix()}")
        if path.is_dir():
            for child in sorted(path.iterdir()):
                if child.name not in EXCLUDED and not child.name.startswith(".env"):
                    add(child)
        elif path.is_file():
            paths.append(path.relative_to(source).as_posix())
        else:
            raise ValueError(f"builder source is not a regular file: {path.relative_to(source).as_posix()}")

    for name in FILES:
        add(source / name)
    for name in TREES:
        if (source / name).exists():
            add(source / name)
    return package, sorted(paths)


def package(source, destination):
    """Stage a source-only builder atomically, e.g. package(source, archive / 'docs-builder')."""
    source, destination = Path(source), Path(destination)
    if destination.exists() or destination.is_symlink():
        raise ValueError(f"builder destination already exists: {destination}")
    if destination.resolve().is_relative_to(source.resolve()):
        raise ValueError("builder destination must be outside its source")
    metadata, paths = inventory(source)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".docs-builder-", dir=destination.parent) as temporary:
        stage = Path(temporary) / "docs-builder"
        stage.mkdir()
        entries = []
        for name in paths:
            data = (source / name).read_bytes()
            target = stage / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            entries.append({"path": name, "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})
        manifest = {
            "schema": "jacket-docs-builder", "schemaVersion": 1,
            "package": {key: metadata[key] for key in ("name", "version")},
            "files": entries,
        }
        (stage / "builder-manifest.json").write_bytes((json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
        shutil.move(str(stage), str(destination))
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--source", type=Path, default=ROOT / "website/docs")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        manifest = package(args.source, args.output)
    except (OSError, ValueError) as error:
        parser.exit(2, f"docs builder: {error}\n")
    print(f"docs builder: {len(manifest['files'])} source files copied to {args.output}")


if __name__ == "__main__":
    main()
