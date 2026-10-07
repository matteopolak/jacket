# Fuzzing

## What it is

Coverage-guided fuzzing of the compiler with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). The targets in `fuzz/` feed generated input to the lexer, parser, the whole front end with the real stdlib, the interpreter, the language server and the `jaifmt` formatter, and treat every panic, abort, stack overflow, hang and runaway allocation as a bug: bad input must produce a diagnostic, never a crash.

Fuzzing looks for crashes, plus one round-trip property: `jaifmt` must not change what a program means. Wrong output is the job of [differential testing](differential-testing.md): `tools/jaigen.py` generates programs whose behaviour is fully defined, and `tools/jaic-diff.py` requires the interpreter, the native builds at `-O0` and `-O2`, and the browser engine to print the same thing for each one. The `generated` target below aims for programs that compile (and sometimes not); jaigen aims for programs that run to the end with a checked result.

## How it works

`fuzz/` is its own Cargo workspace (excluded from the root one, with its own `Cargo.lock`), with two packages:

- `fuzz/harness` (`jai-fuzz-harness`): one plain function per target taking `&[u8]`, plus the grammar-based program generator. It has no libFuzzer dependency, so `cargo test` can replay inputs through exactly the code the fuzzer runs.
- `fuzz` (`jai-fuzz`): the libFuzzer binaries in `fuzz/fuzz_targets/`, each a one-line call into the harness.

The harnesses call library entry points directly. Nothing on these paths uses `catch_unwind`, so a panic aborts the process and libFuzzer records it.

| Target | Input | What runs |
| --- | --- | --- |
| `lexer` | bytes (lossy UTF-8) | `jaic::lexer::lex` |
| `parser` | bytes (lossy UTF-8) | `jaic::parser::parse_file` |
| `check` | bytes as `main.jai` | the playground compile (`jai_wasm::play::run_with` with `compile_only`): bundled stdlib, every `#run`, type checking, lowering; `main` is not run |
| `interp` | bytes as `main.jai` | `check`, then `main` in the interpreter |
| `generated` | bytes chosen by `arbitrary` | `harness::generate::program` renders a Jai program, then as `interp` |
| `lsp` | bytes as a document | `jai_language_server::Session` with the playground environment: diagnostics, symbols, semantic tokens, hover and definition at ~60 offsets, completion at 4, then an edit and the same again |
| `lsp_json` | newline-separated JSON-RPC | `JsonSession::handle_json` after an `initialize` handshake (non-UTF-8 input goes to the frame decoder instead) |
| `lsp_edits` | a document, then optionally a NUL and script bytes | a `Session` with two open documents driven by up to 24 random steps (`harness/src/lsp_edits.rs`): incremental and whole-document edits, every request at random positions, close and reopen |
| `jaifmt` | bytes as a Jai file | `Jai_Format` under the playground interpreter, twice, checked against the compiler's lexer and parser (`harness/src/jaifmt.rs`) |

### Sandboxing

Compile-time execution and `main` run through the browser playground's sandbox, the same one `jaic run -os wasm` uses: target OS/CPU `wasm`, the stdlib from an in-memory file system, and `SandboxHost`, a small in-memory libc with no dynamic linker, real files, network or clock. `BLOCK_BUDGET` (2M interpreter basic blocks per input) stops infinite loops in `#run` and `main` with an "execution budget exhausted" diagnostic, also in the workspaces a metaprogram compiles, which draw from the same budget. Any hang that is left is in the compiler itself.

Compiling harnesses (`check`, `interp`, `generated`, `lsp`, `lsp_json`, `lsp_edits`, `jaifmt`) run on a 256 MiB thread, the stack the wasm build links with. The native CLI and `jailsp` use 1 GiB. `lexer` and `parser` run on libFuzzer's main thread (8 MiB).

The interpreter uses real memory, so a mutated program that builds a pointer from an integer and writes through it can crash the fuzzer process. That is the program's bug, not the compiler's. Before treating a `SEGV` in `interp`/`check` as a compiler bug, check whether the input forms a pointer itself (`cast(*T)`, `xx`, `---`, pointer arithmetic). The generator never emits these forms.

### Structure-aware generation

Random bytes rarely get past the parser. `harness/src/generate.rs` reads the fuzzer's bytes through `arbitrary::Unstructured` and uses them to choose productions of a small typed grammar: enums (`enum`, `enum u8`, `enum_flags`, explicit values), structs (`using` bases, defaults, `#place`), a polymorphic struct `Box(T)`, procedures with typed parameters and returns, polymorphic procedures (`$T`), `#expand` macros (with `Code` arguments and backticks), constants (`#run` initializers), globals, `#if`/`else` declarations and `#run` blocks. `main` calls every procedure, because sema is demand-driven and checks only what is reachable. The generator tracks the locals in scope and builds expressions of a requested type, so about 70% of programs compile cleanly. One program in eight is a "chaos" program that sometimes uses a value of the wrong type, which keeps the error paths covered.

One top-level declaration in eleven is an "edge" production (`edge_decl`): a shape that crashed the compiler before, filled with numbers from the input (`0`, `-1`, powers of two up to `1 << 62`, `i64::MAX`, small negatives). These shapes are polymorphic struct and `$N` procedure recursion, `using` pointer cycles with `#align`, self-inserting strings, a cyclic `#run` pointer ring, nested huge arrays, `#no_padding` structs, shift/divide constant arithmetic, out-of-range enum values, `cast(Type) n`, distinct type cycles and inserted strings. `main` names each one with `type_of`, so sema reaches it.

### Edit scripts (`lsp_edits`)

`lsp` queries one document and applies one edit; an editor sends long runs of small edits, each answered while the text is half-typed. `lsp_edits` reads the input as a document, a NUL byte and a script. Plain Jai seeds have no NUL, so their script comes from a hash of the text, and every seed still drives edits. The script is read through `arbitrary::Unstructured`: each step is an edit (1-3 ranged changes in one notification, now and then with a stale version, or a whole-document replacement), a request (hover, definition, type definition, references, highlights, prepare-rename and rename with valid and invalid names, signature help, completion, expansion, polymorphs, code actions, inlay hints, symbols, semantic tokens, folding ranges, code lenses, links, workspace symbols) or a close with an optional reopen. Positions are mostly on existing lines but include offsets past the line end, inside a surrogate pair (the snippets insert `𝄞`), past the last line and `u32::MAX`; ranges are sometimes reversed. A second document (`b.jai`, the first half of the input) gets a fifth of the steps. Every result is ignored: errors are fine, panics are not.

### Formatter round trip (`jaifmt`)

`jaifmt` is a Jai program, so the target runs it the way the browser does: the playground compiles a small driver (embedded in `harness/src/jaifmt.rs`) against the bundled stdlib and interprets it. The driver reads the input and a `jaifmt.toml` from the sandbox, formats the input, formats the result again and reports by exit code. A hash of the input picks one of four configs (default, 2-space with `case_indent = 0`, `brace_style = "preserve"` with no blank lines, 8-space). Compiling the driver and formatting an input take about 30 ms, so the target gets tens of executions per second per worker.

The harness panics (a finding) when:

- the formatter's own token check fails ("internal error"), or it refuses its own output;
- the second pass changes the text (not idempotent);
- `jaic::lexer` lexes the output to a different token sequence than the input;
- the input parses with `jaic::parser` and the output does not, or the two syntax trees differ. Trees are compared through their `Debug` text with span offsets and `AstId`s blanked, so line breaks the parser looks at (`newline_before`) are covered;
- the driver runs out of its interpreter budget (`FORMAT_BUDGET`): formatting is linear, so that is a hang.

Input the formatter refuses (it does not lex, or brackets do not balance) is not a finding. The seeds include `corpus/upstream` when it is checked out: third-party code nobody formatted with jaifmt.

### Corpora and dictionary

`fuzz/seed_corpus.py` builds `fuzz/corpus/<target>/` from the repository's own Jai files: `tests/stdlib`, `tests/corpus/{positive,negative}`, `examples` and `fuzz/seeds` for the compiling targets, plus `stdlib/` for the lexer and parser. Size caps keep execution fast, and `lsp_json` gets each file wrapped in a JSON-RPC session. Saved regressions for the target are added too. `fuzz/seeds/` holds small hand-written programs that probe the compiler's limits: overflowing constants, recursive and oversized types, metaprogram misuse, odd `#insert`/`#run` use. Corpora are not committed (`fuzz/.gitignore`); CI caches them between runs. `fuzz/jai.dict` lists keywords, directives, operators and literal shapes for libFuzzer's mutator.

## Running locally

Install cargo-fuzz once (`cargo install cargo-fuzz`). The repository's nightly toolchain is picked up automatically. Then:

```sh
fuzz/run.sh check 1200 4      # target, seconds, parallel workers
fuzz/run.sh lexer 600
```

`run.sh` seeds an empty corpus, builds without a sanitizer but with debug assertions (arithmetic overflow panics), and runs libFuzzer in fork mode. Crashes, timeouts and OOMs are collected in `fuzz/artifacts/<target>/` without stopping the run. Per-target limits: `-max_len` 16 KiB (lexer/parser), 8 KiB (compiling targets and `jaifmt`), 4 KiB (LSP targets); `-timeout` 5/10/20 s (`jaifmt` 20 s); `-rss_limit_mb=4096`. `-report_slow_units` equals the timeout: libFuzzer's default (10 s) is below the LSP targets' timeout, and a slow unit is a file in `artifacts/`, which fails the run. A test that compiles many workspaces (`tests/stdlib/arithmetic-overflow-check.jai`: 19, each importing Basic) costs about 2 s per `lsp` input on an M-series laptop, since the target compiles the document about ten times; on a CI runner that crossed 10 s.

ASan is off on purpose. The compiler is safe Rust apart from the interpreter's program memory, and the interpreted program's own raw memory use (which ASan would flag) is not a compiler bug.

Throughput differs by orders of magnitude. With three fork workers on an M-series laptop, `lexer` runs about 10,000 executions per second and `parser` several thousand. `check`, `interp` and `generated` run around 100, `lsp_edits` about 40 and `jaifmt` under 20 (each input is formatted twice by the interpreted stdlib formatter). Give the slow targets longer runs.

## Triage and minimizing

Replay a finding without libFuzzer, and see the panic message and timing:

```sh
cargo run --release --manifest-path fuzz/harness/Cargo.toml --example replay -- check fuzz/artifacts/check/crash-*
JAI_FUZZ_VERBOSE=1 cargo run ... -- check <file>      # also print diagnostics and output
cargo run ... --example replay -- show <generated-input>  # print the generated program
```

Minimize with libFuzzer (`cargo fuzz tmin --sanitizer none --debug-assertions <target> <file>`), or by hand: crash inputs are usually small Jai files. For `generated`, minimize the printed program and file it under `check`/`interp` instead of keeping the raw bytes.

## Adding regressions

Every fixed crash gets two things:

1. The minimized input in `fuzz/regressions/<target>/` (any file name). `fuzz/harness/tests/regressions.rs` replays every file there through the same harness function on every push and pull request.
2. A test next to the fix: a parser unit test, a `crates/jai-language-server/tests/fuzz_regressions.rs` case, or a `tests/stdlib/` / `tests/corpus/negative` program run by the sweep.

```sh
cargo test --manifest-path fuzz/harness/Cargo.toml   # replay fuzz/regressions/
```

## CI

`.github/workflows/fuzz.yml`:

- `regressions` (every push and pull request): checks the dependency age of `fuzz/Cargo.lock` and runs the replay test. It takes a few minutes and needs no cargo-fuzz.
- `fuzz` (nightly at 03:23 UTC and by `workflow_dispatch`, with a `seconds` input, default 600): one ubuntu job per target. Each restores the cached corpus (`actions/cache`, key `fuzz-corpus-<target>-<run>` with a prefix restore), adds the repository seeds, runs `fuzz/run.sh` with one worker per core, minimizes the corpus with `cargo fuzz cmin` and saves it. Any finding fails the job and uploads `fuzz/artifacts/<target>` as `fuzz-artifacts-<target>`. A push to the `ci/fuzz` branch runs the same jobs for 60 seconds each, to test the workflow itself.

To act on a nightly failure, download the artifact, replay it with the `replay` example, fix it, and add the input under `fuzz/regressions/`.

## Bug classes found

These explain the limits listed under Configuration; each has a regression test.

- **Lexer:** a here-string flag (`#string,\`) at the end of the file sliced past the end of the input. An unknown escape of a multibyte character (`"\é"`, or an invalid byte, which reads as the three-byte U+FFFD) ended its error span inside the character, and rendering the error panicked (`check`, `interp`). Lexer error spans now end on a character boundary.
- **Parser stack overflow:** deeply nested parentheses, blocks or array types recursed until the stack overflowed. Now `MAX_NESTING` gives a diagnostic.
- **Parser hang:** `#assert(` tried the `(cond, message)` form, rewound and parsed again, so each nested `#assert(` doubled the work. The form is now chosen up front, by looking for a top-level comma. A statement starting with `ifx` had the same shape: it was parsed as an assigning `if` (`ifx c then a = 1;`), then rewound and parsed as an expression, so each `{ifx` nested in the condition doubled the work. Each part is now parsed once and the form chosen afterwards.
- **LSP:** the native server ran on the 8 MiB main thread. Completion after a multibyte character split a UTF-8 boundary. Go to definition on a builtin constant (`OS`), which has no source location (`Span::NONE`), indexed past the file table (`lsp`, `lsp_json`, `lsp_edits`). The class is any span reaching source text: `SourceMap::try_get` returns `None` for a file id with no file and `SourceMap::snippet` returns `""` for a span without a place, and the IDE queries, the language server's `location` and jailint's rules go through them, so a builtin's span drops out of every location query rather than only go to definition.
- **Oversized types:** array and struct sizes overflowed `u64` in layout, and the interpreter then aborted on the allocation. Types over `MAX_SIZE` are now an error. Allocations the host cannot make return null (`malloc`) or trap (globals).
- **Compile-time images:** every struct's default value was built in memory at the struct's full size, so a legal struct of 2^48 bytes with all-zero defaults aborted the compiler. The LSP target found this. Default images are now built only when a field has a nonzero default. Constants, initializers and default images over `MAX_IMAGE` are a diagnostic.
- **Constant folding:** `i64::MIN / -1`, `% -1`, negation and negative shift amounts overflowed or panicked.
- **Unbounded compile-time recursion:** polymorphic recursion (each instance creating a new one), a string that `#insert`s itself, and `using` pointer cycles in member lookup. These are now capped by `MAX_INSTANCES`, `MAX_INSERT_DEPTH` and a visited set.
- **Formatter:** a directive at the very start of a file had its flags spaced (`#library,link_always` became `#library, link_always`, which the parser no longer reads as flags), because `directive_zone` marked token 0 only after reading it. The formatter's own token check uses the same zones, so only the compiler's parser could see it (`jaifmt` target, on `corpus/upstream`).
- **Formatter, more (`jaifmt`):** `1.2.3` (`1.2`, `.`, `3`) was spaced to `1.2 .3`, where `.3` lexes as a float; and a note with an open `(` (`@Note(x`) swallowed the trailing blank lines the formatter trims. Both were caught by the formatter's own token check, so the tool refused valid input with an "internal error" instead of corrupting it.
- **Signature help (`lsp_edits`):** a call whose callee starts at byte 0 underflowed computing the callee's start, and in text that does not parse the callee lookup resolved a builtin procedure (`type_info(`) as a declaration, which the resolver treated as unreachable.
- **Unary operators on `Type` (`interp`):** `~T` and `-T` with `T := int` type-checked and produced a garbage type id, and printing it crashed the interpreter (`~` on a `bool` likewise made the byte 254). Unary `-` and `~` now require an integer, enum or float operand unless an operator overload applies.
- **Workspace budgets (`check`):** a workspace's compiler had no interpreter budget in the playground (and a fresh one per workspace in jailsp and jailint), so an endless `#run` in a program a metaprogram compiled hung. The workspace now runs on what is left of its driver's budget.
- **Record ids (`lsp_edits`, `check`):** compile-time code run during a typed `compiler_get_nodes` export (a pending `#insert` expanded by a lookup) made records in an empty stand-in table numbered from 1, so `stdlib/Compiler`'s by-id struct cache returned other records' structs and the program read through garbage pointers. The stand-in now draws ids from the same counter (`Records::lend`).
- **Metaprogram rounds (`lsp_edits`):** a metaprogram that added code at every `TYPECHECKED_ALL_WE_CAN` (here because an edit broke the line that stopped it) ran until the language server's budget was spent, and each round revisited every scope, entity and file so far (`expand_all`, and the module lookup behind each new file's FILE message). That made the budget's ~16 000 rounds quadratic, past the 20 s timeout under coverage. Rounds now cost only what is new.
- **Compile-time values:** a `#run` result holding a pointer cycle was copied into the program recursively until the stack overflowed. An integer cast to `Code` indexed past the code table. `#align` accepted values that broke layout arithmetic.

## How to change it

- Read a span's text with `SourceMap::snippet` and its file with `SourceMap::try_get`. `SourceMap::get` and direct slicing are for file ids and ranges known to be real: declaration spans can be `Span::NONE`.
- New target: add a `pub fn <name>(data: &[u8])` to `harness/src/lib.rs`, a two-line file in `fuzz_targets/`, a `[[bin]]` in `fuzz/Cargo.toml`, a case in `run.sh` and `seed_corpus.py`, a `#[test]` in `harness/tests/regressions.rs`, the replay example's match, and the CI matrix.
- Generator: `generate.rs` is one `Gen` struct. Add a statement kind in `stmt`, an expression form in `expr` (keep it type-correct for the requested `Ty`), or a declaration in `top_decl`. Check validity with the `show` and `check` replay modes over random inputs. Never generate forms that make a pointer from an integer (see Sandboxing).
- `jaifmt`: a new oracle goes after the exit-code checks in `harness/src/jaifmt.rs`; a new formatter config goes in `CONFIGS`. Formatter changes reach the target through `jai-wasm`'s bundled stdlib, so rebuild the fuzz binary after editing `stdlib/Jai_Format`.
- `lsp_edits`: a new request is one more arm in `Script::step`; new insertable text goes in `SNIPPETS`. Keep `MAX_STEPS` small: every step that needs types compiles the document.

## Configuration

| Setting | Where | Default |
| --- | --- | --- |
| Interpreter budget per input | `BLOCK_BUDGET` in `harness/src/lib.rs` | 2,000,000 blocks |
| Compiler thread stack | `COMPILER_STACK` in `harness/src/lib.rs` | 256 MiB |
| Parser nesting limit | `jaic::parser::MAX_NESTING` | 1000 |
| Largest type | `jaic::types::MAX_SIZE` | 2^48 bytes |
| Polymorphic instances per procedure/struct | `MAX_INSTANCES` in `sema/mod.rs` | 2000 |
| Nested `#insert` of strings | `MAX_INSERT_DEPTH` in `sema/consteval.rs` | 256 |
| Largest compile-time value (constant, initializer, default image) | `jaic::sema::value::MAX_IMAGE` | 4 GiB |
| `#align` range | `eval_align` in `sema/structs.rs` | 0..=2^30 |
| `-max_len`, `-timeout`, `-report_slow_units`, `-rss_limit_mb` | `fuzz/run.sh` | see above |
| Interpreter budget for formatting one input twice | `FORMAT_BUDGET` in `harness/src/jaifmt.rs` | 400,000,000 blocks |
| Steps per `lsp_edits` input | `MAX_STEPS` in `harness/src/lsp_edits.rs` | 24 |
| Print diagnostics / generated source (`jaifmt`: the formatted text and both syntax trees) | `JAI_FUZZ_VERBOSE` env var | off |
| Seconds per target in CI | `workflow_dispatch` input `seconds` | 600 |

## Dependencies

cargo-fuzz 0.13 and a nightly toolchain (`rust-toolchain.toml`), `libfuzzer-sys` and `arbitrary` (crates.io, under the [dependency policy](dependency-policy.md)), and the internal crates `jaic`, `jai-wasm` (playground sandbox, bundled stdlib, LSP environment) and `jai-language-server`. `jai-wasm/build.rs` lets its unused cdylib leave libFuzzer's coverage hooks undefined on macOS when built with `--cfg fuzzing`.
