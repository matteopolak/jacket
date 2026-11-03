# Language server

`crates/jai-language-server` is one JSON-RPC server: the `jailsp` binary for native editors (stdio, `Content-Length` framing) and the same `JsonSession` inside `jai_wasm.wasm` for the browser playground.

Two layers:

- **Syntax** (always on): lexer and parser diagnostics, semantic tokens, symbols, folding, selection ranges, `#load`/`#import` links. Works while the text does not parse.
- **Semantic** (when the session has an `Environment`): hover, completion, definition, references, rename, signature help, inlay hints, code actions, call hierarchy, code lenses, and [jailint](../tools/jailint.md) lints. Open documents are compiled with `jaic` and the compiler's editor facts are queried.

Formatting is not built in; use `jaifmt --stdin` as an external formatter.

## How it works

- `Session` holds versioned open documents. Each version is lexed and parsed once into declaration rows; positions are UTF-16, converted in `position::LineIndex::byte` only.
- Half-typed text is compiled as it stands. The editor lexer and parser recover at line and grammar boundaries, so nothing is patched first.
- `semantic.rs` compiles on demand and caches by source text. Open documents overlay the file system (`OverlayFs`). The root is the open document that `#load`s the file, else the project entry.
- With `compiler.ed` recording, sema writes what each name refers to into `IdeFacts` ([IDE facts](ide-facts.md)). `ide_check_all` lowers every non-polymorphic body; `ide_sketch_polymorphs` checks uninstantiated polymorphs once with stand-in types so hover works inside them.
- Compile-time code runs in `SandboxHost` under a block budget and a wall-clock limit ([limits](compile-time-limits.md)), so a looping `#run` traps at its own site.
- Code in an `#if OS == ...` branch the target skipped is checked by compiling for other targets in `OTHER_TARGETS` order.
- Projects built by a metaprogram are run; each workspace compiler becomes a `Program`, and `Analysis::focus(path)` picks the one that compiled a file.
- Config files apply on `didSave`, not while typing ([saved configuration](saved-configuration.md)). Generic lenses aggregate real instances ([polymorph reporting](polymorph-reporting.md)).

### Cache and memory

A big compile is hundreds of MiB, so `semantic::Cache` keeps two compiles plus a few alternates for other targets, preferring the newest that checked, so a half-typed edit does not evict the last good one.

- Per-request work is cached on `Document` or looked up by position index; never rescan the file.
- Compiles share parse trees through `ParseReuse`. Trees must never be mutated after parsing.
- Typing in a file a program never loaded does not recompile it (`OverlayFs` records every path a compile reads).
- Completion reuses a compile whose text matches up to the cursor expression.
- Diagnostics are debounced in `jailsp`, and the check stops when input arrives (`jaic::interrupt`).
- A request never compiles (`Session::defer_compiles`, on in `jailsp`). With no compile for the text it answers from the last one, offsets moved past the edit (`semantic::Shift`), or leaves out what needs one, and queues the text for `warm_pending`, which compiles while input is quiet. Completion marks such answers `is_incomplete`; the client is asked to fetch inlay hints and semantic tokens again once the compile exists. Rename and a diagnostics pull are the exceptions: their answer is the exact result.
- Lints run after every compile, so a rule that reads other files must cache them.
- `jailsp` returns large blocks to the OS and trims after each compile; that is the crate's only `unsafe`.

### Requests without a compile

Handlers query the compile through `with_semantic` (exact text or nothing), `with_semantic_at` and `on_older_compile` (the last compile, for hover, definition, signature help and completion), so a new handler gets the rule by using them. `Session::background()` is the guard that lets a compile happen (diagnostics, `warm_pending`, rename). Tests: `tests/no_compile_in_requests.rs`; latency: `tools/bench/typing_latency.py` (`--immediate` types a key right after Enter).

### Feature list

- **Completion**: scope and members come from the compiler; names whose type fits the expected type rank first; auto-import adds the `#import`/`#load` edit; call snippets and postfix snippets (`.if`, `.not`, `.ref`) when the client supports snippets.
- **Outline** (document symbols): declarations nested under `#scope_*` sections and `#if`/`#else` blocks, with `#run`, `#insert`, `#placeholder` and `#load` listed in place; fields and members are children of their type. `analysis/outline.rs` collects the non-declaration sections, and `Session::document_symbols` nests them with the rows by the spans they cover.
- **Hints**: inferred types, argument names, `#run` values, closing-brace names, implicit `it`/`it_index`.
- **Lenses**: "Run test" and "Debug test" over `@test` procedures; compile-time cost lenses.
- **Definition, references, rename**: from recorded references. Rename refuses declarations outside project files.
- **Declaration and implementation**: declaration is the declaration that introduced the name as written (an alias, a named import), where definition follows on to what it stands for. Implementation is every procedure with a body that shares the name in the open files the file reaches (the overloads); procedure types, `#foreign` declarations and non-procedures have none. Both read the syntax layer, so they never wait for a compile.
- **Type hierarchy**: a struct's supertypes are the types its `using` and `#as using` members are declared with, its subtypes the structs in the open files that embed it (`type_hierarchy.rs`, from `Analysis::bases`). Names are matched, not resolved, and nothing waits for a compile.
- **Linked editing**: a local variable, parameter or loop name is edited together with its uses. Only names the syntax layer binds to a single local and the check's references agree on; globals, members and types are not linked, nor is anything while the text has no compile.
- **Unreachable code**: shown as a faded hint.
- **Expansions**: `jai.showExpansion` shows what an `#insert`, `#run` or macro produced.
- **Semantic tokens**: the legend is append-only (`SemanticTokenKind`). `full`, `range` (the tokens that start in the range) and `full/delta` (one edit against the last answer sent for the document; the result ids are a counter, `JsonSession::token_results`).

### Lints and quick fixes

Lints run on documents that parse. The `[lint]` table of the nearest `jacket.toml` sets levels. Quick fixes carry the rule in `data.rule`; `source.fixAll.jailint` applies every machine-applicable fix. Compiler errors carry their notes, suggestions and `help` lines, with quick fixes for suggestions.

### Project settings

```toml
[project]
entry = ["first.jai", "src/main.jai"]   # what jaic is given
import_path = ["vendor/modules"]        # extra module folders
```

Without `entry`, the server infers `build.jai`, `first.jai`, `main.jai` or `src/main.jai`.

## How to change it

- The checker never asks "am I in an editor"; it calls a policy method on the `Recorder` trait (`ide/recorder.rs`), which gives the build answer when not recording.
- To record more facts, add an event to `Recorder` or an `ide_*` call guarded by `self.ed.is_live()`. Sema never reads `IdeFacts`.
- New hover content: a `hover::Block` kind. New block scope: call `ide_scope_span`. New expansion kind: add an `IdeExpansionKind` and handle it in `features.rs`.
- New request: add it to `Request::METHODS` or `Notification::METHODS` and an arm in `JsonSession`.
- Token legend: append a `SemanticTokenKind`, never reorder.
- Print-family names (`format::PRINT_FAMILY`) must stay in step with `stdlib/Basic/Print.jai`.
- Tests: `crates/jai-language-server/tests/` and `crates/jai-wasm/src/language_server.rs`; `node tools/browser/check_scripting_wasm.mjs <wasm>` runs them against the real module.

## Configuration

- `JAIC_STDLIB`, `JAIC_PRELUDE`: see [prelude selection](prelude-selection.md).
- `jai` settings: `completion.autoImport` (true), `completion.callSnippets` (`addParentheses`, `fillArguments`, `none`), `inlayHints.closingBraceMinLines` (25, 0 off), `tests.codeLens` (true), `trusted` (true; false runs no compile-time code, see [workspace trust](../tools/workspace-trust.md)).
- Cache constants in `semantic.rs` (`CACHED`, `ALTERNATES`, ...); `Limits` caps documents, sizes and messages.
- Benchmark: `PYTHONPATH=tools python3 tools/bench/main.py lsp`.

```sh
cargo run -p jai-language-server --bin jailsp
```

## Dependencies

`jaic`, `jailint`, `serde_json`. LSP 3.17 over JSON-RPC 2.0. The browser reaches it through `engine.lsp(message)` ([playground](../browser/playground.md)).
