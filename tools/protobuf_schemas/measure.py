#!/usr/bin/env python3
"""Measure schema generation of `Extensions/Protobuf/Schema` on the pinned real-world sets.

Run `fetch.py` first. For each variant in `schemas.json` this builds a small program that imports
the schema, round-trips a default message of every listed type and prints OK, then records:

  gen_ms       protobuf_generate in the interpreter (gen_probe.jai), excluding start-up
  gen_wall     whole `jaic -interp gen_probe.jai` run
  check_s      `jaic -check prog.jai` (generation + type-check, nothing built)
  o2_s         `jaic -O2 prog.jai` to an executable
  emitted      bytes of generated Jai
  binary       size of the executable; `ok` says whether it ran and printed OK

The fetched `.proto` files are untrusted data and are only read as text by the generator.
`--lock DIR` serialises heavy steps with other jobs through an `mkdir` lock.

    python3 tools/protobuf_schemas/measure.py --jaic PATH [variant ...] [--json out.json]
"""
from __future__ import annotations

import argparse
import contextlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_CACHE = "/Volumes/LodestoneScratch/protobuf-schemas"
DEFAULT_OUT = "/Volumes/LodestoneScratch/protobuf-schemas-out"
TIMEOUT = 900

PROGRAM = """#import "Basic";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(%SOURCE%, IMPORT_PATHS = "%IMPORTS%"%PARAMS%);

round_trip :: ($T: Type, name: string) {
    value: T;
    bytes, error := Proto.encode(value);
    if error.kind != .NONE {
        print("% encode: %\\n", name, error.kind);
        return;
    }
    copy, decode_error := Proto.decode(T, bytes);
    if decode_error.kind != .NONE print("% decode: %\\n", name, decode_error.kind);
    Proto.free(*copy);
    Proto.free(*decode_error);
    free(bytes.data);
}

main :: () {
%BODY%
    print("OK\\n");
}
"""


@contextlib.contextmanager
def heavy(lock: str | None):
    if not lock:
        yield
        return
    while True:
        try:
            os.mkdir(lock)
            break
        except FileExistsError:
            time.sleep(5)
    try:
        yield
    finally:
        os.rmdir(lock)


def run(command: list[str], lock: str | None, cwd: Path) -> tuple[float, subprocess.CompletedProcess]:
    with heavy(lock):
        start = time.perf_counter()
        try:
            done = subprocess.run(command, cwd=cwd, capture_output=True, text=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired as error:
            done = subprocess.CompletedProcess(command, 124, "", f"timeout after {TIMEOUT}s: {error.stderr or ''}")
        return time.perf_counter() - start, done


def tail(done: subprocess.CompletedProcess, lines: int = 12) -> str:
    return "\n".join((done.stdout + done.stderr).strip().splitlines()[-lines:])


def measure_variant(variant: dict, arguments, cache: Path, out: Path) -> dict:
    name = variant["name"]
    work = out / name
    work.mkdir(parents=True, exist_ok=True)
    set_root = cache / variant["set"]
    roots = [set_root / p for p in variant.get("import_paths", [])]
    if variant.get("with_descriptor"):
        # google/protobuf/descriptor.proto is not built in: supply the pinned file from the cache.
        roots.append(cache / "descriptor")
    imports = ";".join(str(p.resolve()) for p in roots)
    if "dir" in variant:
        source = f'DIR = "{set_root / variant["dir"]}"'
        probe_source = ["--dir", str(set_root / variant["dir"])]
    else:
        source = f'PATH = "{set_root / variant["path"]}"'
        probe_source = ["--path", str(set_root / variant["path"])]
    body = "\n".join(f'    round_trip(Api.{t}, "{t}");' for t in variant["types"])
    program = work / "prog.jai"
    program.write_text(
        PROGRAM.replace("%SOURCE%", source).replace("%IMPORTS%", imports).replace("%PARAMS%", arguments.params).replace("%BODY%", body)
    )
    jaic = arguments.jaic
    threads = ["-threads", str(arguments.threads)]
    result: dict = {"variant": name, "set": variant["set"]}

    # Generation alone, in the interpreter.
    emitted = work / "emitted.jai"
    wall, done = run(
        [jaic, *threads, "-interp", str(HERE / "gen_probe.jai"), "--", *probe_source,
         "--import-paths", imports, "--out", str(emitted)],
        arguments.lock, work)
    match = re.search(r"gen_ms=(\S+) bytes=(\d+)", done.stdout)
    result["gen_wall_s"] = round(wall, 2)
    if match:
        result["gen_ms"] = round(float(match.group(1)), 1)
        result["emitted_bytes"] = int(match.group(2))
    else:
        result["gen_error"] = tail(done)

    wall, done = run([jaic, *threads, "-check", str(program)], arguments.lock, work)
    result["check_s"] = round(wall, 2)
    result["check_ok"] = done.returncode == 0
    if done.returncode != 0:
        result["check_error"] = tail(done, 20)
        return result

    if arguments.check_only:
        return result
    exe = work / "prog"
    wall, done = run([jaic, *threads, "-O2", str(program), "-exe", str(exe)], arguments.lock, work)
    result["o2_s"] = round(wall, 2)
    if done.returncode != 0 or not exe.exists():
        result["o2_error"] = tail(done, 20)
        return result
    result["binary_bytes"] = exe.stat().st_size
    _, ran = run([str(exe)], None, work)
    result["run_ok"] = ran.returncode == 0 and ran.stdout.strip().endswith("OK")
    if not result["run_ok"]:
        result["run_output"] = tail(ran)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("variants", nargs="*")
    parser.add_argument("--jaic", required=True)
    parser.add_argument("--cache", default=os.environ.get("PROTOBUF_SCHEMAS_CACHE", DEFAULT_CACHE))
    parser.add_argument("--out", default=DEFAULT_OUT, help="scratch folder for programs and binaries")
    parser.add_argument("--threads", type=int, default=3)
    parser.add_argument("--lock", help="mkdir lock shared with other heavy jobs")
    parser.add_argument("--params", default="", help='extra module parameters, e.g. ", STRIP_ENUM_PREFIX = true"')
    parser.add_argument("--check-only", action="store_true", help="skip the -O2 build (parameter sweeps)")
    parser.add_argument("--protoc", action="store_true", help="also time protoc --cpp_out (sandboxed)")
    parser.add_argument("--cxx", action="store_true", help="with --protoc: compile the generated C++ with clang++ -O2 -c")
    parser.add_argument("--jai-protobuf", action="store_true", help="also try segcore/jai-protobuf (sandboxed)")
    parser.add_argument("--third-party-only", action="store_true", help="skip our own measurements")
    parser.add_argument("--json", help="write results here")
    arguments = parser.parse_args()
    sys.path.insert(0, str(HERE))
    import third_party

    pins = json.loads((HERE / "schemas.json").read_text())
    variants = [{**v, "set": name} for name, s in pins["sets"].items() for v in s["variants"]]
    for v in [v for v in variants if v.get("needs_descriptor")]:
        variants.append({**v, "name": v["name"] + "+descriptor", "with_descriptor": True})
    if arguments.variants:
        variants = [v for v in variants if v["name"] in arguments.variants]
    results = []
    for variant in variants:
        print(f"== {variant['name']}", file=sys.stderr, flush=True)
        cache, out = Path(arguments.cache), Path(arguments.out)
        row = ({"variant": variant["name"], "set": variant["set"]} if arguments.third_party_only
               else measure_variant(variant, arguments, cache, out))
        # The third-party tools get the same files as ours, with descriptor.proto and friends available.
        if arguments.protoc:
            row.update(third_party.measure_protoc(variant, arguments, cache, out / variant["name"]))
        if arguments.jai_protobuf:
            row.update(third_party.measure_jai_protobuf(variant, arguments, cache, out / variant["name"]))
        results.append(row)
        print(json.dumps(results[-1]), flush=True)
    if arguments.json:
        Path(arguments.json).write_text(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
