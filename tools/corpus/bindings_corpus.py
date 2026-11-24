#!/usr/bin/env python3
"""Checks Bindings_Generator's struct sizes, alignments and field offsets against clang for pinned C libraries.

For every library and triple in corpus/bindings/libraries.json: fetch the pinned archive, generate
bindings with tools/corpus/bindings-corpus/generate.jai, dump clang's record layouts for the same
headers and triple (`-fdump-record-layouts-complete`), and compare each record both sides name.
The upstream headers are parsed only inside tools/corpus/upstream_sandbox.py. Exits 1 on any mismatch,
generation error or manifest problem. Docs: docs/tools/bindings-corpus.md.

Usage: JAIC=<jaic> PYTHONPATH=tools python3 tools/corpus/bindings_corpus.py [--library ID]... [--triple T]...
"""

import argparse
import dataclasses
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

from toolkit import download
from toolkit.repo import ROOT
import corpus.upstream_sandbox as usb

MANIFEST = ROOT / "corpus/bindings/libraries.json"
GENERATOR = ROOT / "tools/corpus/bindings-corpus/generate.jai"
STDLIB = ROOT / "stdlib"
CACHE = download.cache_dir() / "bindings-corpus"

# The `#assert`s the generator emits for each record and field (`offset_of` takes a field name).
ASSERT = re.compile(r'^#assert\((size_of|align_of|offset_of)\((\w+)(?:, "(\w+)")?\) == (\d+)\);$', re.MULTILINE)
# A record header in clang's dump: `         0 | struct NAME` (nested records are indented further and skipped).
RECORD_HEAD = re.compile(r"^ *\d+ \| (?:struct|union) (.+?)\s*$")
# A direct member: an offset, then exactly three spaces and the declaration. Bitfield offsets (`0:3-5`) are skipped.
MEMBER = re.compile(r"^ *(\d+) \| {3}(?! )(.+)$")
RECORD_FOOT = re.compile(r"^ *\| \[sizeof=(\d+), (?:dsize=\d+, )?align=(\d+)")


class ManifestError(Exception):
    pass


def load_manifest(path: Path = MANIFEST) -> dict:
    """The manifest, checked: every library names a pinned archive and gives each triple a
    run or a reason it cannot run. Raises ManifestError with every problem found."""
    manifest = json.loads(path.read_text())
    problems = []
    triples = manifest.get("triples", {})
    ids = set()
    for lib in manifest.get("libraries", []):
        name = lib.get("id", "<no id>")
        if name in ids:
            problems.append(f"{name}: duplicate id")
        ids.add(name)
        if not re.fullmatch(r"[0-9a-f]{64}", lib.get("sha256", "")):
            problems.append(f"{name}: sha256 must be 64 lowercase hex digits")
        for key in ("url", "root", "headers", "include"):
            if not lib.get(key):
                problems.append(f"{name}: missing {key}")
        for triple, entry in lib.get("triples", {}).items():
            if triple not in triples:
                problems.append(f"{name}: {triple} is not a manifest triple")
            if "skip" in entry and not str(entry["skip"]).strip():
                problems.append(f"{name}: {triple} skipped without a reason")
        for triple in triples:
            entry = lib.get("triples", {}).get(triple, {})
            if "skip" in entry and entry.keys() - {"skip"}:
                problems.append(f"{name}: {triple} skipped but also overrides include or defines")
    if problems:
        raise ManifestError("\n".join(problems))
    return manifest


def cell_plan(lib: dict, triple: str) -> dict:
    """What a cell runs: `{"skip": reason}`, or the include directories (relative to the library root)."""
    override = lib.get("triples", {}).get(triple, {})
    if "skip" in override:
        return {"skip": override["skip"]}
    return {
        "include": [*lib["include"], *override.get("include", [])],
        "clang": [*lib.get("clang", []), *override.get("clang", [])],
    }


def generated_layouts(text: str) -> dict:
    """`{record: {"size", "align", "fields": {name: offset}}}` from the generator's `#assert`s."""
    layouts: dict = {}
    for kind, record, field, value in ASSERT.findall(text):
        entry = layouts.setdefault(record, {"fields": {}})
        if kind == "size_of":
            entry["size"] = int(value)
        elif kind == "align_of":
            entry["align"] = int(value)
        else:
            entry["fields"][field] = int(value)
    return layouts


def clang_layouts(text: str) -> dict:
    """`{record: {"size", "align", "fields": {name: offset}}}` from `-fdump-record-layouts-complete`.

    Only direct members are kept. Bitfields, anonymous members and nested record bodies are skipped,
    since the generator's asserts name fields, not bit positions."""
    layouts: dict = {}
    current = None
    anonymous = 0
    for line in text.splitlines():
        head = RECORD_HEAD.match(line)
        if head:
            current = head.group(1)
            if current.startswith("(unnamed") or current.startswith("(anonymous"):
                # A typedef'd or unnamed record: clang gives no name, so it is matched by its fields.
                anonymous += 1
                current = f"<anonymous {anonymous}>"
            layouts[current] = {"fields": {}}
            continue
        if current is None:
            continue
        foot = RECORD_FOOT.match(line)
        if foot:
            layouts[current]["size"] = int(foot.group(1))
            layouts[current]["align"] = int(foot.group(2))
            current = None
            continue
        member = MEMBER.match(line)
        if member and member.group(1).isdigit():
            declaration = member.group(2).strip()
            if declaration.startswith(("struct ", "union ")) and " " not in declaration[7:]:
                continue  # a nested record's header, not a member
            name = re.sub(r"\[.*", "", re.split(r"[\s*]+", declaration)[-1])
            if name and name.isidentifier():
                layouts[current]["fields"][name] = int(member.group(1))
    return layouts


def is_anonymous(record: str) -> bool:
    return record.startswith("<anonymous ")


@dataclasses.dataclass
class Comparison:
    records: int = 0           # records the generator names that clang also dumped
    fields: int = 0            # fields compared
    unmatched: list = dataclasses.field(default_factory=list)  # generator records clang did not dump
    mismatches: list = dataclasses.field(default_factory=list)


def compare(generated: dict, oracle: dict) -> Comparison:
    """Compare the generator's records with clang's. A record is matched by its C tag; a typedef'd
    anonymous record in clang's dump has no tag, so it matches the one anonymous record with exactly
    the same field names. Zero or several candidates leave the record unmatched, never silently passed."""
    result = Comparison()
    claimed: set = set()
    for record, gen in sorted(generated.items()):
        ref = oracle.get(record)
        if ref is None:
            names = set(gen["fields"])
            candidates = [
                other for other, layout in oracle.items()
                if is_anonymous(other) and other not in claimed and names and set(layout["fields"]) == names
            ]
            if len(candidates) == 1:
                ref = oracle[candidates[0]]
                claimed.add(candidates[0])
        if ref is None:
            result.unmatched.append(record)
            continue
        result.records += 1
        for key in ("size", "align"):
            if key in gen and gen[key] != ref.get(key):
                result.mismatches.append(f"{record}: {key} generator {gen[key]} clang {ref.get(key)}")
        for field, offset in sorted(gen["fields"].items()):
            if field not in ref["fields"]:
                continue
            result.fields += 1
            if offset != ref["fields"][field]:
                result.mismatches.append(f"{record}.{field}: offset generator {offset} clang {ref['fields'][field]}")
    return result


def extract(archive: Path, root: str) -> Path:
    """The archive's tree under the cache, extracted once; `root` is its single top-level directory."""
    target = CACHE / "trees" / archive.name
    if (target / root).is_dir():
        return target / root
    CACHE.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix="extract-", dir=CACHE))
    try:
        with tarfile.open(archive) as tar:
            tar.extractall(staging, filter="data")
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            shutil.rmtree(target)
        staging.rename(target)
    finally:
        shutil.rmtree(staging, ignore_errors=True)
    return target / root


def library_tree(lib: dict) -> Path:
    archive = download.pinned(lib["url"], lib["sha256"], f"bindings-corpus/{lib['id']}-{lib['version']}.tar.gz")
    return extract(archive, lib["root"])


def build_generator(jaic: str) -> Path:
    """The generator program, compiled with the jaic given (JAIC or PATH)."""
    binary = CACHE / "bin" / "generate"
    binary.parent.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "JAIC_STDLIB": str(STDLIB)}
    subprocess.run([jaic, str(GENERATOR), "-exe", str(binary)], check=True, env=env)
    return binary


def include_flags(manifest: dict, lib: dict, plan: dict, tree: Path) -> dict:
    """The directories a cell compiles against: the library's own, its corpus config, and the corpus libc."""
    includes = [str(tree / directory) for directory in plan["include"]]
    if lib.get("config"):
        includes.insert(0, str(ROOT / manifest["config"] / lib["config"]))
    return {"includes": includes, "system": [str(ROOT / manifest["libc"])]}


def run_sandboxed(command: list[str], scratch: Path, reads: list[Path], sandboxed: bool) -> subprocess.CompletedProcess:
    """Run one generator or oracle command. It starts in the scratch directory: the repository as
    working directory is not readable inside the sandbox, and nothing here should depend on it."""
    env = None
    if sandboxed:
        box = usb.make_sandbox(reads=reads, writes=[scratch])
        command, env = usb.confine(box, scratch, command, None)
    return subprocess.run(command, capture_output=True, text=True, env=env, timeout=600, cwd=scratch)


def run_cell(manifest: dict, lib: dict, triple: str, generator: Path, clang: str, sandboxed: bool) -> dict:
    """Generate and check every listed header of one library at one triple.

    The generator binds the declarations of each source file it is given, not those of the headers
    that file includes, so each listed header is one source and one clang translation unit. Records
    are compared per header; a header that fails to generate fails the cell."""
    plan = cell_plan(lib, triple)
    result = {"library": lib["id"], "triple": triple}
    if "skip" in plan:
        result.update(status="skipped", reason=plan["skip"])
        return result
    tree = library_tree(lib)
    dirs = include_flags(manifest, lib, plan, tree)
    # A prelude header is force-included before each listed header (FreeRTOS.h before task.h).
    prelude = [flag for header in lib.get("prelude", []) for flag in ("-include", str(tree / header))]
    extra = [*manifest["triples"][triple].get("clang", []), *plan["clang"], *prelude]
    reads = [ROOT / manifest["libc"], ROOT / manifest["config"], tree, STDLIB, CACHE]
    total = Comparison()
    with tempfile.TemporaryDirectory(prefix=f"bindings-{lib['id']}-{triple}-") as scratch_name:
        scratch = Path(scratch_name)
        for header in lib["headers"]:
            source = str(tree / header)
            out = scratch / f"{Path(header).stem}.jai"
            generate = [str(generator), "--out", str(out), "--triple", triple, "--os", manifest["triples"][triple]["os"]]
            for directory in dirs["includes"]:
                generate += ["--include", directory]
            for directory in dirs["system"]:
                generate += ["--system", directory]
            for flag in extra:
                generate += ["--clang", flag]
            generate += ["--", source]
            gen = run_sandboxed(generate, scratch, reads, sandboxed)
            if gen.returncode != 0 or not out.exists():
                return failed(result, "error", f"{header}: {first_error(gen.stdout + gen.stderr)}", gen.stdout + gen.stderr)
            oracle = [clang, "-x", "c", "-fsyntax-only", "-w", "-Xclang", "-fdump-record-layouts-complete", f"--target={triple}", *extra]
            for directory in dirs["includes"]:
                oracle += ["-I", directory]
            for directory in dirs["system"]:
                oracle += ["-isystem", directory]
            dump = run_sandboxed([*oracle, source], scratch, reads, sandboxed)
            if dump.returncode != 0:
                text = dump.stdout + dump.stderr
                return failed(result, "error", f"{header}: clang oracle: {first_error(text)}", text)
            comparison = compare(generated_layouts(out.read_text()), clang_layouts(dump.stdout + dump.stderr))
            total.records += comparison.records
            total.fields += comparison.fields
            total.unmatched += [f"{header}: {name}" for name in comparison.unmatched]
            total.mismatches += [f"{header}: {message}" for message in comparison.mismatches]
    if total.records == 0:
        return failed(result, "error", "no generated record matched a clang record: nothing was checked", "")
    result.update(
        status="mismatch" if total.mismatches else "ok",
        records=total.records,
        fields=total.fields,
        unmatched=total.unmatched,
        mismatches=total.mismatches,
    )
    return result


def failed(result: dict, status: str, first: str, text: str) -> dict:
    result.update(status=status, first_error=first, error=text.strip()[-2000:])
    return result


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--library", action="append", default=[], help="library id (repeatable)")
    parser.add_argument("--triple", action="append", default=[], help="target triple (repeatable)")
    parser.add_argument("--json", type=Path, help="write the full report here")
    parser.add_argument("--no-sandbox", action="store_true", help="run upstream headers unconfined (not in CI)")
    args = parser.parse_args(argv)

    manifest = load_manifest()
    jaic = os.environ.get("JAIC") or shutil.which("jaic")
    if not jaic:
        raise SystemExit("set JAIC to the jaic built from this checkout")
    clang = shutil.which("clang")
    if not clang:
        raise SystemExit("clang is required for the layout oracle")
    sandboxed = usb.policy(args.no_sandbox)
    generator = build_generator(jaic)

    libraries = [lib for lib in manifest["libraries"] if not args.library or lib["id"] in args.library]
    triples = [t for t in manifest["triples"] if not args.triple or t in args.triple]
    results = []
    for lib in libraries:
        for triple in triples:
            result = run_cell(manifest, lib, triple, generator, clang, sandboxed)
            results.append(result)
            print(format_line(result), flush=True)
    if args.json:
        args.json.write_text(json.dumps(results, indent=2) + "\n")
    failed = [r for r in results if r["status"] in ("error", "mismatch")]
    print(f"{len(results)} cells: {sum(r['status'] == 'ok' for r in results)} ok, "
          f"{sum(r['status'] == 'skipped' for r in results)} skipped, {len(failed)} failing")
    return 1 if failed else 0


def first_error(text: str) -> str:
    """The first diagnostic in a failed run's output: the first line that names an error, else the last line."""
    lines = [line for line in text.splitlines() if line.strip()]
    for line in lines:
        if "error" in line or "fatal" in line:
            return line.split("Bindings_Generator: ", 1)[-1]
    return lines[-1].split("Bindings_Generator: ", 1)[-1] if lines else "no output"


def format_line(result: dict) -> str:
    cell = f"{result['library']:<11} {result['triple']:<24}"
    if result["status"] == "skipped":
        return f"{cell} skip  {result['reason']}"
    if result["status"] == "error":
        return f"{cell} ERROR {result['first_error']}"
    if result["status"] == "mismatch":
        return f"{cell} MISMATCH {len(result['mismatches'])}: " + "; ".join(result["mismatches"][:3])
    unmatched = f", {len(result['unmatched'])} unmatched" if result["unmatched"] else ""
    return f"{cell} ok    {result['records']} records, {result['fields']} fields{unmatched}"


if __name__ == "__main__":
    sys.exit(main())
