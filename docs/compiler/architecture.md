# Compiler architecture

## What it is

An independent Jai compiler in Rust, aiming at source compatibility with real Jai programs. It is not complete; [features](../features.md) tracks what works.

| Crate | Role |
| --- | --- |
| `crates/jaic` | Compiler core: lexer, parser, sema, IR, interpreter. No external crates. |
| `crates/jaic-cli` | The `jaic` binary (`jaic file.jai` builds, `-check` only compiles, `-interp` interprets). |
| `crates/jaic-llvm` | Native backend over the shared IR (Inkwell, LLVM 23). |
| `crates/jai-language-server` | JSON-RPC language server on top of `jaic` ([language server](language-server.md)). |
| `crates/jailint` | The `jailint` linter, also used by the language server ([jailint](../tools/jailint.md)). |
| `crates/jai-wasm` | `jaic` and the language server compiled to WebAssembly with the bundled `stdlib/` ([browser](../browser/playground.md)). |

## How it works

```
lexer -> parser (ast) -> sema (demand-driven checking, then lowering) -> ir -> interp | jaic-llvm
```

- `source.rs`, `intern.rs`: files, spans, diagnostics, the global symbol interner.
- `lexer.rs`: keywords are ordinary identifiers the parser interprets by position; directives (`#run`) and notes (`@x`) are their own token kinds.
- `parser/`: recursive descent to `ast.rs`, stopping at the first error ([parser](parser.md)).
- `sema/` ([sema](sema.md)) resolves names on demand, so a declaration is checked when something needs it (top-level structs are also laid out at the end, used or not). `scope.rs`/`modules.rs`: imports and scopes; `decls.rs`, `procs.rs`, `structs.rs`: declarations; `expr.rs`, `stmt.rs`, `calls.rs`, `convert.rs`: checking; `consteval.rs`: constants and `#run`; `typeinfo.rs`, `runtime_info.rs`: reflection; `bake.rs`, `modify.rs`: polymorphism; `code_export.rs`: syntax trees for metaprograms; `lower.rs`: to IR; `driver.rs`: entry points and `Options`.
- `ir.rs`: the shared [IR](ir.md).
- `build.rs`, `records.rs`: workspaces and compiler messages for metaprograms ([workspaces](../metaprogramming/workspaces.md)).
- `interp/`: the IR [interpreter](interpreter.md), used for `#run`, metaprograms and `jaic -interp`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`. Raw memory access is confined to `interp::memory`.
- `stdlib/` (including `stdlib/prelude/`) is plain Jai, loaded through `sema::FileSystem` (real or virtual).

## How to change it

Add syntax to `lexer.rs`, `parser/` and `ast.rs` first, with tests in `parser/tests.rs`. Then teach `sema/` to check and lower it, and add a program under `tests/stdlib/` or `tests/corpus/` that exits 0, or a negative case. Run the sweep (`tools/jaic-sweep.py`); everything must pass. Unsupported constructs must produce a diagnostic, never a silent success.

## Configuration

```sh
cargo run -p jaic-cli -- file.jai -check [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- file.jai -interp [- metaprogram args]
```

`jaic-cli` features: `dynamic-llvm` (default) links LLVM's shared library, `static-llvm` links it statically for [release archives](../tools/releases.md); both need LLVM 23 ([LLVM setup](../tools/llvm-setup.md)). `--no-default-features` builds a `jaic` that only checks and interprets, useful where LLVM is unavailable (e.g. an x86-64 `jaic` under Rosetta to test the interpreter's x86-64 foreign calls).

## Dependencies

`jaic` uses only the Rust standard library (it loads the host's libclang at run time for `Bindings_Generator`). `jaic-llvm` uses Inkwell and LLVM 23. The language server uses `serde` and `serde_json`.
