#!/usr/bin/env python3
"""Line and function coverage for `jaic -coverage` programs (docs/tools/coverage.md).

  report.py ingest --exe EXE RUN.cov... --out DIR/name.json   fold runs of one executable into
                                                              per-line and per-function hits
  report.py report DIR_OR_JSON... [--root stdlib] [--lcov F]  merge ingested runs and print the table

`ingest` needs the executable's debug info (the .dSYM next to it on macOS) and must run before
either is deleted. A run file lists every block's link-time address and whether it ran; the
DWARF line table turns block address ranges into source lines, and `llvm-symbolizer` names the
function entries.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import bisect
import json
import os
import re
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from toolkit.repo import ROOT


def llvm_tool(name):
    """`name` from the LLVM jaic links against (LLVM_SYS_231_PREFIX), else Homebrew's, else PATH."""
    prefixes = [
        os.environ.get("LLVM_SYS_231_PREFIX"),
        "/opt/homebrew/opt/llvm",
        "/usr/local/opt/llvm",
    ]
    for prefix in prefixes:
        if prefix and (Path(prefix) / "bin" / name).is_file():
            return str(Path(prefix) / "bin" / name)
    for candidate in (f"{name}-23", name):
        found = shutil.which(candidate)
        if found:
            return found
    sys.exit(f"coverage: {name} not found (install LLVM or set LLVM_SYS_231_PREFIX)")


def read_runs(paths):
    """{address: [is_entry, hit]} for the blocks of every run file (hits of the same
    executable's runs are OR-ed)."""
    blocks = {}
    exe = None
    for path in paths:
        with open(path) as f:
            if not f.readline().startswith("jaic-coverage"):
                sys.exit(f"coverage: {path} is not a jaic coverage file")
            for line in f:
                if line.startswith("exe "):
                    exe = line[4:].strip()
                elif not line.startswith("module "):
                    address, entry, hit = line.split()
                    block = blocks.setdefault(int(address, 16), [int(entry), 0])
                    block[1] |= int(hit)
    return exe, blocks


ROW = re.compile(r"^0x([0-9a-f]+)\s+(\d+)\s+\d+\s+(\d+)\s+\d+\s+\d+\s+(.*)$")


def line_table(debug_file):
    """Rows of the DWARF line tables as (address, line, path, ends_sequence), sorted by address."""
    out = subprocess.run(
        [llvm_tool("llvm-dwarfdump"), "--debug-line", str(debug_file)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    rows = []
    directories, files = {}, {}
    for text in out.splitlines():
        m = re.match(r'include_directories\[\s*(\d+)\]\s*=\s*"(.*)"', text)
        if m:
            directories[int(m[1])] = m[2]
            continue
        if text.startswith("debug_line["):
            directories, files, pending = {}, {}, None
            continue
        m = re.match(r"file_names\[\s*(\d+)\]:", text)
        if m:
            pending = int(m[1])
            continue
        m = re.match(r'\s+name: "(.*)"', text)
        if m and pending is not None:
            files[pending] = [m[1], 0]
            continue
        m = re.match(r"\s+dir_index: (\d+)", text)
        if m and pending is not None:
            files[pending][1] = int(m[1])
            continue
        m = ROW.match(text)
        if m:
            name, directory = files.get(int(m[3]), ["?", 0])
            base = directories.get(directory, "")
            path = name if name.startswith("/") else f"{base}/{name}" if base else name
            rows.append((int(m[1], 16), int(m[2]), path, "end_sequence" in m[4]))
    rows.sort(key=lambda r: (r[0], not r[3]))
    return rows


def ingest(exe, cov_files, prefix):
    exe_path, blocks = read_runs(cov_files)
    exe = Path(exe or exe_path)
    dsym = Path(f"{exe}.dSYM/Contents/Resources/DWARF/{exe.name}")
    debug_file = dsym if dsym.is_file() else exe
    rows = line_table(debug_file)
    addresses = [r[0] for r in rows]
    pcs = sorted(blocks)
    lines = defaultdict(dict)
    for i, pc in enumerate(pcs):
        end = pcs[i + 1] if i + 1 < len(pcs) else pc + 1
        hit = blocks[pc][1]
        # The row in force at `pc` and every row that starts inside the block.
        start = max(bisect.bisect_right(addresses, pc) - 1, 0)
        index = start
        while index < len(rows) and (index == start or rows[index][0] < end):
            address, line, path, ended = rows[index]
            if line and not ended and path.startswith(prefix):
                lines[path][line] = lines[path].get(line, 0) | hit
            index += 1
    entries = [pc for pc in pcs if blocks[pc][0]]
    functions = {}
    if entries:
        query = "\n".join(f"0x{pc:x}" for pc in entries) + "\n"
        out = subprocess.run(
            [
                llvm_tool("llvm-symbolizer"),
                f"--obj={debug_file}",
                "--functions=linkage",
                "--no-inlines",
            ],
            input=query,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.split("\n\n")
        for pc, record in zip(entries, out):
            parts = record.strip().split("\n")
            if len(parts) < 2:
                continue
            name, location = parts[0], parts[1]
            if not location.startswith(prefix):
                continue
            file_path, line_number = re.match(
                r"(.*?):(\d+)(?::\d+)?$", location
            ).groups()
            if line_number == "0":
                continue  # compiler-generated, no source location
            name = re.sub(r"\.\d+$", "", name)
            functions[f"{file_path}:{line_number}:{name}"] = blocks[pc][1]
    return {
        "lines": {p: {str(line): h for line, h in v.items()} for p, v in lines.items()},
        "functions": functions,
    }


def merge(runs):
    lines = defaultdict(lambda: defaultdict(int))
    functions = defaultdict(int)
    for run in runs:
        for path, hits in run["lines"].items():
            for line, hit in hits.items():
                lines[path][int(line)] += hit
        for key, hit in run["functions"].items():
            if not re.match(r".*:0:", key):
                functions[key] += hit
    return lines, functions


def module_of(path, root):
    parts = Path(path).relative_to(root).parts
    if parts[0] in ("internal", "Extensions") and len(parts) > 2:
        return "/".join(parts[:2])
    return parts[0].removesuffix(".jai")


def pct(hit, total):
    return f"{100 * hit / total:5.1f}%" if total else "    -"


def report(inputs, root, lcov, per_file, least, as_json=False):
    paths = []
    for item in inputs:
        item = Path(item)
        paths += sorted(item.glob("*.json")) if item.is_dir() else [item]
    runs = [json.loads(p.read_text()) for p in paths]
    lines, functions = merge(runs)
    root = Path(root).resolve()
    stats = defaultdict(
        lambda: [0, 0, 0, 0]
    )  # lines hit, lines, functions hit, functions
    by_file = defaultdict(lambda: [0, 0, 0, 0])
    for path, hits in lines.items():
        if not Path(path).is_relative_to(root):
            continue
        for key in (module_of(path, root),):
            stats[key][0] += sum(1 for h in hits.values() if h)
            stats[key][1] += len(hits)
        by_file[path][0] += sum(1 for h in hits.values() if h)
        by_file[path][1] += len(hits)
    for key, hit in functions.items():
        path = split_function(key)[0]
        if not Path(path).is_relative_to(root):
            continue
        stats[module_of(path, root)][2] += 1 if hit else 0
        stats[module_of(path, root)][3] += 1
        by_file[path][2] += 1 if hit else 0
        by_file[path][3] += 1
    total = [sum(v[i] for v in stats.values()) for i in range(4)]
    if as_json:
        print(json.dumps({"modules": stats, "total": total}, indent=1))
    else:
        rows = sorted(
            stats.items(), key=lambda kv: (kv[1][0] / kv[1][1]) if kv[1][1] else 2
        )
        if least:
            rows = rows[:least]
        width = max([len(k) for k, _ in rows] + [6])
        print(f"{'module':<{width}}  {'lines':>16}  {'functions':>16}")
        for key, (lh, lt, fh, ft) in rows:
            print(
                f"{key:<{width}}  {lh:>6}/{lt:<6} {pct(lh, lt)}  {fh:>5}/{ft:<5} {pct(fh, ft)}"
            )
        print(
            f"{'total':<{width}}  {total[0]:>6}/{total[1]:<6} {pct(total[0], total[1])}"
            f"  {total[2]:>5}/{total[3]:<5} {pct(total[2], total[3])}"
        )
        print(
            f"({len(runs)} runs; only code compiled into the instrumented programs is counted)"
        )
        if per_file:
            print()
            for path, (lh, lt, fh, ft) in sorted(
                by_file.items(), key=lambda kv: kv[1][0] / max(kv[1][1], 1)
            ):
                rel = (
                    Path(path).relative_to(ROOT)
                    if Path(path).is_relative_to(ROOT)
                    else path
                )
                print(
                    f"{str(rel):<{width + 40}} {lh:>5}/{lt:<5} {pct(lh, lt)}  {fh:>4}/{ft:<4} {pct(fh, ft)}"
                )
    if lcov:
        write_lcov(lcov, lines, functions)


def write_lcov(path, lines, functions):
    by_file = defaultdict(list)
    for key, hit in functions.items():
        file_path, line, name = split_function(key)
        by_file[file_path].append((int(line), name, hit))
    with open(path, "w") as out:
        for file_path in sorted(lines):
            out.write(f"TN:\nSF:{file_path}\n")
            funcs = sorted(by_file.get(file_path, []))
            for line, name, _ in funcs:
                out.write(f"FN:{line},{name}\n")
            for line, name, hit in funcs:
                out.write(f"FNDA:{hit},{name}\n")
            out.write(f"FNF:{len(funcs)}\nFNH:{sum(1 for f in funcs if f[2])}\n")
            for line in sorted(lines[file_path]):
                out.write(f"DA:{line},{lines[file_path][line]}\n")
            out.write(
                f"LF:{len(lines[file_path])}\nLH:{sum(1 for h in lines[file_path].values() if h)}\n"
            )
            out.write("end_of_record\n")


def split_function(key):
    """`/path/file.jai:12:name` -> (path, line, name)."""
    m = re.match(r"(.*?):(\d+):(.*)$", key)
    return m[1], m[2], m[3]


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="command", required=True)
    i = sub.add_parser("ingest")
    i.add_argument("--exe")
    i.add_argument("--out", required=True)
    i.add_argument(
        "--prefix",
        default=str(ROOT / "stdlib"),
        help="keep only sources under this path",
    )
    i.add_argument("runs", nargs="+")
    r = sub.add_parser("report")
    r.add_argument("inputs", nargs="+")
    r.add_argument("--root", default=str(ROOT / "stdlib"))
    r.add_argument("--lcov")
    r.add_argument("--files", action="store_true", help="also list every file")
    r.add_argument(
        "--least", type=int, default=0, help="only the N least covered modules"
    )
    r.add_argument("--json", action="store_true")
    a = ap.parse_args()
    if a.command == "ingest":
        data = ingest(a.exe, a.runs, a.prefix)
        Path(a.out).parent.mkdir(parents=True, exist_ok=True)
        Path(a.out).write_text(json.dumps(data))
    else:
        report(a.inputs, a.root, a.lcov, a.files, a.least, a.json)


if __name__ == "__main__":
    main()
