#!/usr/bin/env python3
"""Fetch only pinned public application/tutorial Jai source into this worktree.

No Git cache, alternate checkout, credentials, reference libraries, compiler/parser
projects, native binaries or repository discovery. Every byte is manifest-verified.
"""
import concurrent.futures
import hashlib
import json
from pathlib import Path, PurePosixPath
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
APPLICATIONS = {
    "focus-editor/focus", "Ivo-Balbaert/The_Way_to_Jai", "ostef/Vk-Engine",
    "roeyb1/sgpu", "danieltan1517/chess-jai", "gabrielmfern/forbear",
    "rexim/rexim.github.io", "kooparse/ui_builder", "DavidColson/Photon",
    "kujukuju/KodaJai", "UnNabbo/no_api",
}


def fetch(project, record):
    path = PurePosixPath(record["path"])
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"unsafe manifest path: {path}")
    destination = ROOT / "corpus/lsp-applications" / project["repository"].replace("/", "--") / path
    if destination.is_file():
        data = destination.read_bytes()
    else:
        url = f"https://raw.githubusercontent.com/{project['repository']}/{project['revision']}/{urllib.parse.quote(str(path))}"
        request = urllib.request.Request(url, headers={"User-Agent": "cleanroom-lsp-corpus"})
        with urllib.request.urlopen(request, timeout=45) as response:
            data = response.read()
    if len(data) != record["bytes"] or hashlib.sha256(data).hexdigest() != record["sha256"]:
        raise ValueError(f"manifest mismatch: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)


def main():
    manifest = json.loads((ROOT / "corpus/upstreams.json").read_text())
    items = [(project, record) for project in manifest["projects"] if project["repository"] in APPLICATIONS
             for record in project["files"] if record["path"].endswith(".jai")]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        futures = [pool.submit(fetch, project, record) for project, record in items]
        for future in concurrent.futures.as_completed(futures):
            future.result()
    print(f"Verified {len(items)} pinned application/tutorial source files in corpus/lsp-applications")


if __name__ == "__main__":
    main()
