# Shared Jai language server

## What it is

`jai-language-server` provides one bounded source-analysis engine and JSON-RPC dispatcher for native LSP clients and the real `jai-wasm` browser adapter. The native `jai-lsp` binary adds standard `Content-Length` stdio framing; the portable library performs no stdio or filesystem operations.

It is built on the `jaic` lexer and parser (the same ones the compiler uses). This first protocol slice provides lexer/parser diagnostics, full semantic-token responses, source document symbols, completion, hover and definition from actual parsed declarations. It reports its analysis as source syntax. It does not execute edited `#run`, evaluate types, resolve arbitrary modules, expand generated declarations, or authorize foreign libraries.

## How it works

`Session` admits versioned documents into a closed set of open files (`VirtualSources` is an immutable snapshot of it). Each accepted version is lexed with `jaic::lexer::lex` and parsed with `jaic::parser::parse_file`; the parser stops at its first error, so a syntax error yields one diagnostic and no declaration rows for that version. Identifier spellings go through the `jaic` global interner (`Sym`), which keeps every distinct spelling for the life of the process. Reanalysis replaces the previous index.

Positions and ranges use UTF-16, including supplementary characters and CRLF. Incremental changes apply sequentially to a temporary version, then publish atomically. A stale version, split surrogate pair, incorrect `rangeLength`, invalid range or resource rejection preserves the previous text and version. URI paths are normalized in a target-independent namespace. An open file URI is a document key and never causes a disk read.

Definitions follow actual unconditional file declarations, supported lexical locals and `#load` links to other open documents. Unrelated open files do not enter lookup. File-private declarations are withheld across loads; qualified members and unsupported scope producers remain unresolved instead of selecting a similarly named global. The retained index stores `Sym` names and byte spans instead of duplicating name and declaration strings. Symbol kinds and diagnostic severity/codes are domain enums; only the JSON boundary maps them to standard LSP numbers and strings. Declaration excerpts are read from their pinned source spans when a response is requested. Document symbols retain source spans from parsed nodes; the declaration's value picks the kind (procedure, struct, enum, type, library or constant), `#scope_file` marks later top-level declarations file-private, and `#if` branches are walked. Hover shows the actual source declaration as plain text. Tokens classify compiler tokens and parsed declaration sites; they do not imply evaluated type information.

`JsonSession::handle_json` handles initialize/initialized, shutdown/exit, document open/change/close, `semanticTokens/full`, document symbols, hover, completion, definition and cancellation. Diagnostics include the current document version. Unknown notifications produce no response. Unsupported requests receive an explicit protocol error. Completed requests retain their responses; cancellation received before a queued request is dispatched returns `RequestCancelled`. Synchronous bounded analysis cannot receive another transport message during its current call.

The browser worker carries `{type: "lsp", id, message}` and returns `{type: "lsp", id, messages}`. The outer ID correlates worker transport; each standard JSON-RPC ID is preserved separately. The existing execution worker and mutable language session have separate lifetimes, so cancelling a program does not erase analysis documents. Browser exports transfer bounded scalar bytes and invoke this same Rust `JsonSession`; JavaScript does not substitute a separate language implementation.

## How to change it

Extend compiler-derived facts in `analysis.rs` (an AST walk over `jaic::ast`; keywords are plain identifiers in the `jaic` lexer, so `KEYWORDS` there classifies them) and lookup rules in `session.rs`, with source-span and ambiguity witnesses. Never add name-only guesses for evaluated records, imports, generated source or overload selection. Authentic semantic snapshots can be attached later only through the compiler's actual checked ownership boundary and a mode that refuses compile-time execution during editing.

Keep UTF-16 conversion and atomic document admission in `position.rs` and `document.rs`. Typed source results live in `model.rs`; extend its enums and the exhaustive wire mappings together. Standard JSON types and lifecycle/error conversion live in `protocol.rs`; native framing lives in `framing.rs` and stdio only in `main.rs`. Keep browser engine/worker capabilities and export tests paired with this protocol. Framing, Unicode/versioned edits, multi-file lookup, incomplete source, cancellation and a real native stdio session have focused tests.

## Configuration

```sh
cargo run --offline --locked -j 1 -p jai-language-server --bin jai-lsp
cargo test --offline --locked -j 1 -p jai-language-server
```

The server accepts LSP requests on stdin and writes only framed responses/notifications to stdout. Transport failures are printed to stderr. A shutdown request followed by an exit notification returns status zero; exiting without shutdown returns one. Configure an editor's Jai language server command to invoke `jai-lsp`; documents must use absolute local `file:///...` URIs. Browser documents normally use `file:///jai-script/<relative-name>`.

Default `Limits` admit 32 documents, 256 KiB per document, 4 MiB for text and URI storage, 1 MiB messages and 2 MiB output batches. Analysis admits 8,192 tokens, a nesting budget of 96 (bracket depth or a run of prefix operators) and 1,024 declaration rows per document. Edit batches and retained cancellation IDs are capped at 128. The conservative parser admission is a language-server budget, not a claim that larger source is invalid Jai. Token/declaration limit diagnostics explicitly state when navigation is unavailable or incomplete.

Only UTF-8 message bytes and UTF-16 LSP positions are supported. There are no formatting, rename, type inference, workspace-module search, delta-token or native-file-loading capabilities in this slice. Native tests, a real Wasm adapter run and editor interactions are separate acceptance gates; source registration does not establish them.

## Dependencies

The portable library depends on `jaic` (lexer, parser, AST). Standard JSON parsing uses `serde` and `serde_json`, pinned to the newest non-yanked releases at least 14 days old when selected. The complete generated lockfile must pass the existing dependency publication-age checker before any dependency build script runs. There is no LLVM, code generation, compiler driver or host interpreter dependency in the LSP core.

Native stdio uses Rust's standard library. The browser adapter links this core into the existing actual `jai_wasm.wasm` and uses standard worker/WebAssembly APIs. Protocol details follow the [official LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) and [JSON-RPC 2.0](https://www.jsonrpc.org/specification).
