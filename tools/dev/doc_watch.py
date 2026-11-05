"""Keep Astro HMR separate from semantic extraction during local documentation work.

    run_dev(export_command, builder, export_dir, port='8000', root=ROOT)

Component and CSS edits are Astro's. A semantic edit performs one full deterministic
export and restarts the dev server so deleted/new routes cannot retain stale loader state.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
import time

from toolkit.repo import ROOT


def snapshot(roots):
    """Content identities of watched source inputs; e.g. snapshot([ROOT / 'docs'])."""
    entries = []
    for root in roots:
        root = Path(root)
        paths = sorted(root.rglob("*")) if root.is_dir() else [root]
        for path in paths:
            if path.is_dir():
                continue
            try:
                entries.append(
                    (str(path), hashlib.sha256(path.read_bytes()).hexdigest())
                )
            except FileNotFoundError:
                continue
            except OSError as error:
                entries.append((str(path), f"unreadable:{error.errno}"))
    return tuple(sorted(entries))


class SourceWatcher:
    """Attempt one refresh per actual source change, retaining failure until the next edit."""

    def __init__(self, roots, export):
        self.roots = tuple(roots)
        self.export = export
        self.previous = snapshot(self.roots)

    def refresh(self):
        current = snapshot(self.roots)
        if current == self.previous:
            return None
        self.previous = current
        try:
            return self.export() == 0
        except OSError as error:
            print(f"docs export: {error}", file=sys.stderr)
            return False


def source_roots(root=ROOT, runtime=None):
    """The repository's semantic inputs and revision provenance, excluding website presentation."""
    roots = [
        root / "stdlib",
        root / "docs",
        root / "jacket.toml",
        root / "crates/jacket/src/doc/since.tsv",
        root / "editors/vscode/images",
    ]
    if runtime:
        roots.extend(Path(runtime) / name for name in ("engine.mjs", "jai_wasm.wasm"))
    try:

        def git_path(name):
            result = subprocess.run(
                ["git", "rev-parse", "--git-path", name],
                cwd=root,
                capture_output=True,
                text=True,
                check=True,
            )
            path = Path(result.stdout.strip())
            return path if path.is_absolute() else root / path

        head = git_path("HEAD")
        roots.extend((head, git_path("packed-refs")))
        content = head.read_text().strip()
        if content.startswith("ref: "):
            roots.append(git_path(content[5:]))
    except (OSError, subprocess.SubprocessError):
        pass
    return roots


def astro_cli(builder):
    """Find the CLI declared by the pinned Astro package, rather than guessing its layout."""
    package = Path(builder) / "node_modules/astro"
    metadata = json.loads((package / "package.json").read_text())
    entry = metadata.get("bin", {}).get("astro")
    if not isinstance(entry, str):
        raise ValueError("installed Astro has no declared CLI")
    script = (package / entry).resolve()
    if not script.is_relative_to(package.resolve()) or not script.is_file():
        raise ValueError("installed Astro CLI is absent or outside its package")
    return script


def run_dev(export, builder, export_dir, *, port="8000", root=ROOT, runtime=None):
    """Serve a complete export and watch semantic edits; e.g. `dev.py docs --dev --port 8000`."""
    builder, export_dir = Path(builder), Path(export_dir)
    node = shutil.which("node")
    if not node:
        print(
            "docs development needs Node; install the release in website/docs/.node-version",
            file=sys.stderr,
        )
        return 2
    command = [
        node,
        str(astro_cli(builder)),
        "dev",
        "--host",
        "127.0.0.1",
        "--port",
        str(port),
    ]
    stale = ""
    child = None

    def start():
        env = dict(os.environ, JACKET_DOC_EXPORT=str(export_dir), JACKET_DOC_BASE="/")
        if stale:
            env["JACKET_DOC_EXPORT_STALE"] = stale
        else:
            env.pop("JACKET_DOC_EXPORT_STALE", None)
        return subprocess.Popen(command, cwd=builder, env=env)

    def stop():
        if child is not None and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()

    try:
        child = start()
        watcher = SourceWatcher(source_roots(root, runtime), export)
        while child.poll() is None:
            time.sleep(0.75)
            result = watcher.refresh()
            if result is None:
                continue
            if child.poll() is not None:
                break
            stale = (
                ""
                if result
                else "Source export failed. Showing the last complete export; correct the source to refresh it."
            )
            print(
                "docs development: export refreshed"
                if result
                else f"docs development: {stale}",
                file=sys.stderr,
            )
            stop()
            child = start()
            watcher = SourceWatcher(source_roots(root, runtime), export)
        return child.returncode or 0
    except KeyboardInterrupt:
        return 130
    except OSError as error:
        print(f"docs development: {error}", file=sys.stderr)
        return 2
    finally:
        stop()
