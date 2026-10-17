#!/usr/bin/env python3
"""Build the starting corpora under fuzz/corpus/<target>/ from the repository's own Jai files.

Nothing here is committed: the corpora are regenerated (and grown by fuzzing, cached in CI)
instead of storing megabytes in git. Minimize afterwards with `cargo fuzz cmin <target>`.

    python3 fuzz/seed_corpus.py            # all targets
    python3 fuzz/seed_corpus.py check lsp  # some targets
"""
import hashlib
import json
import sys
from pathlib import Path

FUZZ = Path(__file__).resolve().parent
ROOT = FUZZ.parent

# Programs: self-contained tests and examples (each has its own `main`; a tree may be a glob, as
# `stdlib/**/tests` names every module's tests), and fuzz/seeds: small
# hand-written edge cases (overflowing constants, recursive types, metaprogram misuse, ...) that
# found bugs or sit next to ones that did.
PROGRAMS = ["tests/compiler/regressions", "stdlib/**/tests", "tests/corpus/positive", "tests/corpus/negative", "examples", "fuzz/seeds"]
# Source text only: modules are parsed, never compiled as a main file.
SOURCES = PROGRAMS + ["stdlib"]

# Per target: the trees to draw from and the largest file worth keeping. Compiling targets get
# small files so libFuzzer's mutations stay fast; the lexer and parser take larger ones.
TARGETS = {
    "lexer": (SOURCES, 32 << 10),
    "parser": (SOURCES, 32 << 10),
    "check": (PROGRAMS, 8 << 10),
    "interp": (PROGRAMS, 8 << 10),
    "lsp": (PROGRAMS, 4 << 10),
    "lsp_json": (PROGRAMS, 2 << 10),
    "lsp_edits": (PROGRAMS, 4 << 10),
    # Formatter input: any source, and the third-party corpus when it is checked out, since nobody
    # formatted it with jaifmt (the repository's own files are formatted already).
    "jaifmt": (SOURCES + ["corpus/upstream"], 8 << 10),
}


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


def main():
    targets = sys.argv[1:] or list(TARGETS)
    for target in targets:
        trees, limit = TARGETS[target]
        out = FUZZ / "corpus" / target
        out.mkdir(parents=True, exist_ok=True)
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
