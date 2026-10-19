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

- `source.rs`, `intern.rs`: files, spans, diagnostics, the symbol interner (process-wide; see [threading](#threading)).
- `lexer.rs`: keywords are ordinary identifiers the parser interprets by position; directives (`#run`) and notes (`@x`) are their own token kinds.
- `parser/`: recursive descent to `ast.rs`, stopping at the first error ([parser](parser.md)).
- `sema/` ([sema](sema.md)) resolves names on demand, so a declaration is checked when something needs it (top-level structs are also laid out at the end, used or not). `scope.rs`/`modules.rs`: imports and scopes; `decls.rs`, `procs.rs`, `structs.rs`: declarations; `expr.rs`, `stmt.rs`, `calls/`, `convert.rs`: checking; `consteval.rs`: constants and `#run`; `typeinfo.rs`, `runtime_info.rs`: reflection; `bake.rs`, `modify.rs`: polymorphism; `code_export.rs`: syntax trees for metaprograms; `lower.rs`: to IR; `driver.rs`: entry points and `Options`.
- `ir.rs`: the shared [IR](ir.md).
- `build.rs`, `records.rs`: workspaces and compiler messages for metaprograms ([workspaces](../metaprogramming/workspaces.md)).
- `interp/`: the IR [interpreter](interpreter.md), used for `#run`, metaprograms and `jaic -interp`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`. The compiler's own reads and writes of program memory go through `interp::memory`, which uses the fault-safe copies in `interp::probe`; interpreted loads and stores use raw pointers, like the native program.
- `stdlib/` (including `stdlib/prelude/`) is plain Jai, loaded through `sema::FileSystem` (real or virtual).

### Threading

The compiler proper is single-threaded. The exceptions: `parse_ahead.rs` runs the lexer and parser for the files a compile will load on a few worker threads, ahead of sema ([compile speed](compile-speed.md#parallel-lex-and-parse)); codegen runs one thread per unit; the interpreter has its own threads for `#run` code. Type checking is not parallel.

The symbol interner (`intern.rs`) is shared by every thread: a worker interns as it parses, and a `Sym` means the same spelling on any thread, including threads a `#run` starts. Symbol numbers therefore follow thread timing; nothing visible may depend on them (sort by spelling, `tools/differential/determinism.py --scramble` checks it). The `AstId`s in `ast.rs` come from a thread-local counter and file ids from sema's load order, so a worker parses with its own ids and a placeholder file, and `ast::relocate` rewrites the tree when sema claims it. Any new field in the syntax tree that holds a `Span`, `Ident` or `AstId` must also be visited in `ast/relocate.rs`; `parse_ahead`'s test fails otherwise.

## How to change it

Add syntax to `lexer.rs`, `parser/` and `ast.rs` first, with tests in `parser/tests.rs`. Then teach `sema/` to check and lower it, and add a program under `tests/compiler/regressions/` or `tests/corpus/` that exits 0, or a negative case. Run the sweep (`tools/differential/jaic_sweep.py`); everything must pass. Unsupported constructs must produce a diagnostic, never a silent success.

A closed set of names the compiler matches on (directives, procedure flags, the sandbox's libc procedures, runtime check reasons, builtin keys) is an enum declared with `raw_enum!` (`macros.rs`), which derives `ALL` and the conversion from and to the name or number. Match on the enum, not on strings: the compiler then lists the cases a new variant needs.

## Configuration

```sh
cargo run -p jaic-cli -- file.jai -check [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- file.jai -interp [- metaprogram args]
```

`jaic-cli` features: `dynamic-llvm` (default) links LLVM's shared library, `static-llvm` links it statically for [release archives](../tools/releases.md); both need LLVM 23 ([LLVM setup](../tools/llvm-setup.md)). `mimalloc` (turned on by both LLVM features) puts mimalloc under the global allocator ([compile speed](compile-speed.md#allocator)). `--no-default-features` builds a `jaic` that only checks and interprets, useful where LLVM is unavailable (e.g. an x86-64 `jaic` under Rosetta to test the interpreter's x86-64 foreign calls).

## Dependencies

`jaic` uses the Rust standard library and `rayon` (the one thread pool, `jaic::pool`: body scans, code generation units and `jailint`'s file checks share it, sized by `-threads`; the parse-ahead workers and the interpreter's Jai threads stay dedicated OS threads) (`Bindings_Generator` calls the host's libclang as an ordinary `#foreign` library through the interpreter). `jaic-llvm` uses Inkwell and LLVM 23. The language server uses `serde` and `serde_json`.
