# Compiler architecture

## What it is

An independent Jai compiler in Rust, aiming at source compatibility with real Jai programs. It is not a complete Jai implementation; the [README](../../README.md#compatibility) tracks what works.

| Crate | Role |
| --- | --- |
| `crates/jaic` | Compiler core: lexer, parser, semantic analysis, IR, interpreter. No external crates. |
| `crates/jaic-cli` | The `jaic` binary (`run`, `check`, `build`). |
| `crates/jaic-llvm` | Native backend over the shared IR (Inkwell, LLVM 22). |
| `crates/jai-language-server` | JSON-RPC language server on top of `jaic`. See [language server](language-server.md). |
| `crates/jai-wasm` | Browser build: `jaic` and the language server compiled to WebAssembly with the bundled `stdlib/`. See [browser compiler](../browser/playground.md). |

## How it works

```
lexer -> parser (ast) -> sema (demand-driven checking, then lowering) -> ir -> interp | jaic-llvm
```

- `source.rs`, `intern.rs`: files, spans, diagnostics, and the global symbol interner.
- `lexer.rs`: tokens. Keywords are ordinary identifiers that the parser interprets by position; directives (`#run`) and notes (`@x`) are their own token kinds.
- `parser/`: recursive descent to `ast.rs`, stopping at the first error. Types are ordinary expressions. See [parser](parser.md).
- `sema/`: resolves names on demand, so a declaration is checked when something needs it. Top-level structs are also laid out at the end, used or not.
  - `scope.rs`, `modules.rs`: imports, `#load`, scopes ([sema: modules](sema-modules.md))
  - `decls.rs`, `procs.rs`, `structs.rs`: declarations ([sema: polymorphism and declarations](sema-polymorphism-and-declarations.md))
  - `expr.rs`, `stmt.rs`, `calls.rs`, `convert.rs`: checking
  - `consteval.rs`: constants and `#run`
  - `typeinfo.rs`, `runtime_info.rs`: reflection
  - `bake.rs`, `modify.rs`: polymorphism helpers
  - `code_export.rs`: syntax trees for metaprograms
  - `lower.rs`: to IR; `driver.rs`: entry points and `Options`
- `ir.rs`: the shared [IR](ir.md).
- `build.rs`, `records.rs`: workspaces and compiler messages for metaprograms ([workspaces](../metaprogramming/workspaces.md)).
- `interp/`: the IR [interpreter](interpreter.md), used for `#run`, metaprograms and `jaic run`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`. Raw memory access is confined to `interp::memory`.
- `stdlib/` are plain Jai, loaded through `sema::FileSystem` (real or virtual).

## How to change it

Add syntax to `lexer.rs`, `parser/` and `ast.rs` first, with tests in `parser/tests.rs`. Then teach `sema/` to check and lower it, and add a program under `tests/stdlib/` or `tests/corpus/` that exits 0, or a negative case. Run the [sweep](../tools/jaic-sweep.md); everything must pass.

Unsupported constructs must produce a diagnostic, never a silent success.

## Configuration

```sh
cargo run -p jaic-cli -- check file.jai [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- run file.jai [- metaprogram args]
```

Cargo features of `jaic-cli`:

- `dynamic-llvm` (default) links LLVM's shared library; `static-llvm` links it statically, for [release archives](../tools/releases.md). Both need LLVM 22 ([LLVM setup](../tools/llvm-setup.md)).
- `--no-default-features` builds a `jaic` that only checks and interprets; `build` reports that it can't write native output. Useful for hosts without LLVM libraries, such as an x86-64 `jaic` under Rosetta for testing the interpreter's x86-64 foreign calls.

## Dependencies

`jaic` uses only the Rust standard library (it loads the host's libclang at run time for `Bindings_Generator`). `jaic-llvm` uses Inkwell and LLVM 22. The language server uses `serde` and `serde_json`.
