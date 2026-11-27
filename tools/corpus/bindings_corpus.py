#!/usr/bin/env python3
"""Checks Bindings_Generator's struct sizes, alignments and field offsets against clang for pinned C libraries.

For every library and triple in corpus/bindings/libraries.json: fetch the pinned archive, generate
bindings with tools/corpus/bindings-corpus/generate.jai, ask clang for the layout of every record the
generator names (names from `-ast-dump=json`, numbers from `sizeof`/`_Alignof`/`offsetof` compiled to
LLVM IR), and compare them.
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


def top_level_nodes(path: Path, chunk: int = 1 << 24):
    """The children of the translation unit in a `-ast-dump=json` file, one at a time.

    A library's dump is hundreds of megabytes, so it is neither loaded whole nor kept: each child is
    decoded from a bounded buffer and handed out before the next is read. Raises `ValueError` when the
    file is not a translation unit's dump."""
    decoder = json.JSONDecoder()
    separators = re.compile(r"[ \t\r\n,]*")
    # The unit's own keys come first and hold no `inner`; its children follow the first one.
    marker = '"inner": ['
    with open(path, encoding="utf-8") as handle:
        buffer = handle.read(chunk)
        while marker not in buffer:
            more = handle.read(chunk)
            if not more:
                raise ValueError("the AST dump has no translation unit children")
            buffer += more
        position = buffer.index(marker) + len(marker)
        while True:
            position = separators.match(buffer, position).end()
            if position >= len(buffer):
                more = handle.read(chunk)
                if not more:
                    raise ValueError("the AST dump ends before its children close")
                buffer = buffer[position:] + more
                position = 0
                continue
            if buffer[position] == "]":
                return
            try:
                node, position = decoder.raw_decode(buffer, position)
            except json.JSONDecodeError:
                more = handle.read(chunk)
                if not more:
                    raise ValueError("the AST dump ends inside a node") from None
                buffer = buffer[position:] + more
                position = 0
                continue
            yield node


def typedef_record(node: dict):
    """The id of the record a typedef names directly, `None` for a pointer, array or other typedef.

    `typedef struct S S_t;` and `typedef struct {...} S_t;` name the record; `typedef struct S *SP;`
    names a pointer type and is not a record called `SP`."""
    inner = node.get("inner") or []
    child = inner[0] if inner else None
    while isinstance(child, dict) and child.get("kind") == "ElaboratedType":
        children = child.get("inner") or []
        child = children[0] if children else None
    if isinstance(child, dict) and child.get("kind") == "RecordType":
        return (child.get("decl") or {}).get("id")
    return None


def clang_records(nodes) -> dict:
    """`{name: {"spelling", "fields"}}` for each complete record among the translation unit's children
    (`top_level_nodes`).

    A tagged record is named by its tag and spelled `struct tag`. A typedef of a record names it and is
    spelled by the typedef, which is the only way to name a typedef'd anonymous record. `fields` are the
    named direct members that are not bitfields, so `offsetof` accepts them. Only what these need is kept
    of each node."""
    complete = {}
    typedefs = []
    for node in nodes:
        kind = node.get("kind")
        if kind == "RecordDecl" and node.get("completeDefinition") and node.get("id"):
            fields = [
                member["name"] for member in node.get("inner") or []
                if member.get("kind") == "FieldDecl" and member.get("name") and not member.get("isBitfield")
            ]
            complete[node["id"]] = {"tag": node.get("name"), "keyword": node.get("tagUsed") or "struct", "fields": fields}
        elif kind == "TypedefDecl" and node.get("name"):
            typedefs.append((node["name"], typedef_record(node)))
    records = {}
    for info in complete.values():
        if info["tag"]:
            records[info["tag"]] = {"spelling": f"{info['keyword']} {info['tag']}", "fields": info["fields"]}
    for name, ref in typedefs:
        if ref in complete and name not in records:
            records[name] = {"spelling": name, "fields": complete[ref]["fields"]}
    return records


PROBE = "jai_probe"


def probe_source(header: str, records: dict, generated: dict) -> tuple[str, list]:
    """A C file that includes `header` and holds `sizeof`, `_Alignof` and `offsetof` of every record the
    generator asserts on. The second result lists what each array element answers, as `(record, kind, field)`.

    The numbers come from compiled constants, not `-fdump-record-layouts`: the dump forces a record's
    layout at its closing brace, before a trailing `__attribute__((packed))` applies, and so reports
    sizes the compiler does not use."""
    order = []
    values = []
    # With nothing to ask, `order` is empty and the source is not worth compiling (see `run_cell`).
    for name, gen in sorted(generated.items()):
        record = records.get(name)
        if record is None:
            continue
        spelling = record["spelling"]
        order += [(name, "size", None), (name, "align", None)]
        values += [f"sizeof({spelling})", f"_Alignof({spelling})"]
        for field in sorted(gen["fields"]):
            if field in record["fields"]:
                order.append((name, "offset", field))
                values.append(f"__builtin_offsetof({spelling}, {field})")
    body = ",\n  ".join(values)
    return f'#include "{header}"\nconst unsigned long long {PROBE}[] = {{\n  {body}\n}};\n', order


def probe_layouts(ir: str, order: list) -> dict:
    """`{record: {"size", "align", "fields": {name: offset}}}` from the LLVM IR of a `probe_source`."""
    found = re.search(rf"@{PROBE} = [^\[]*\[\d+ x i64\] \[(.*?)\]", ir, re.DOTALL)
    if not found:
        raise ValueError("the probe's constant array is missing from the compiler output")
    numbers = [int(number) for number in re.findall(r"i64 (-?\d+)", found.group(1))]
    if len(numbers) != len(order):
        raise ValueError(f"the probe holds {len(numbers)} values for {len(order)} questions")
    layouts: dict = {}
    for (record, kind, field), number in zip(order, numbers):
        entry = layouts.setdefault(record, {"fields": {}})
        if kind == "offset":
            entry["fields"][field] = number
        else:
            entry[kind] = number
    return layouts


@dataclasses.dataclass
class Comparison:
    records: int = 0           # records the generator names that clang also has a layout for
    fields: int = 0            # fields compared
    unmatched: list = dataclasses.field(default_factory=list)  # generator records clang has no layout for
    mismatches: list = dataclasses.field(default_factory=list)


def compare(generated: dict, oracle: dict) -> Comparison:
    """Compare the generator's records with clang's by name. A record clang has no layout for is
    unmatched, never silently passed."""
    result = Comparison()
    for record, gen in sorted(generated.items()):
        ref = oracle.get(record)
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


def run_sandboxed(command: list[str], scratch: Path, reads: list[Path], sandboxed: bool, stdout_to: Path | None = None) -> subprocess.CompletedProcess:
    """Run one generator or oracle command. It starts in the scratch directory: the repository as
    working directory is not readable inside the sandbox, and nothing here should depend on it.
    `stdout_to` sends standard output to a file instead of memory (`stdout` is then empty)."""
    env = None
    if sandboxed:
        box = usb.make_sandbox(reads=reads, writes=[scratch])
        command, env = usb.confine(box, scratch, command, None)
    if stdout_to is None:
        return subprocess.run(command, capture_output=True, text=True, env=env, timeout=600, cwd=scratch)
    with open(stdout_to, "w", encoding="utf-8") as sink:
        return subprocess.run(command, stdout=sink, stderr=subprocess.PIPE, text=True, env=env, timeout=600, cwd=scratch)


def run_cell(manifest: dict, lib: dict, triple: str, generator: Path, clang: str, sandboxed: bool) -> dict:
    """Generate and check every listed header of one library at one triple.

    Each listed header is one generation and one pair of clang runs (its AST, then a layout probe), so a
    failure names its header. Records are compared per header. A header that fails to generate, or whose
    clang output cannot be read, fails the cell with an `ERROR` naming it."""
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
            generated = generated_layouts(out.read_text())
            frontend = [clang, "-x", "c", "-w", f"--target={triple}", *extra]
            for directory in dirs["includes"]:
                frontend += ["-I", directory]
            for directory in dirs["system"]:
                frontend += ["-isystem", directory]
            ast_path = scratch / f"{Path(header).stem}-ast.json"
            tree_dump = run_sandboxed([*frontend, "-fsyntax-only", "-Xclang", "-ast-dump=json", source], scratch, reads, sandboxed, stdout_to=ast_path)
            if tree_dump.returncode != 0:
                return failed(result, "error", f"{header}: clang oracle: {first_error(tree_dump.stderr)}", tree_dump.stderr)
            try:
                records = clang_records(top_level_nodes(ast_path))
            except (ValueError, KeyError, TypeError, AttributeError) as error:
                return failed(result, "error", f"{header}: clang AST dump: {error}", repr(error))
            finally:
                ast_path.unlink(missing_ok=True)
            source_text, order = probe_source(source, records, generated)
            if not order:
                # A header that binds only functions or macros has no record to ask clang about, and an
                # empty initializer is not C.
                total.unmatched += [f"{header}: {name}" for name in sorted(generated)]
                continue
            probe = scratch / f"{Path(header).stem}-probe.c"
            probe.write_text(source_text)
            compiled = run_sandboxed([*frontend, "-S", "-emit-llvm", "-o", "-", str(probe)], scratch, reads, sandboxed)
            if compiled.returncode != 0:
                text = compiled.stdout + compiled.stderr
                return failed(result, "error", f"{header}: clang oracle: {first_error(text)}", text)
            try:
                oracle = probe_layouts(compiled.stdout, order)
            except (ValueError, KeyError) as error:
                return failed(result, "error", f"{header}: clang layout probe: {error}", repr(error))
            comparison = compare(generated, oracle)
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
