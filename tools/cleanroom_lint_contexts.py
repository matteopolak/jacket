"""Validate real jailint contexts and retain incomplete/unvisited coverage."""
from pathlib import Path
import re


TARGETS = {
    "macos-arm64": ["-os", "macos", "-cpu", "arm64"],
    "macos-x64": ["-os", "macos", "-cpu", "x64"],
    "linux-arm64": ["-os", "linux", "-cpu", "arm64"],
    "linux-x64": ["-os", "linux", "-cpu", "x64"],
    "windows-arm64": ["-target", "aarch64-pc-windows-msvc"],
    "windows-x64": ["-target", "x86_64-pc-windows-msvc"],
    "wasi": ["-target", "wasm64-unknown-wasi"],
}


def resolve_contexts(document, root):
    if document.get("version") != 1:
        raise ValueError("unsupported lint-context manifest version")
    jobs = []
    names = set()
    for context in document["contexts"]:
        name = context["id"]
        if not re.fullmatch(r"[A-Za-z0-9_.-]+", name):
            raise ValueError("invalid lint context identifier")
        if name in names:
            raise ValueError("duplicate lint context: " + name)
        names.add(name)
        source = (root / context["source"]).resolve(strict=True)
        paths = [(root / path).resolve(strict=True) for path in context["paths"]]
        if not source.is_file() or not source.is_relative_to(root):
            raise ValueError("lint context must be a real source under this checkout")
        if not paths or any(not path.is_relative_to(root) for path in paths):
            raise ValueError("lint paths must remain under this checkout")
        if not context["targets"]:
            raise ValueError("lint context requires a target")
        for target in context["targets"]:
            if target not in TARGETS:
                raise ValueError("unknown source-check target: " + target)
            jobs.append({"id": name + "-" + target, "target": target,
                         "source": source, "paths": paths})
    if not jobs:
        raise ValueError("empty lint-context manifest")
    return jobs


def checked_coverage(document, exit_code):
    """A visited file in an incomplete/denied run never establishes clean lint."""
    if document.get("version") != 1 or document.get("kind") != "source-check":
        raise ValueError("invalid jailint coverage manifest")
    clean = exit_code == 0 and not document["incomplete"]
    visited = set()
    partial = set()
    for context in document["contexts"]:
        paths = {str(Path(path).resolve()) for path in context["visited_files"]}
        if clean and context["complete"]:
            visited.update(paths)
        else:
            partial.update(paths)
    return {"clean": clean and all(c["complete"] for c in document["contexts"]),
            "visited_complete": sorted(visited), "visited_partial": sorted(partial),
            "unvisited": document["unvisited_files"]}
