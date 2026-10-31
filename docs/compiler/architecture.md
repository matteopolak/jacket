# Compiler architecture

An independent Jai compiler in Rust. [Features](../features.md) tracks what works.

| Crate | Role |
| --- | --- |
| `jaic-syntax` | Front end: sources, spans, diagnostics, lexer, parser, AST. No type checking. |
| `jaic` | Core: sema, IR, interpreter, workspaces, IDE queries. Re-exports `jaic-syntax`. |
| `jaic-cli` | The `jaic` binary. |
| `jaic-llvm` | Native backend over the shared IR (Inkwell, LLVM 23). |
| `jai-language-server` | JSON-RPC server on `jaic` ([language server](language-server.md)). |
| `jailint` | The linter, also used by the language server. |
| `jai-wasm` | `jaic` and the language server as WebAssembly ([browser](../browser/playground.md)). |

## How it works

```
lexer -> parser (ast) -> sema (demand-driven checking + lowering) -> ir -> interp | jaic-llvm
```

- Keywords are ordinary identifiers the parser reads by position; directives (`#run`) and notes (`@x`) are their own tokens. The parser stops at the first error ([parser](parser.md)).
- Sema ([sema](sema.md)) checks a declaration when something needs it. Checking a body and lowering it to [IR](ir.md) are one pass.
- The [interpreter](interpreter.md) runs `#run`, metaprograms and `jaic -interp`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`.
- `ide/` records what each name refers to while checking, and the editor queries read those facts ([IDE facts](ide-facts.md)).
- `stdlib/` is plain Jai, loaded through `sema::FileSystem` (real or virtual).

### Threading

Sema is single-threaded. Parsing runs ahead on worker threads ([compile speed](compile-speed.md#parallel-lex-and-parse)), codegen runs one thread per unit, and `#run` code can start its own threads. The one `rayon` pool (`pool.rs`, sized by `-threads`) is shared by body scans, codegen units and `jailint`.

The symbol interner and `AstId` counter are process-wide, so their numbers follow thread timing. Nothing visible may depend on them (sort by spelling; `tools/differential/determinism.py --scramble` checks it). Workers parse with a placeholder file id and `ast::relocate` rewrites spans when sema claims the tree, so any new `Span` or `Ident` field in the AST must also be visited in `ast/relocate.rs`.

[Parallel type checking](parallel-sema.md) describes the planned driver/worker split: one driver owns everything order-sensitive and merges worker results in serial order.

## How to change it

- New syntax: `lexer.rs`, `parser/`, `ast.rs` (tests in `parser/tests.rs`), then sema, then a program under `tests/compiler/regressions/` or `tests/corpus/`.
- Unsupported constructs must produce a diagnostic, never a silent success.
- Closed sets of names (directives, procedure flags, builtin keys) are enums declared with `raw_enum!` (`macros.rs`). Match on the enum, not strings.
- Only the syntax layer is a separate crate; sema, interp and build call each other both ways.

## Configuration

```sh
cargo run -p jaic-cli -- file.jai -check [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- file.jai -interp
```

- `dynamic-llvm` (default) and `static-llvm` (release archives) need LLVM 23 ([LLVM setup](../tools/llvm-setup.md)).
- `mimalloc` is on with either LLVM feature ([allocator](compile-speed.md#allocator)).
- `--no-default-features` builds a check-and-interpret `jaic` without LLVM.

## Dependencies

`rayon`; Inkwell and LLVM for `jaic-llvm`; `serde` for the language server.
