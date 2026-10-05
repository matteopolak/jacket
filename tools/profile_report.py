#!/usr/bin/env python3
"""Summarize a `samply record --save-only --unstable-presymbolicate` profile in the terminal.

    samply record --save-only --unstable-presymbolicate -o prof.json.gz -- jaic check big.jai
    python3 tools/profile_report.py prof.json.gz [--top 30] [--thread NAME] [--within TEXT] [--without TEXT]

Prints the busiest thread's functions by self samples and by inclusive samples (a function counts
once per sample however deep it recurses). Symbols come from the `.syms.json` file samply writes next
to the profile. --within/--without keep only samples whose stack has (or lacks) a frame containing the
text, e.g. `--within jaic::build::call --without jaic::build::step` for time in compiler primitives only.
--callers TEXT lists who calls the matching frames instead.
"""
import argparse
import bisect
import collections
import gzip
import json
import re
from pathlib import Path


def load_symbols(path):
    data = json.loads(Path(path).read_text())
    strings = data["string_table"]
    tables = {}
    for lib in data["data"]:
        rows = sorted(lib["symbol_table"], key=lambda r: r["rva"])
        tables[lib["debug_name"]] = ([r["rva"] for r in rows], rows, strings)
    return tables


def resolver(profile, symbols):
    libs = profile["libs"]

    def resolve(thread, frame):
        frames, funcs, resources = thread["frameTable"], thread["funcTable"], thread["resourceTable"]
        func = frames["func"][frame]
        resource = funcs["resource"][func]
        if resource is None or resource < 0:
            return thread["stringArray"][funcs["name"][func]]
        lib = libs[resources["lib"][resource]]
        table = symbols.get(lib["debugName"])
        address = frames["address"][frame]
        if table and address is not None and address >= 0:
            starts, rows, strings = table
            i = bisect.bisect_right(starts, address) - 1
            if i >= 0 and address < rows[i]["rva"] + max(rows[i]["size"], 1):
                return strings[rows[i]["symbol"]]
        return f"{lib['debugName']}+{address:#x}" if address is not None else lib["debugName"]

    return resolve


def short(name):
    name = re.sub(r"::h[0-9a-f]{16}$", "", name)
    name = re.sub(r"<[^<>]*>", "<..>", name) if len(name) > 110 else name
    return name[:110]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("profile")
    ap.add_argument("--top", type=int, default=30)
    ap.add_argument("--thread", default="", help="substring of the thread name (default: busiest)")
    ap.add_argument("--within", default="", help="only samples with a frame containing this text")
    ap.add_argument("--without", default="", help="skip samples with a frame containing this text")
    ap.add_argument("--callers", default="", help="list the callers of the frames containing this text")
    a = ap.parse_args()
    profile = json.load(gzip.open(a.profile))
    syms = Path(re.sub(r"\.json(\.gz)?$", "", a.profile) + ".json.syms.json")
    resolve = resolver(profile, load_symbols(syms) if syms.exists() else {})
    threads = [t for t in profile["threads"] if a.thread in t["name"]]
    thread = max(threads, key=lambda t: t["samples"]["length"])
    stacks, samples = thread["stackTable"], thread["samples"]
    names = {}

    def name_of(frame):
        if frame not in names:
            names[frame] = short(resolve(thread, frame))
        return names[frame]

    self_count, inclusive, callers = collections.Counter(), collections.Counter(), collections.Counter()
    for stack in samples["stack"]:
        if stack is None:
            continue
        leaf = name_of(stacks["frame"][stack])
        seen, chain = set(), []
        while stack is not None:
            seen.add(name_of(stacks["frame"][stack]))
            chain.append(name_of(stacks["frame"][stack]))
            stack = stacks["prefix"][stack]
        if a.within and not any(a.within in n for n in seen):
            continue
        if a.without and any(a.without in n for n in seen):
            continue
        self_count[leaf] += 1
        inclusive.update(seen)
        if a.callers:
            # The nearest frame above the outermost consecutive match.
            for i, name in enumerate(chain):
                if a.callers in name:
                    j = i
                    while j + 1 < len(chain) and a.callers in chain[j + 1]:
                        j += 1
                    if j + 1 < len(chain):
                        callers[chain[j + 1]] += 1
                    break
    total = samples["length"]
    print(f"thread {thread['name']}: {total} samples")
    sections = [("self", self_count), ("inclusive", inclusive)]
    if a.callers:
        sections = [(f"callers of {a.callers}", callers)]
    for title, counter in sections:
        print(f"\n-- {title} --")
        for name, n in counter.most_common(a.top):
            print(f"{100 * n / total:6.1f}%  {name}")


if __name__ == "__main__":
    main()
