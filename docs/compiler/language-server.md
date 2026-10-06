# Shared Jai language server

## What it is

`jai-language-server` is one JSON-RPC language server used by both native editors and the browser playground. The native `jai-lsp` binary adds `Content-Length` stdio framing. The browser links the same Rust `JsonSession` into `jai_wasm.wasm`.

It has two layers:

- **Syntax** (always on): lexer/parser diagnostics, semantic tokens, document symbols, and go-to-definition among the open documents. These come from the `jaic` lexer and parser.
- **Semantic** (when the session has an `Environment`): hover, completion and go-to-definition answered by actually type-checking the open documents with `jaic`. This covers locals, procedures, struct and enum types, imported modules' exports, Preload, members after `.`, and hover text with the real type (`count: s64`, `helper :: (t: *Thing) -> int`, `Thing :: struct { alpha: s64; ... }`).

## How it works

### Documents and syntax

`Session` admits versioned documents into a closed set of open files. Each version is lexed and parsed (`analysis.rs`) into declaration rows. The parser stops at its first error, so a version that does not parse has no rows. Completion then falls back to the rows of the last version that parsed (`Session::parsed`).

Positions use UTF-16, including supplementary characters and CRLF. Edits apply to a temporary copy and publish atomically. A stale version or invalid edit keeps the previous text.

### Semantic hover and completion

`semantic.rs` compiles on demand, only when a hover or completion request arrives, and caches by source text:

1. **Overlay.** Open documents are laid over the environment's file system (`OverlayFs`). Natively that is the disk plus the repository stdlib; in the browser it is the bundled stdlib `VirtualFs`.
2. **Root.** The check starts from the document that `#load`s the requested one and is loaded by none (`Session::root`).
3. **Recording.** `Compiler::ide` is set to `IdeFacts` for files under the root's directory. While checking, sema records what each identifier and member names, with its type, and the source extent of block and procedure scopes. See [Editor facts](#editor-facts-in-jaic).
4. **All bodies.** `ide_check_all` then lowers every non-polymorphic procedure body in those files, not only what `main` reaches, so helpers nobody calls yet still have facts.
5. **Isolation.** Compile-time code runs in a `SandboxHost`, so `#run` output never reaches the protocol's stdout. It also has an interpreter block budget (`Interp::block_budget`), so an edit that makes `#run` loop forever traps instead of hanging.

Half-typed text usually does not parse. `repair` blanks lines with spaces, so byte offsets stay put, until the text parses: first the cursor's line (for completion), then the line the parser reports. If the error is reported on an empty line or a lone `}`, it blanks the last non-empty line before it instead, because that is where a missing `;` belongs.

For completion, the word being typed and any `a.b.` chain before it are cut out of the text first. The probe text therefore stays the same while a word is typed, and the cached compile is reused for every keystroke.

- **Completion** asks `ide_scope_at` for the innermost recorded scope at the cursor.
  - **Plain names:** `ide_visible` walks the scope chain. It includes locals of the current procedure declared before the cursor, enclosing declarations, `using` members, imported modules' exports including re-exports, and Preload.
  - **After `a.b.`:** `ide_receiver` resolves the chain and `ide_members` lists the struct fields (through `using`), enum members, struct constants, or array/string/`Any` fields.
  - Results are filtered by the typed prefix (case-insensitive), and keywords are appended.
  - **After `#`:** the directives in `analysis.rs` `DIRECTIVES` (labels include the `#`, details are one-line descriptions), without compiling.
  - **Inside `#load "..."`:** files (`.jai`) and folders (`name/`) relative to the document, in the folder typed so far. **Inside `#import "..."`:** modules (folders and `.jai` files) on the import path and in the document's `modules/` folder. Entries come from the environment's `FileSystem::list_dir` plus the open documents (`path_completion`).
  - Trigger characters: `.`, `#`, `"` and `/`.
- **Hover** finds the smallest recorded reference at the offset and formats it:
  - a local or member as `name: Type`;
  - a procedure (each overload, under the name used) as `name :: <header>`;
  - a type with its fields or members;
  - a constant with its value.

- **Definition** (`ide_definition`) uses the same reference: an entity's declaration, every procedure of an overload set (aliases under their own name), or a struct. Spans are narrowed to the declared name; a procedure's span starts at its literal, so the name is found earlier on its line (`name :: (`). Targets can be modules or stdlib files the client never opened. Members and modules have no target yet.

When no environment is set, or the text cannot be repaired, hover, completion and definition fall back to the syntax layer.

### Editor facts in jaic

`crates/jaic/src/sema/ide.rs` holds `IdeFacts` and the queries. The hooks are small:

- `check_expr` wraps `check_expr_kind` and records `Ident`, `Member` and `InferredMember` results (`ide_note_expr`). `check_ident` leaves the resolved entity in `IdeFacts::last_entity`.
- `add_entity` records each declaration's name span (`ide_note_entity`).
- `ide_scope_span` is called where block scopes are made: procedure bodies, `check_scoped`, `if`/`case` arms, `while` bindings, `for` loops and block expressions.

All hooks do nothing when `Compiler::ide` is `None`, which is the case outside the language server.

Completion lists classify unresolved declarations by syntax (`ide_entity_name`): a procedure literal is a function, `struct`/`enum` is a type. Listing a module's exports therefore never compiles the whole module. Hover resolves the one entity it shows.

### Protocol and browser

`JsonSession::handle_json` handles:

- lifecycle: initialize/initialized, shutdown/exit;
- document open, change and close;
- `semanticTokens/full`, document symbols, hover, completion and definition;
- `jai/source` (non-standard, `{uri}` → text or `null`): the text of a definition target the client has not opened, from the open documents or the environment's file system. A browser editor can use it to show stdlib files read-only;
- cancellation.

Completion kinds map to LSP numbers in `protocol.rs`: function 3, field 5, variable 6, module 9, keyword 14, file 17, folder 19, enum member 20, constant 21, struct 22. Clients (including the hosted playground's editor) map those numbers to icons.

The worker carries `{type: "lsp", id, message}`. The wasm bridge (`crates/jai-wasm/src/language_server.rs`) keeps the session in a `thread_local`, because the compiler state uses `Rc`.

## How to change it

- **Record more facts.** Add a hook in sema that calls an `ide_*` method guarded by `self.ide.is_some()`, and keep the hook cheap. To show more in hover, extend `ide_hover`/`ide_entity_hover`. For more completion sources, extend `ide_visible`/`ide_members`.
- **Scope extents.** A new kind of block scope needs an `ide_scope_span` call, otherwise completion inside it sees the enclosing scope only.
- **Repair heuristics** live in `session.rs` (`repair`, `blank_line`). They must keep byte offsets unchanged, because hover positions are mapped back into the real text.
- **Environment.**
  - Native: `main.rs` `native_environment` (import paths, Preload).
  - Browser: `language_server.rs` `environment` (bundled stdlib, wasm target).
  - Keep these in step with how `jaic` and the playground compile.
- **Syntax features** stay in `analysis.rs`. Typed results live in `model.rs`; extend its enums and the wire mappings in `protocol.rs` together.

Tests:

- `crates/jai-language-server/tests/semantic.rs`: hover, completion while typing, member completion, and hover with a broken line elsewhere.
- `crates/jai-wasm/src/language_server.rs`: the same through the wasm bridge with the bundled stdlib.

## Configuration

```sh
cargo run -p jai-language-server --bin jai-lsp
cargo test -p jai-language-server
```

- `JAIC_STDLIB` overrides the stdlib directory the native server reads. The default is the repository's `stdlib/`.
- A `modules/` folder next to the root document is searched first.
- `semantic.rs` constants:
  - `BLOCK_BUDGET`: interpreter blocks per analysis.
  - `CACHED`: compiles kept, 3.
- Default `Limits`:
  - 32 documents, 256 KiB per document, 4 MiB total.
  - 1,024 completion items, 8,192 tokens.
- Documents must use absolute `file:///...` URIs. The browser uses `file:///jai-script/<name>`.

## Dependencies

- `jaic`: lexer, parser, sema with `IdeFacts`, interpreter `SandboxHost`.
- `serde` / `serde_json` for JSON.
- The browser adapter links into `jai_wasm.wasm` and is reached through `engine.lsp(message)` ([browser compiler](../browser/playground.md)). The hosted playground's editor lives in the portfolio repository.
- Protocol: [LSP 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) over [JSON-RPC 2.0](https://www.jsonrpc.org/specification).
