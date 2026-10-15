#!/usr/bin/env python3
"""Stage manifest-verified application files, excluding vendor paths before access.

This performs no fetching and never walks source trees. Excluded records are
classified using manifest strings alone; neither their metadata nor bodies are
opened. Original application bytes are preserved. Missing dependencies remain
real compilation failures against the NEW production library.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath

from fetch_lsp_application_corpus import APPLICATIONS

BLOCKED = frozenset({"module", "modules", "stdlib", "prelude", "compiler",
                     "jai_parser", "vendor", "third_party", "third-party",
                     "dependencies", "libs", "libraries"})


def exclusion(path):
    relative = PurePosixPath(path)
    if relative.is_absolute() or ".." in relative.parts or "\\" in path:
        raise ValueError("unsafe manifest path: " + path)
    if any(part.lower() in BLOCKED for part in relative.parts[:-1]):
        return "excluded library/compiler/module directory; never accessed"
    if relative.name.lower() in {"preload.jai", "module.jai"}:
        return "excluded library entry; never accessed"
    return None


def stage(manifest, source, destination):
    records = []
    for project in manifest["projects"]:
        repository = project["repository"]
        if repository not in APPLICATIONS:
            continue
        folder = repository.replace("/", "--")
        for record in project["files"]:
            name = record["path"]
            if not name.endswith(".jai"):
                continue
            reason = exclusion(name)
            entry = dict(repository=repository, revision=project["revision"],
                         path=name, sha256=record["sha256"], status="excluded" if reason else "staged")
            if reason:
                entry["reason"] = reason
            else:
                original = source / folder / name
                # Reject alternate paths through links, including parent links.
                for path in [source, *original.relative_to(source).parents]:
                    check = path if path == source else source / path
                    if check.is_symlink():
                        raise ValueError("symlink source path: " + str(check))
                if original.is_symlink():
                    raise ValueError("symlink source file: " + str(original))
                data = original.read_bytes()
                if len(data) != record["bytes"] or hashlib.sha256(data).hexdigest() != record["sha256"]:
                    raise ValueError("manifest mismatch: " + str(original))
                target = destination / folder / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            records.append(entry)
    return dict(schema=1, manifest_files=len(records), staged=sum(r["status"] == "staged" for r in records),
                excluded=sum(r["status"] == "excluded" for r in records), records=records)


def main():
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=root / "corpus/lsp-applications")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if ".." in args.output.parts:
        parser.error("output cannot contain parent traversal")
    output = args.output.absolute().resolve()
    if not output.is_relative_to(root / "artifacts") or output.exists():
        parser.error("output must be a fresh directory under this worktree's artifacts")
    output.mkdir(parents=True)
    manifest = json.loads((root / "corpus/upstreams.json").read_text())
    report = stage(manifest, args.source.absolute(), output)
    (output.parent / (output.name + "-provenance.json")).write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in ("manifest_files", "staged", "excluded")}))


if __name__ == "__main__":
    main()
