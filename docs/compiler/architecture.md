# Compiler architecture

## What it is

An independent Rust implementation targeting Jai source compatibility. Recent upstream applications and libraries refine the older beta  local distribution. It is **not a complete Jai implementation**; the [README](../../README.md#compatibility) tracks what is supported.

The workspace has five crates:

| Crate | Role |
| --- | --- |
| `crates/jaic` | The compiler core: lexer, parser, semantic analysis, IR and interpreter. No external dependencies. |
| `crates/jaic-cli` | The `jaic` binary (`run`, `check`, `build`). |
| `crates/jaic-llvm` | Native backend over the shared IR (Inkwell, LLVM 22). |
| `crates/jai-language-server` | Bounded JSON-RPC language server built on the `jaic` lexer and parser. See [language server](language-server.md). |
| `crates/jai-wasm` | Browser build: `jaic` plus the language server compiled to WebAssembly with the bundled `stdlib/`. See [browser compiler](../browser/playground.md). |

## How it works

`jaic` is a pipeline: `lexer` -> `parser` (-> `ast`) -> `sema` (demand-driven typing into the checked tree, then lowering) -> `ir` -> `interp` (compile-time execution, `jaic run`, the browser) or `jaic-llvm`.

- `source.rs`, `intern.rs`: files, spans, diagnostics and the global symbol interner.
- `lexer.rs`: tokens. Keywords are ordinary identifiers; the parser gives them meaning by position. Directives (`#run`) and notes (`@x`) are separate token kinds.
- `parser/`: recursive descent to `ast.rs`. Types are ordinary expressions. Parsing stops at the first error (no recovery).
- `sema/`: resolves names on demand, so declarations are checked when something needs them (top-level structs are also laid out at the end, used or not; see [sema-polymorphism-and-declarations.md](sema-polymorphism-and-declarations.md)). Files: `scope.rs`/`modules.rs` (imports, `#load`, scopes), `decls.rs`/`procs.rs`/`structs.rs` (declarations), `expr.rs`/`stmt.rs`/`calls.rs` (checking), `consteval.rs` (constants), `typeinfo.rs`/`runtime_info.rs` (reflection), `bake.rs`/`modify.rs` (polymorphism), `lower.rs` (to IR), `driver.rs` (entry points and `Options`).
- `build.rs`: build workspaces and compiler messages for metaprograms (`#run` programs that call the `Compiler` module).
- `interp/`: IR interpreter used for `#run`, metaprograms and `jaic run`. Its host abstraction (`Host`, `SandboxHost`) decides which `#foreign` calls exist; the browser uses the sandboxed one. Raw memory access is confined to `interp::memory`.
- `stdlib/` (independently written standard library) and `prelude/` (runtime type definitions) are plain Jai loaded through the virtual or real filesystem (`sema::FileSystem`).

## How to change it

Add syntax in `lexer.rs`/`parser/` and `ast.rs` first, with parser tests in `parser/tests.rs`. Then teach `sema/` to check and lower it, and add a program under `tests/stdlib/` or `tests/corpus/` that exits 0 (or a negative case). Run `python3 tools/jaic-sweep.py corpus negative stdlib modules upstream --timeout 900`; everything should pass. Unsupported constructs should produce a diagnostic, never a silent success.

## Configuration

```sh
cargo run -p jaic-cli -- check file.jai [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- run file.jai [- metaprogram args]
```

Native builds need an independently installed LLVM 22 (see [LLVM setup](../tools/llvm-setup.md)). The
`jaic-cli` feature `llvm` pulls in `jaic-llvm`. `dynamic-llvm` (the default) links LLVM's shared library and
`static-llvm` its static libraries (release archives, see [releases](../tools/releases.md)); `--no-default-features` builds a `jaic`
that only checks and interprets (`build` reports that it cannot write native output), for hosts or
targets without LLVM libraries, such as an x86-64 `jaic` run under Rosetta to test the interpreter's
x86-64 foreign calls.

## Dependencies

`jaic` uses only Rust's standard library. `jaic-llvm` uses Inkwell/LLVM 22. The language server uses `serde`/`serde_json`. No supplied binary is part of the implementation.
