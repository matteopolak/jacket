#!/usr/bin/env python3
"""Build the starting corpora under fuzz/corpus/<target>/ from the repository's own Jai files.

Nothing here is committed: the corpora are regenerated (and grown by fuzzing, cached in CI)
instead of storing megabytes in git. Minimize afterwards with `cargo fuzz cmin <target>`.

    python3 fuzz/seed_corpus.py            # all targets
    python3 fuzz/seed_corpus.py check lsp  # some targets
    python3 fuzz/seed_corpus.py --matrix   # the target list (fuzz/targets.json) as workflow matrix JSON
"""
import hashlib
import itertools
import json
import random
import sys
from pathlib import Path

FUZZ = Path(__file__).resolve().parent
ROOT = FUZZ.parent

# Programs: self-contained tests and examples (each has its own `main`; a tree may be a glob, as
# `stdlib/**/tests` names every module's tests), and fuzz/seeds: small
# hand-written edge cases (overflowing constants, recursive types, metaprogram misuse, ...) that
# found bugs or sit next to ones that did.
PROGRAMS = ["tests/compiler/regressions", "stdlib/**/tests", "tests/corpus/positive", "tests/corpus/negative", "tests/lint", "examples", "fuzz/seeds"]
# Source text only: modules are parsed, never compiled as a main file.
SOURCES = PROGRAMS + ["stdlib"]

# Seed trees by the group names fuzz/targets.json uses. The third-party corpus is read for the
# formatter only, since nobody formatted it with jaifmt (the repository's own files are formatted
# already). The target list, input sizes and seed limits live in that manifest; compiling targets
# get small files so libFuzzer's mutations stay fast, the lexer and parser take larger ones.
TREES = {"programs": PROGRAMS, "sources": SOURCES, "sources+upstream": SOURCES + ["corpus/upstream"]}


def load_targets():
    return json.loads((FUZZ / "targets.json").read_text())


def jai_files(trees, limit):
    for tree in trees:
        for directory in sorted(ROOT.glob(tree)):
            if not directory.is_dir():
                continue
            for path in sorted(directory.rglob("*.jai")):
                if path.is_file() and path.stat().st_size <= limit:
                    yield path


def lsp_session(text):
    """JSON-RPC lines (the lsp_json target's input) opening `text` and querying it."""
    uri = "file:///workspace/main.jai"
    lines = [
        {"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
            "uri": uri, "languageId": "jai", "version": 1, "text": text}}},
    ]
    rows = text.count("\n")
    for i, method in enumerate(["textDocument/hover", "textDocument/definition",
                                "textDocument/completion", "textDocument/documentSymbol",
                                "textDocument/semanticTokens/full"]):
        params = {"textDocument": {"uri": uri}}
        if "Symbol" not in method and "semantic" not in method:
            params["position"] = {"line": min(i * 3, rows), "character": 4}
        lines.append({"jsonrpc": "2.0", "id": i + 1, "method": method, "params": params})
    lines.append({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
        "textDocument": {"uri": uri, "version": 2},
        "contentChanges": [{"range": {"start": {"line": 0, "character": 0},
                                      "end": {"line": 0, "character": 0}}, "text": "x."}]}})
    lines.append({"jsonrpc": "2.0", "id": 9, "method": "shutdown"})
    lines.append({"jsonrpc": "2.0", "method": "exit"})
    return "\n".join(json.dumps(line) for line in lines).encode()


def config_seeds(out):
    """NUL-separated `jai.toml`, `jailint.toml`, `jaifmt.toml` and `jacket.toml` texts (the
    `jacket_config` target's input): the hand-written bundles in fuzz/seeds_config plus each
    config file in the repository in its own slot."""
    slots = {"jai.toml": 0, "jailint.toml": 1, "jaifmt.toml": 2, "jacket.toml": 3}
    inputs = [p.read_bytes() for p in sorted((FUZZ / "seeds_config").glob("*.bundle"))]
    for name, slot in slots.items():
        for path in sorted(ROOT.rglob(name)):
            if "target" in path.parts or "node_modules" in path.parts:
                continue
            inputs.append(b"\0" * slot + path.read_bytes())
    for data in inputs:
        (out / hashlib.sha1(data).hexdigest()).write_bytes(data)
    return len(inputs)


def workspace_seeds(out):
    """NUL-separated `main.jai`, `a.jai` and `b.jai` (the `lsp_workspace` target's input): the
    hand-written bundles in fuzz/seeds_workspace plus the loading pairs under tests/lsp and small single-file programs."""
    inputs = [p.read_bytes() for p in sorted((FUZZ / "seeds_workspace").glob("*.bundle"))]
    for main in sorted((ROOT / "tests" / "lsp").rglob("main.jai")):
        others = sorted(p for p in main.parent.glob("*.jai") if p != main)[:2]
        inputs.append(b"\0".join(p.read_bytes() for p in [main, *others]))
    # Single-file programs: most compile cleanly, which is what a rename needs.
    inputs += [p.read_bytes() for p in itertools.islice(jai_files(PROGRAMS, 3000), 400)]
    for data in inputs:
        (out / hashlib.sha1(data).hexdigest()).write_bytes(data)
    return len(inputs)


def trim_corpus(directory, limit):
    """Delete random files from `directory` until `limit` are left (no limit: keep all).

    libFuzzer's fork mode first runs the whole corpus in one process, outside `-max_total_time`,
    and `cargo fuzz cmin` runs it again. On the slow targets (a language-server session or a
    format-and-recompile per input, a few executions per second) the seed corpus alone took 13
    minutes each, so a nightly job passed its timeout. The seeds are added again every night and
    the sample differs each time, so the whole set is still covered over several nights.
    """
    if not limit or not directory.is_dir():
        return
    files = sorted(p for p in directory.iterdir() if p.is_file())
    if len(files) <= limit:
        return
    for path in random.sample(files, len(files) - limit):
        path.unlink()
    print(f"{directory.name}: kept {limit} of {len(files)} corpus files")


def main():
    manifest = load_targets()
    args = sys.argv[1:]
    if args == ["--matrix"]:
        # The workflow's matrix: {"target": [...]}.
        print(json.dumps({"target": list(manifest)}))
        return
    if args[:1] == ["--settings"]:
        # `max_len timeout` of one target, for fuzz/run.sh.
        entry = manifest.get(args[1])
        if entry is None:
            raise SystemExit(f"unknown target {args[1]}")
        print(entry["max_len"], entry["timeout"])
        return
    if args[:1] == ["--trim"]:
        # Keep a random `corpus_max` files of a slow target's corpus (see trim_corpus).
        entry = manifest.get(args[1])
        if entry is None:
            raise SystemExit(f"unknown target {args[1]}")
        trim_corpus(FUZZ / "corpus" / args[1], entry.get("corpus_max"))
        return
    for target in args or list(manifest):
        if target not in manifest:
            raise SystemExit(f"unknown target {target}")
        if manifest[target]["seeds"] is None:
            continue
        out = FUZZ / "corpus" / target
        out.mkdir(parents=True, exist_ok=True)
        if manifest[target]["seeds"] in ("configs", "workspaces"):
            count = (config_seeds if manifest[target]["seeds"] == "configs" else workspace_seeds)(out)
            print(f"{target}: {count} seeds in {out.relative_to(ROOT)}")
            continue
        trees, limit = TREES[manifest[target]["seeds"]], manifest[target]["seed_limit"]
        count = 0
        # Saved crash inputs are good starting points for finding their neighbours.
        regressions = sorted(p for p in (FUZZ / "regressions" / target).glob("*") if p.is_file())
        for path in [*jai_files(trees, limit), *regressions]:
            data = path.read_bytes()
            if target == "lsp_json":
                data = lsp_session(data.decode("utf-8", "replace"))
            (out / hashlib.sha1(data).hexdigest()).write_bytes(data)
            count += 1
        print(f"{target}: {count} seeds in {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
