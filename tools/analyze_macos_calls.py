#!/usr/bin/env python3
"""Resolve native ARM64 Mach-O call targets and summarize sensitive API paths."""
import collections
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "artifacts/audit"
SENSITIVE = {"_system", "_execv", "_execve", "_posix_spawn", "_dlopen", "_unlink", "_remove", "_chmod", "_fchmod", "_open", "_fopen", "_getenv", "_socket", "_connect", "_send", "_recv"}


def run(*args):
    return subprocess.run(args, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout


def shortest_path(graph, start, target):
    parents = {start: None}
    todo = collections.deque([start])
    while todo:
        caller = todo.popleft()
        if caller == target:
            path = []
            while caller is not None:
                path.append(caller)
                caller = parents[caller]
            return path[::-1]
        for callee in graph.get(caller, ()):
            if callee not in parents:
                parents[callee] = caller
                todo.append(callee)
    return None


def main():
    report = json.loads((OUT / "report.json").read_text())
    expected = next(b for b in report["binaries"] if b["path"] == "bin/jai-macos")
    actual = hashlib.file_digest(REFERENCE.open("rb"), "sha256").hexdigest()
    if actual != expected["sha256"]:
        raise ValueError("reference binary changed since disassembly")
    with tempfile.TemporaryDirectory(prefix="jai-import-analysis-") as temp:
        thin = str(Path(temp) / "arm64")
        run("lipo", str(REFERENCE), "-thin", "arm64", "-output", thin)
        imports_text = run("otool", "-Iv", thin)
    (OUT / "macos-arm64-indirect-symbols.txt").write_text(imports_text)
    imports = {}
    for line in imports_text.splitlines():
        m = re.match(r"^(0x[0-9a-fA-F]+)\s+\d+\s+(\S+)$", line)
        if m:
            imports[int(m[1], 16)] = m[2]
    graph = collections.defaultdict(set)
    callers = collections.defaultdict(list)
    indirect = collections.Counter()
    table = OUT / "bin_jai-macos_arm64.calls.tsv.gz"
    with gzip.open(table, "rt") as f:
        next(f)
        for line in f:
            caller, site, kind, target = line.rstrip("\n").split("\t", 3)
            if kind == "indirect":
                indirect[caller] += 1
                continue
            address = re.match(r"0x([0-9a-fA-F]+)", target)
            symbol = re.search(r"<(.+)>", target)
            callee = imports.get(int(address[1], 16)) if address else None
            if not callee and symbol:
                callee = re.sub(r"\+0x[0-9a-fA-F]+$", "", symbol[1])
            if callee and callee != caller:
                graph[caller].add(callee)
                if callee in SENSITIVE:
                    callers[callee].append({"caller": caller, "site": site, "instruction_kind": kind})
    summary = {"binary_sha256": actual, "architecture": "arm64",
               "assessment": "not certified harmless; unrestricted host execution not recommended",
               "sensitive_imports": sorted(set(imports.values()) & SENSITIVE),
               "sensitive_call_sites": dict(callers),
               "direct_static_paths_from_main": {s: shortest_path(graph, "_main", s) for s in sorted(callers)},
               "functions_with_indirect_calls": len(indirect),
               "most_indirect_call_sites": indirect.most_common(20),
               "path_limit": "No direct path is not proof of unreachability: worker callbacks, indirect calls, initializers and foreign code are outside this traversal."}
    (OUT / "macos-arm64-capabilities.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(summary["assessment"])
    for api, sites in sorted(callers.items()):
        path = summary["direct_static_paths_from_main"][api]
        print(f"{api}: {len(sites)} observed call/branch sites; direct path from main: {bool(path)}")


if __name__ == "__main__":
    main()
