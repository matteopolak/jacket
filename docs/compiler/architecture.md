# Compiler architecture

## What it is

An independent Jai compiler in Rust, aiming at source compatibility with real Jai programs. It is not complete; [features](../features.md) tracks what works.

| Crate | Role |
| --- | --- |
| `crates/jaic-syntax` | The front end: source files and spans, diagnostics, lexer, parser, syntax tree, the file system trait, and the small utilities they share. Knows nothing about type checking. |
| `crates/jaic` | Compiler core: sema, IR, interpreter, workspaces, IDE queries. Re-exports `jaic-syntax` at its root, so downstream crates keep writing `jaic::parser`, `jaic::source` and so on. |
| `crates/jaic-cli` | The `jaic` binary (`jaic file.jai` builds, `-check` only compiles, `-interp` interprets). |
| `crates/jaic-llvm` | Native code generation over the shared IR (Inkwell, LLVM 23). |
| `crates/jaic-native` | Native output without LLVM: build options, the system linker, and the loader for the code generation library ([lazy backend](lazy-backend.md)). |
| `crates/jaic-backend` | `jaic-llvm` as a shared library (`libjaic_backend`) that release builds of `jaic` open on the first native build. |
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
- `ide/` (`jaic::ide`): editor facts and the queries on them ([language server](language-server.md)). While `Compiler::ed` (an `Editor`, `ide/editor.rs`) is recording, checking records what each name refers to and its type (`ide/recording.rs`, `ide/meta/`) into `IdeFacts` (`ide/facts.rs`); hover, completion, definition and the other `ide_*` queries read those facts and the finished compile. Lookups by source span (`ide/span_index.rs`) are described in [IDE facts](ide-facts.md).
- `ir.rs`: the shared [IR](ir.md).
- `build.rs`, `records.rs`: workspaces and compiler messages for metaprograms ([workspaces](../metaprogramming/workspaces.md)).
- `interp/`: the IR [interpreter](interpreter.md), used for `#run`, metaprograms and `jaic -interp`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`. The compiler's own reads and writes of program memory go through `interp::memory`, which uses the fault-safe copies in `interp::probe`; interpreted loads and stores use raw pointers, like the native program.
- `stdlib/` (including `stdlib/prelude/`) is plain Jai, loaded through `sema::FileSystem` (real or virtual).

### Crate layers

`jaic-syntax` holds `source`, `render`, `diagnostics`, `lexer`, `parser`, `ast`, `intern`, `lexicon`, `docs`, `doc_comments`, `paths`, `filesystem`, `fxhash`, `knobs`, `suggest` and `macros`, plus the prelude and display-path helpers in its `lib.rs`. `jaic` depends on it and does `pub use jaic_syntax::*`, so inside `jaic` a path like `crate::ast::Expr` still works.

Only the syntax layer was split off. Sema, the interpreter and the build code call each other in both directions (`sema` <-> `interp` <-> `build`), and `sema` holds the IDE recorder, so cutting further needs trait inversion across about fifty call sites. Splitting `ir`, `target` and `ide` out would not shorten the rebuilds below either: everything above `jaic-syntax` still rebuilds when the parser changes.

Dev-profile rebuild times in seconds (no incremental; touching one file, then building the binary), measured on the same machine before and after the split:

| Touch | `jaic-cli` | `jailint` | `jai-language-server` |
| --- | --- | --- | --- |
| parser file, before / after | 10 / 11 | 10 / 11 | 10 / 12 |
| sema file, before / after | 11 / 10 | 10 / 9 | 13 / 13 |
| ide file, before / after | 10 / 7 | 12 / 11 | 12 / 10 |

The split does not speed up those three binaries, because each links the whole compiler. It pays off for tools that only need the syntax layer (parser tests, fuzz targets): after touching a parser file, `cargo build -p jaic-syntax` takes about 1.5 seconds, where the same touch used to rebuild all of `jaic`.

### Threading

The compiler proper is single-threaded. The exceptions: `parse_ahead.rs` runs the lexer and parser for the files a compile will load on a few worker threads, ahead of sema ([compile speed](compile-speed.md#parallel-lex-and-parse)); codegen runs one thread per unit; the interpreter has its own threads for `#run` code. Type checking is not parallel yet ([below](#parallel-front-end)).

The symbol interner (`intern.rs`) is shared by every thread: a worker interns as it parses, and a `Sym` means the same spelling on any thread, including threads a `#run` starts. Symbol numbers therefore follow thread timing; nothing visible may depend on them (sort by spelling, `tools/differential/determinism.py --scramble` checks it). `AstId`s (`ast.rs`) are process-wide too: each thread takes blocks of 256 from one atomic counter, so their numbers also follow thread timing and nothing visible may depend on their order. File ids come from sema's load order, so a worker parses with a placeholder file and `ast::relocate` rewrites the spans when sema claims the tree. Any new field in the syntax tree that holds a `Span` or `Ident` must also be visited in `ast/relocate.rs`; `parse_ahead`'s test fails otherwise.

### Parallel front end

Work is split between one driver thread, which owns `Compiler` and everything order-sensitive (the interpreter and `#run` order, metaprogram messages, diagnostics and their order, polymorph instance caches, record and IR numbering), and workers that run pure functions of a frozen snapshot. Workers never write shared tables. Each returns its result keyed by a stable id, and the driver merges results in the order the serial path would have produced them, so ids are assigned at merge time and `-threads N` gives the same output as `-threads 1`. When a worker needs something only the driver may do (a new polymorph instance, a `#run`, a name not resolved yet, a table still being written), it stops that body and hands it back; the driver then runs it through the ordinary serial path. Any doubt means falling back, never guessing. The full protocol, with its cost model, is in [parallel type checking](parallel-sema.md).

What runs on workers today: parsing ahead of sema (`parse_ahead.rs`, dedicated threads) and, on the shared pool (`pool.rs`, sized by `-threads`), body scans for stack-trace instrumentation, code generation units and `jailint`'s file checks. Checking a body and lowering it to IR are one pass (`FnCtx` is filled while expressions are checked), so there is no finished-bodies-to-IR step to hand to workers on its own; body checking is the unit that has to move, and it needs the world/per-body split described in the design before it can. The editor hooks checking calls are behind the `Recorder` trait (`ide/recorder.rs`, implemented by `Editor`), so a worker can later be given a recorder that buffers events for the driver to replay in order.

Wall and CPU seconds of `-check` at each `-threads` value (Apple M5, 10 cores, median of 7, `tools/bench/main.py compile`; CPU is user plus system). Checking itself is one thread, so what differs is the pool's size and run-to-run noise:

| Workload | Threads | Wall s | CPU s |
| --- | ---: | ---: | ---: |
| gen-60k | 1 | 0.148 | 0.139 |
| gen-60k | 2 | 0.148 | 0.153 |
| gen-60k | 4 | 0.133 | 0.193 |
| gen-60k | 8 | 0.141 | 0.250 |
| focus | 1 | 0.709 | 0.697 |
| focus | 2 | 0.697 | 0.744 |
| focus | 4 | 0.695 | 0.865 |
| focus | 8 | 0.691 | 0.938 |
| forbear | 1 | 0.176 | 0.170 |
| forbear | 2 | 0.167 | 0.200 |
| forbear | 4 | 0.156 | 0.227 |
| forbear | 8 | 0.159 | 0.263 |

CPU rises with the pool size and wall barely moves, because the front end (0.12 s of gen-60k, 0.62 s of focus) is one thread.

## How to change it

Add syntax to `lexer.rs`, `parser/` and `ast.rs` first, with tests in `parser/tests.rs`. Then teach `sema/` to check and lower it, and add a program under `tests/compiler/regressions/` or `tests/corpus/` that exits 0, or a negative case. Run the sweep (`tools/differential/jaic_sweep.py`); everything must pass. Unsupported constructs must produce a diagnostic, never a silent success.

A closed set of names the compiler matches on (directives, procedure flags, the sandbox's libc procedures, runtime check reasons, builtin keys) is an enum declared with `raw_enum!` (`macros.rs`), which derives `ALL` and the conversion from and to the name or number. Match on the enum, not on strings: the compiler then lists the cases a new variant needs.

## Configuration

```sh
cargo run -p jaic-cli -- file.jai -check [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- file.jai -interp [- metaprogram args]
```

`jaic-cli` features: `dynamic-llvm` (default) links LLVM's shared library into `jaic`, `static-llvm` links it statically, and `lazy-llvm` links none: `jaic` opens `libjaic_backend` beside it on the first build that writes native code, which is how [release archives](../tools/releases.md) are built ([lazy backend](lazy-backend.md)). All three need LLVM 23 to build ([LLVM setup](../tools/llvm-setup.md)). `mimalloc` (turned on by each LLVM feature) puts mimalloc under the global allocator ([compile speed](compile-speed.md#allocator)). `--no-default-features` builds a `jaic` that only checks and interprets, useful where LLVM is unavailable (e.g. an x86-64 `jaic` under Rosetta to test the interpreter's x86-64 foreign calls).

## Dependencies

`jaic` uses the Rust standard library and `rayon` (the one thread pool, `jaic::pool`: body scans, code generation units and `jailint`'s file checks share it, sized by `-threads`; the parse-ahead workers and the interpreter's Jai threads stay dedicated OS threads) (`Bindings_Generator` calls the host's libclang as an ordinary `#foreign` library through the interpreter). `jaic-llvm` uses Inkwell and LLVM 23; `jaic-native` uses neither. The language server uses `serde` and `serde_json`.
