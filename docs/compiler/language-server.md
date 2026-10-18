# Language server

## What it is

`crates/jai-language-server` is one JSON-RPC language server used by native editors (the `jailsp` binary, with `Content-Length` stdio framing) and the browser playground (the same `JsonSession` linked into `jai_wasm.wasm`).

Two layers:

- **Syntax** (always on): lexer/parser diagnostics, semantic tokens, symbols, folding, selection ranges, format-string hover, `#load`/`#import` links. Built from the `jaic` lexer and parser; works while the text does not parse.
- **Semantic** (when the session has an `Environment`): everything needing the type checker: hover, completion, definition, references, rename, signature help, inlay hints, code actions, call hierarchy, code lenses, and [jailint](../tools/jailint.md) lints with quick fixes. Open documents are compiled with `jaic` and the compiler's editor facts are queried.

Not supported: formatting (use `jaifmt --stdin` as an external formatter), type errors in a polymorphic procedure nobody instantiates (sketches cover hover only), unopened files in workspace diagnostics, bare-name uses of `using`'d fields in references/rename.

## How it works

### Documents

`Session` holds versioned open documents. Each version is lexed and parsed once (`analysis.rs`) into declaration rows; `Analysis` keeps the token stream (`syntax_tokens`) so features never lex the text again. Positions are UTF-16. A stale version or invalid edit keeps the previous text. If a version does not parse, completion falls back to the last version that did.

Facts that come straight from syntax are the compiler's, not re-derived here: `jaic::parser::ide_facts` gives top-level items with their visibility and `#if` nesting (`top_level`), each procedure with its notes (`procedures`, for `@PrintLike`), a header's type parameters (`type_parameters`: baked parameters and `$T`) the call around a position with the active argument (`call_at`, for named-argument completion), the cursor's context (`cursor_context`) and declaration name spans (`decl_name_spans`, `inferred_name_spans`). `analysis.rs`, `exports.rs` and `named_arguments.rs` read these; extend `ide_facts` rather than matching tokens in the server.

Half-typed text is compiled as it stands: the editor lexer and parser recover at line and grammar boundaries and keep every byte offset, so nothing is blanked or patched first. What the cursor is in (a string and the `#import`/`#load` it names, a comment, a `.` member access with its receiver chain, a declaration name) comes from `ide_facts::cursor_context`, built on the lexer; completion, path completion and inlay hints use it instead of scanning bytes. An inferred `.Member` (no receiver) compiles the text once with a placeholder name after the dot (`Session::complete_member_at`). Every semantic feature goes through `Session::checked`, so hover, hints and tokens of one version share one compile.

### Semantic analysis (`semantic.rs`)

Compiles on demand and caches by source text:

1. Open documents overlay the file system (`OverlayFs`; disk plus repo stdlib natively, the bundled `VirtualFs` in the browser).
2. The root is the open document that `#load`s the requested one, else the project entry (`jacket.toml` `[project] entry`, or an inferred `build.jai`/`first.jai`/`main.jai`/`src/main.jai`).
3. `Compiler::ide` is set to `IdeFacts`. Sema records what each identifier names, scopes, expansions and calls. Every hook is a no-op when `ide` is `None`.
4. `ide_check_all` lowers every non-polymorphic body, not only what `main` reaches, and keeps each declaration's error separately (so several errors per file show).
5. `ide_sketch_polymorphs` checks each uninstantiated polymorphic procedure and unexpanded macro once with stand-in types so hover and completion work inside them. The sketch is then discarded; jailint ignores its facts.
6. Compile-time code runs in `SandboxHost` with a block budget, so a looping `#run` traps instead of hanging and never writes to protocol stdout.

Code in an `#if OS == ...` branch the target skipped is checked by compiling for the other targets (`OTHER_TARGETS`) until one takes it.

**Build metaprograms.** For a project built by a metaprogram (`first.jai` calling `add_build_file`), the metaprogram is run and each workspace compiler becomes a `Program` in `Analysis::others`; `Analysis::focus(path)` picks the one that compiled a file. Afterwards `release_build_state` and `release_lowered_code` free metaprogram state and lowered IR, since nothing runs again.

**Build facts.** Project discovery (which program a file is in, which module folders it has) follows what the metaprogram *did*. `jaic::build::WorkspaceObserver::source_added` reports each `add_build_file`/`add_build_string`; `semantic::compile` turns them, plus the `import_path` the workspace ended up with, into `BuildFacts` (canonical keys, see [file identity](file-identity.md)), so computed paths (`tprint` + `#filepath`, variables, loops) are exact. `Cache::recorded` keeps the last run that finished without errors per build file; a failed or budget-exhausted run never replaces it. `Session::sync_builds` hands them to `Index::builds`; `Index::closure` and `modules::find_import_dirs` prefer them over the static scan (`add_build_file` literals, `FileScan::import_paths`), which still applies before the first run. When no entry's loads reach a file, `Session::run_unrecorded_builds` runs the `jai.toml` build files once; `Index::probed` keys that on a hash of the metaprogram, its loads and `jai.toml`, so editing `main.jai` does not rerun `build.jai`. Folders a past run added are not handed back to its next run, so deleting an `import_path` line takes effect. Runs happen in `SandboxHost` (writes stay in memory, no processes or network) under `BLOCK_BUDGET`, the same budget as any other metaprogram run. Tests: `tests/build_facts.rs`.

### Cache and memory

A large program compile is hundreds of MiB, so `semantic::Cache` keeps `CACHED` (2) compiles, plus up to `ALTERNATES` for other targets. It keeps one compile per program, preferring the newest one that checked and read the most source, so a half-typed edit does not evict the last good compile. Rules to keep:

- `jailsp` installs `large_alloc::LargeBlocksToOs` (blocks of 128 KiB or more via `mmap`) and trims after each compile; this is the crate's only `unsafe`.
- Answers that grow with the file (`documentSymbol`, semantic tokens) are written straight to text (`Reply::Raw` in `protocol.rs`), not built as `serde_json::Value`.
- Per-request work must be cached on `Document` or by position index (`IdeFacts::refs_at`, `member_use_at`), never rescan the file. Drop references with `IdeFacts::truncate_refs`.
- Compiles share parse trees through `jaic::parser::ParseReuse`. Trees must never be mutated after parsing.
- Push diagnostics are debounced in `jailsp`: the wait is a quarter of the last publication's time, between 15 and 200 ms (`DIAGNOSTICS_DEBOUNCE` in `main.rs`); edits that arrive during a compile are read together afterwards, so a short wait is enough for small programs. This way a completion request does not wait behind a compile. Clients advertising `textDocument.diagnostic` get pull diagnostics instead and no pushes.
- Finding a document's program must not cost a walk of the tree. `Index::graphs` keeps the direct loaders of every file under a folder (`ProjectFs::loader_graph`, built by `project::build_loader_graph`); `Index::graph_epoch` moves on only when a scan sees `#load` paths that differ from the file's last scan (`Index::note_loads`), a file is created or deleted, or the set of open and walked files changes. An edit that leaves the `#load`s alone (almost every keystroke) reuses the graph; before this, each keystroke in a stdlib file read every file of the stdlib again.
- Module lists (`Index::stdlib`, `module_dirs`) are per module: a module remembers the files it was read from, and a changed file only makes those modules stale (`ModuleCache::stale`), which `Sources::refreshed` reads again on the next ask. Do not drop a whole list on a file change.
- Lints run after every compile, so rules must not read or lex the files around the edited one again. `unused_import` keeps what it counted in a file by hash of its text (`FILES`) and `unused_parameter` keeps the definitions of a file by modification time and size (`DEFS`), both thread-local with a size cap. A rule that reads another file should cache the same way, or a lint of a large module costs more than the compile.
- Workspace symbols read the listed files by their keys (`Sources::scan_keyed`, `Sources::open`); asking `canonical_key` or `open_file_uri` for each listed path is a `realpath` per file per request.
- Completion does not compile while a word is typed: it reuses a compile whose text matches up to the cursor expression (`Cache::compiled_before`).

### Features worth knowing

- **Completion**: `ide_scope_at` finds the scope; `ide_visible` lists names, `ide_members` lists members after `a.b.`. The typed word is cut out so the probe text is stable across keystrokes. An inferred `.` completes the expected enum's members. Items carry a `textEdit` covering the whole word. `#asm` blocks have their own completion, hover and signature help (`asm.rs`, instruction data from `jaic::sema::asm_catalog`; see [asm](asm.md)).
- **Call and keyword snippets** (`call_snippets.rs`): when the client advertises `completionItem.snippetSupport`, a procedure item becomes `name($0)` / `name()$0` (`CompletionItem::call` is the `IdeCall` the checker or `exports.rs` recorded: whether every overload is parameterless, and the leading required parameter names for `fillArguments`) with the `editor.action.triggerParameterHints` command when the cursor lands inside. `call_snippets::site` reads the lexer tokens around the word to decide where parentheses do not belong (next token `(` or a declaration colon, `#procedure_of_call`, `type_of(`/`size_of(`/`type_info(`/`cast(`, prefix `*`, `::` and global `:=` initialisers, type positions after `:` `->` `$`). Statement keywords `if`/`for`/`while` expand only when they start a statement with nothing after them on the line; `case` only directly inside the braces of `... == {`. The default is `addParentheses` rather than rust-analyzer's `fill_arguments`: Jai parameters often have defaults, are variadic or are polymorphic, so placeholders would mostly be deleted again, and signature help shows the parameters anyway. Items from the syntactic fallback (no type checker) and `#asm` templates are not affected.
- **Postfix completion** (`call_snippets.rs`): after `value.` the snippets `if`, `while`, `for`, `return` (only where a statement starts), `not` (`!value`) and `ref` (`*value`) replace the receiver and the typed word (`textEdit` from the receiver's start; `filterText` is `receiver.label` so the client still matches what was typed). Offered only for a receiver the checker says is a value (a module or type receiver gets none; a call or index is a value) and only to snippet clients.
- **Closing-brace hints** (`closing_hints.rs`): `Session::inlay_hints` adds `// name` after the `}` of a procedure, struct or enum at least `closingBraceMinLines` long, from the document symbols' ranges (no type checker needed), next to `typed_hints`.
- **Auto-import**: names from stdlib modules, project modules and unloaded project files are offered with an `additionalTextEdits` `#import`/`#load`. `auto_import.rs` builds items, `exports.rs` lists a file's declarations from `ide_facts::top_level` (every name of a multi-name declaration; files with errors are scanned as far as they parse), `project.rs` finds the program. Stdlib files are offered only through their modules, even when the workspace holds the stdlib; a file inside a module folder sees that folder's files by `#load` and never `#import`s its own module. Max 50 items; needs 2+ characters. Disable with `completion.autoImport: false`.
- **Loaded fragments**: a file that nothing in its program reaches but another file `#load`s (`stdlib/internal/input/keymap-text.jai`, loaded by `Keymap/module.jai`) is analysed as part of that loader, so it sees the loader's declarations and imports. `auto_import.rs` `loader_chain` calls `jaic::project::enclosing_program` (see [project structure](project.md)) over the cached token scans of every `.jai` file under the enclosing stdlib tree (or the workspace/jai.toml folder) and walks up to a file nothing loads. Several loaders: `module.jai` first, then the shallowest path, then by name. Diagnostics shown are the fragment's own file; module roots and `#import`ed files are their own program. Nothing is compiled to pick the owner, and the folder walk and scans are cached with the auto-import index.
- **Hover**: built as `hover::Block`s and written as Markdown or plain text depending on the client's `contentFormat`. Shows types, overloads, macro expansions, `#run` values and output, `#if` results, format-string arguments, and memory layout (size, align, padding, field offsets from the compiler's own layout). Doc comments appear under signatures ([doc comments](doc-comments.md)).
- **Definition/references/rename**: from recorded references. Struct fields and enum members are `(use span, declaration span)` pairs in `IdeFacts::member_uses`. Rename refuses declarations outside project files.
- **Missing imports**: the compiler attaches module suggestions to the "undeclared identifier" diagnostic (`Diagnostic::fixes.imports`); `imports.rs` turns them into "Add `#import`" quick fixes. Ranking is in `sema/suggestions.rs`, placement in `imports::insertion`.
- **Lints** (`lints.rs`): run on documents that parse, from real text. The `[lint]` table of `jacket.toml` (nearest above, or an open copy sent via `didOpen`; legacy `jailint.toml` otherwise) sets levels. Quick fixes carry the rule in `data.rule`; `source.fixAll.jailint` applies all machine-applicable fixes.
- **Unreachable code**: `Session::unreachable_diagnostics` publishes a `Hint` diagnostic (code `unreachable`, `DiagnosticTag.Unnecessary`, so editors fade it and draw no squiggle) for each range `Analysis::unreachable` returns. The facts come from the checker, see [unreachable code](sema.md#unreachable-code-reachabilityrs-intervalrs). Not in files the analysis does not record.
- **Expansions**: `jai.showExpansion`, `jai/expansion` and `jai-expansion:` URIs show what an `#insert`, `#run` or macro produced; `jai/source` returns text for any URI the client has not opened.
- **Signature help**: `open_call` scans back to the unmatched `(`; recorded call facts are used when present, else the callee is looked up in scope.
- **Semantic tokens**: the legend is append-only; a token's wire type is `SemanticTokenKind as u32`, so `SemanticTokenKind::ALL` lists the kinds in declaration order and `name` gives each legend name. Here-strings tagged with a language (`#string WGSL`) get no token so the editor can highlight them; `LANGUAGE_TAGS` in `here_string.rs` must match `EMBEDDED_LANGUAGES` in `editors/vscode/scripts/build-grammar.ts`.

### Project settings

`[project]` in the `jacket.toml` at the project root (a legacy `jai.toml` with `build_files` and `import_path` is read the same way; see [jacket](../tools/jacket.md)):

```toml
[project]
entry = ["first.jai", "src/main.jai"]   # entries (what jaic is given)
import_path = ["vendor/modules"]        # extra module folders, like -import_dir
```

Configs are read through the open buffers over the disk and cached per directory by the hash of their text (`config.rs`). A config that does not parse is treated as empty and reported once with `window/showMessage`; a `jacket.toml` next to legacy files wins, with a one-time `window/logMessage`. A config with no `[project]` entries (a lint-only one) does not make its folder a project. Without it, entries are inferred from the workspace folder. Module search order: `modules/` next to the root file and each folder above it up to the project base, then `import_path`, then string literals in entry files that mention `import_path` (`modules.rs`; nothing is run), then the stdlib. A module file not reached by any `#load` is checked in the context of the entry that imports it (`Sources::closure_with_modules`).

### Protocol

`JsonSession::handle_json` parses a message (`incoming`) and hands it to `handle_request` or `handle_notification`. Method names are mapped once, in `Request::METHODS` and `Notification::METHODS`; each method is a match arm or a small `JsonSession` method returning `Result<_, Failure>`, where `Failure` pairs an `ErrorCode` (the JSON-RPC/LSP codes) with a message. `$/cancelRequest` and `exit` skip the lifecycle check; other methods before `initialize` get `ServerNotInitialized`, after `shutdown` `InvalidRequest`. Each handler runs under `catch_unwind` (`guarded`); a panic becomes an `InternalError`, caches are reset and open documents survive (not on `panic = "abort"` builds such as wasm). Non-standard requests: `jai/source`, `jai/expansion`. Documents use absolute `file:///` URIs (the browser uses `file:///jai-script/<name>`). The wasm bridge (`crates/jai-wasm/src/language_server.rs`) keeps the session in a `thread_local` because compiler state uses `Rc`; the worker message is `{type: "lsp", id, message}`.

## How to change it

- **Record more facts**: add a sema hook calling an `ide_*` method guarded by `self.ide.is_some()`; keep it cheap. Name facts go in `sema/ide.rs`, call and metaprogramming facts in `ide_meta.rs`. Extend `ide_hover`/`ide_entity_hover` to show them.
- **New hover content**: add a `hover::Block` kind with cases in `HoverText::plain` and `HoverText::markdown`.
- **New block scope**: call `ide_scope_span`, or completion inside it sees only the enclosing scope.
- **New expansion kind**: add an `IdeExpansionKind`, record it where the compiler evaluates it, handle it in `kind_name`, `describe`, `expansion` and `code_actions` (`features.rs`).
- **Print-family names** are the list `format::PRINT_FAMILY` (diagnostics publish without compiling); keep format semantics in step with `stdlib/Basic/Print.jai` and `jailint::format_string`.
- **Token legend**: append a `SemanticTokenKind` variant, to `ALL` and to `name` (`model.rs`); modifiers are `TOKEN_MODIFIERS` in `lib.rs`. Never reorder. `protocol.rs` tests check the order.
- **New request or notification**: add a variant and its method name to `Request::METHODS` or `Notification::METHODS`, then an arm in `JsonSession::request`/`notification`. Commands are `ServerCommand::NAMES`.
- **Environment**: native `main.rs` `native_environment`, browser `language_server.rs` `environment`; keep both in step with how `jaic` and the playground compile.
- **Lint rules** live in `crates/jailint`; the server picks them up unchanged.
- **Typed results** live in `model.rs`; extend them and the wire mapping in `protocol.rs` together.

Tests are in `crates/jai-language-server/tests/` (`semantic.rs`, `features.rs`, `auto_import.rs`, `links.rs`, `protocol.rs`, `resilience.rs`, plus refactor and hierarchy tests) and in `crates/jai-wasm/src/language_server.rs`. `node tools/browser/check_scripting_wasm.mjs <jai_wasm.wasm>` runs the same against the real wasm module.

## Configuration

```sh
cargo run -p jai-language-server --bin jailsp
cargo test -p jai-language-server
cargo build --release -p jai-wasm --target wasm32-unknown-unknown
node tools/browser/check_scripting_wasm.mjs target/wasm32-unknown-unknown/release/jai_wasm.wasm
```

- `JAIC_STDLIB` overrides the native stdlib directory.
- `jacket.toml` (else the legacy `jailint.toml`, `jai.toml`): nearest above the document.
- `initializationOptions` / `workspace/didChangeConfiguration` (`jai` section): `completion.autoImport` (default `true`), `completion.callSnippets` (`addParentheses` default, `fillArguments`, `none`), `inlayHints.closingBraceMinLines` (default 25, 0 is off), `trusted` (default `true`; `false` runs no compile-time code, see [workspace trust](../tools/workspace-trust.md)). `jailsp --untrusted` starts untrusted; the `jai/setTrusted` notification switches at runtime.
- Constants: `semantic.rs` (`BLOCK_BUDGET`, `CACHED`, `ALTERNATES`, `PARSES_KEPT`), `auto_import.rs` (`MAX_ITEMS`, `MIN_PREFIX`, `MAX_PROJECT_FILES`), `ide_meta.rs` (`MAX_EXPANSIONS`, `MAX_CALLS`, `MAX_VARIANTS`, `MAX_TEXT`), `features.rs` (`HINT_CHARS`).
- `Limits` are safety caps (512 documents, 32 MiB per document, 64 MiB per message); `Limits::browser()` is smaller. A document over a cap is not opened and the client gets a `window/showMessage`.
- Benchmark: `python3 tools/bench/main.py lsp` ([benchmarks](../tools/benchmarks.md)).

## Dependencies

`jaic` (lexer, parser, sema with `IdeFacts`, `SandboxHost`), `jailint`, `serde`/`serde_json`. In the browser it is reached through `engine.lsp(message)` ([playground](../browser/playground.md)). Protocol: LSP 3.17 over JSON-RPC 2.0.

- When `jailsp` has no input waiting it reads the auto-import module indexes (stdlib and project modules; `Session::warm_auto_import`), which cost a few hundred ms once, so the first completion of a name is not slow.
