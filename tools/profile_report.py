#!/usr/bin/env python3
"""Summarize a `samply record --save-only --unstable-presymbolicate` profile in the terminal.

    samply record --save-only --unstable-presymbolicate -o prof.json.gz -- jaic check big.jai
    python3 tools/profile_report.py prof.json.gz [--top 30] [--thread NAME]

Prints the busiest thread's functions by self samples and by inclusive samples (a function counts
once per sample however deep it recurses). Symbols come from the `.syms.json` file samply writes next
to the profile.
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

    self_count, inclusive = collections.Counter(), collections.Counter()
    for stack in samples["stack"]:
        if stack is None:
            continue
        self_count[name_of(stacks["frame"][stack])] += 1
        seen = set()
        while stack is not None:
            seen.add(name_of(stacks["frame"][stack]))
            stack = stacks["prefix"][stack]
        inclusive.update(seen)
    total = samples["length"]
    print(f"thread {thread['name']}: {total} samples")
    for title, counter in (("self", self_count), ("inclusive", inclusive)):
        print(f"\n-- {title} --")
        for name, n in counter.most_common(a.top):
            print(f"{100 * n / total:6.1f}%  {name}")


if __name__ == "__main__":
    main()
