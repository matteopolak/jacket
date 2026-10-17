# Language server

## What it is

`crates/jai-language-server` is one JSON-RPC language server used by native editors (the `jailsp` binary, with `Content-Length` stdio framing) and the browser playground (the same `JsonSession` linked into `jai_wasm.wasm`).

Two layers:

- **Syntax** (always on): lexer/parser diagnostics, semantic tokens, symbols, folding, selection ranges, format-string hover, `#load`/`#import` links. Built from the `jaic` lexer and parser; works while the text does not parse.
- **Semantic** (when the session has an `Environment`): everything needing the type checker: hover, completion, definition, references, rename, signature help, inlay hints, code actions, call hierarchy, code lenses, and [jailint](../tools/jailint.md) lints with quick fixes. Open documents are compiled with `jaic` and the compiler's editor facts are queried.

Not supported: formatting (use `jaifmt --stdin` as an external formatter), type errors in a polymorphic procedure nobody instantiates (sketches cover hover only), unopened files in workspace diagnostics, bare-name uses of `using`'d fields in references/rename.

## How it works

### Documents

`Session` holds versioned open documents. Each version is lexed and parsed (`analysis.rs`) into declaration rows. Positions are UTF-16. A stale version or invalid edit keeps the previous text. If a version does not parse, completion falls back to the last version that did.

Half-typed text rarely parses, so `session.rs` `repair` blanks lines with spaces (byte offsets must not move) until it parses: first the cursor's line, then the line the parser reports. Every semantic feature goes through `Session::checked`, so hover, hints and tokens of one version share one compile.

### Semantic analysis (`semantic.rs`)

Compiles on demand and caches by source text:

1. Open documents overlay the file system (`OverlayFs`; disk plus repo stdlib natively, the bundled `VirtualFs` in the browser).
2. The root is the open document that `#load`s the requested one, else the project entry (`jai.toml` `build_files`, or an inferred `build.jai`/`first.jai`/`main.jai`/`src/main.jai`).
3. `Compiler::ide` is set to `IdeFacts`. Sema records what each identifier names, scopes, expansions and calls. Every hook is a no-op when `ide` is `None`.
4. `ide_check_all` lowers every non-polymorphic body, not only what `main` reaches, and keeps each declaration's error separately (so several errors per file show).
5. `ide_sketch_polymorphs` checks each uninstantiated polymorphic procedure and unexpanded macro once with stand-in types so hover and completion work inside them. The sketch is then discarded; jailint ignores its facts.
6. Compile-time code runs in `SandboxHost` with a block budget, so a looping `#run` traps instead of hanging and never writes to protocol stdout.

Code in an `#if OS == ...` branch the target skipped is checked by compiling for the other targets (`OTHER_TARGETS`) until one takes it.

**Build metaprograms.** For a project built by a metaprogram (`first.jai` calling `add_build_file`), the metaprogram is run and each workspace compiler becomes a `Program` in `Analysis::others`; `Analysis::focus(path)` picks the one that compiled a file. Afterwards `release_build_state` and `release_lowered_code` free metaprogram state and lowered IR, since nothing runs again.

**Build facts.** Project discovery (which program a file is in, which module folders it has) follows what the metaprogram *did*. `jaic::build::WorkspaceObserver::source_added` reports each `add_build_file`/`add_build_string`; `semantic::compile` turns them, plus the `import_path` the workspace ended up with, into `BuildFacts` (canonical keys, see [file identity](file-identity.md)), so computed paths (`tprint` + `#filepath`, variables, loops) are exact. `Cache::recorded` keeps the last run that finished without errors per build file; a failed or budget-exhausted run never replaces it. `Session::sync_builds` hands them to `Index::builds`; `Index::closure` and `modules::find_import_dirs` prefer them over the static scan (`add_build_file` literals, `declared_import_paths`), which still applies before the first run. When no entry's loads reach a file, `Session::run_unrecorded_builds` runs the `jai.toml` build files once; `Index::probed` keys that on a hash of the metaprogram, its loads and `jai.toml`, so editing `main.jai` does not rerun `build.jai`. Folders a past run added are not handed back to its next run, so deleting an `import_path` line takes effect. Runs happen in `SandboxHost` (writes stay in memory, no processes or network) under `BLOCK_BUDGET`, the same budget as any other metaprogram run. Tests: `tests/build_facts.rs`.

### Cache and memory

A large program compile is hundreds of MiB, so `semantic::Cache` keeps `CACHED` (2) compiles, plus up to `ALTERNATES` for other targets. It keeps one compile per program, preferring the newest one that checked and read the most source, so a half-typed edit does not evict the last good compile. Rules to keep:

- `jailsp` installs `large_alloc::LargeBlocksToOs` (blocks of 128 KiB or more via `mmap`) and trims after each compile; this is the crate's only `unsafe`.
- Answers that grow with the file (`documentSymbol`, semantic tokens) are written straight to text (`JsonSession::raw_result`), not built as `serde_json::Value`.
- Per-request work must be cached on `Document` or by position index (`IdeFacts::refs_at`, `member_use_at`), never rescan the file. Drop references with `IdeFacts::truncate_refs`.
- Compiles share parse trees through `jaic::parser::ParseReuse`. Trees must never be mutated after parsing.
- Push diagnostics are debounced (200 ms) in `jailsp` so a completion request does not wait behind a compile. Clients advertising `textDocument.diagnostic` get pull diagnostics instead and no pushes.
- Completion does not compile while a word is typed: it reuses a compile whose text matches up to the cursor expression (`Cache::compiled_before`).

### Features worth knowing

- **Completion**: `ide_scope_at` finds the scope; `ide_visible` lists names, `ide_members` lists members after `a.b.`. The typed word is cut out so the probe text is stable across keystrokes. An inferred `.` completes the expected enum's members. Items carry a `textEdit` covering the whole word. `#asm` blocks have their own completion, hover and signature help (`asm.rs`, instruction data from `jaic::sema::asm_catalog`; see [asm](asm.md)).
- **Auto-import**: names from stdlib modules, project modules and unloaded project files are offered with an `additionalTextEdits` `#import`/`#load`. `auto_import.rs` builds items, `exports.rs` scans declarations from tokens, `project.rs` finds the program. Max 50 items; needs 2+ characters. Disable with `completion.autoImport: false`.
- **Hover**: built as `hover::Block`s and written as Markdown or plain text depending on the client's `contentFormat`. Shows types, overloads, macro expansions, `#run` values and output, `#if` results, format-string arguments, and memory layout (size, align, padding, field offsets from the compiler's own layout). Doc comments appear under signatures ([doc comments](doc-comments.md)).
- **Definition/references/rename**: from recorded references. Struct fields and enum members are `(use span, declaration span)` pairs in `IdeFacts::member_uses`. Rename refuses declarations outside project files.
- **Missing imports**: the compiler attaches module suggestions to the "undeclared identifier" diagnostic (`Diagnostic::fixes.imports`); `imports.rs` turns them into "Add `#import`" quick fixes. Ranking is in `sema/suggestions.rs`, placement in `imports::insertion`.
- **Lints** (`lints.rs`): run on documents that parse, from real text. `jailint.toml` (nearest above, or an open copy sent via `didOpen`) sets levels. Quick fixes carry the rule in `data.rule`; `source.fixAll.jailint` applies all machine-applicable fixes.
- **Expansions**: `jai.showExpansion`, `jai/expansion` and `jai-expansion:` URIs show what an `#insert`, `#run` or macro produced; `jai/source` returns text for any URI the client has not opened.
- **Signature help**: `open_call` scans back to the unmatched `(`; recorded call facts are used when present, else the callee is looked up in scope.
- **Semantic tokens**: the legend is append-only. Here-strings tagged with a language (`#string WGSL`) get no token so the editor can highlight them; `LANGUAGE_TAGS` in `here_string.rs` must match `EMBEDDED_LANGUAGES` in `editors/vscode/scripts/build-grammar.ts`.

### Project settings

`jai.toml` at the project root:

```toml
build_files = ["first.jai", "src/main.jai"]   # entries (what jaic is given)
import_path = ["vendor/modules"]              # extra module folders, like -import_dir
```

Unknown keys make the file invalid (treated as empty, reported once with `window/showMessage`). Without it, entries are inferred from the workspace folder. Module search order: `modules/` next to the root file and each folder above it up to the project base, then `import_path`, then string literals in entry files that mention `import_path` (`modules.rs`; nothing is run), then the stdlib. A module file not reached by any `#load` is checked in the context of the entry that imports it (`Sources::closure_with_modules`).

### Protocol

`JsonSession::handle_json` runs each message under `catch_unwind`; a panic becomes an `InternalError`, caches are reset and open documents survive (not on `panic = "abort"` builds such as wasm). Non-standard requests: `jai/source`, `jai/expansion`. Documents use absolute `file:///` URIs (the browser uses `file:///jai-script/<name>`). The wasm bridge (`crates/jai-wasm/src/language_server.rs`) keeps the session in a `thread_local` because compiler state uses `Rc`; the worker message is `{type: "lsp", id, message}`.

## How to change it

- **Record more facts**: add a sema hook calling an `ide_*` method guarded by `self.ide.is_some()`; keep it cheap. Name facts go in `sema/ide.rs`, call and metaprogramming facts in `ide_meta.rs`. Extend `ide_hover`/`ide_entity_hover` to show them.
- **New hover content**: add a `hover::Block` kind with cases in `HoverText::plain` and `HoverText::markdown`.
- **New block scope**: call `ide_scope_span`, or completion inside it sees only the enclosing scope.
- **New expansion kind**: add an `IdeExpansionKind`, record it where the compiler evaluates it, handle it in `kind_name`, `describe`, `expansion` and `code_actions` (`features.rs`).
- **Print-family names** are the list `format::PRINT_FAMILY` (diagnostics publish without compiling); keep format semantics in step with `stdlib/Basic/Print.jai` and `jailint::format_string`.
- **Token legend**: append to `TOKEN_TYPES`/`TOKEN_MODIFIERS` in `lib.rs` and `semantic_tokens_wire`; never reorder.
- **Environment**: native `main.rs` `native_environment`, browser `language_server.rs` `environment`; keep both in step with how `jaic` and the playground compile.
- **Lint rules** live in `crates/jailint`; the server picks them up unchanged.
- **Typed results** live in `model.rs`; extend them and the wire mapping in `protocol.rs` together.

Tests are in `crates/jai-language-server/tests/` (`semantic.rs`, `features.rs`, `auto_import.rs`, `links.rs`, `protocol.rs`, `resilience.rs`, plus refactor and hierarchy tests) and in `crates/jai-wasm/src/language_server.rs`. `node tools/check_scripting_wasm.mjs <jai_wasm.wasm>` runs the same against the real wasm module.

## Configuration

```sh
cargo run -p jai-language-server --bin jailsp
cargo test -p jai-language-server
cargo build --release -p jai-wasm --target wasm32-unknown-unknown
node tools/check_scripting_wasm.mjs target/wasm32-unknown-unknown/release/jai_wasm.wasm
```

- `JAIC_STDLIB` overrides the native stdlib directory.
- `jailint.toml`, `jai.toml`: nearest above the document.
- `initializationOptions` / `workspace/didChangeConfiguration` (`jai` section): `completion.autoImport` (default `true`), `trusted` (default `true`; `false` runs no compile-time code, see [workspace trust](../tools/workspace-trust.md)). `jailsp --untrusted` starts untrusted; the `jai/setTrusted` notification switches at runtime.
- Constants: `semantic.rs` (`BLOCK_BUDGET`, `CACHED`, `ALTERNATES`, `PARSES_KEPT`), `auto_import.rs` (`MAX_ITEMS`, `MIN_PREFIX`, `MAX_PROJECT_FILES`), `ide_meta.rs` (`MAX_EXPANSIONS`, `MAX_CALLS`, `MAX_VARIANTS`, `MAX_TEXT`), `features.rs` (`HINT_CHARS`).
- `Limits` are safety caps (512 documents, 32 MiB per document, 64 MiB per message); `Limits::browser()` is smaller. A document over a cap is not opened and the client gets a `window/showMessage`.
- Benchmark: `python3 tools/lsp_bench.py` ([benchmarks](../tools/benchmarks.md)).

## Dependencies

`jaic` (lexer, parser, sema with `IdeFacts`, `SandboxHost`), `jailint`, `serde`/`serde_json`. In the browser it is reached through `engine.lsp(message)` ([playground](../browser/playground.md)). Protocol: LSP 3.17 over JSON-RPC 2.0.
