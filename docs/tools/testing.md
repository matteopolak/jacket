# Testing tools

Which one to run:

- `jaic_sweep` for any compiler change.
- `jaic_diff` after touching the interpreter, constant folding, lowering or the LLVM backend.
- [`jacket test`](jacket-test.md) for stdlib module behaviour.
- `stdlib_runtime.py` for the compiler regressions.
- The `jaic-cli` target and selector checks run in `cargo test`.
- Fuzzing is mostly nightly CI.

Rust test helpers and the snapshot workflow (`INSTA_UPDATE=always`) are in [the Rust test harness](test-harness.md).

## jaic sweep

`tools/differential/jaic_sweep.py` runs `jaic <mode> <path>` from each file's directory and compares the exit code (and stdout where recorded).

| set | source |
|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record |
| `negative` | `kind: negative` manifest cases: check mode, non-zero exit, recorded text in stderr |
| `regressions` | `tests/compiler/regressions/*.jai`, refined by its `manifest.json` |
| `examples` | `tests/corpus/examples.json`, one folder per example in `examples/` |
| `upstream` | `tools/upstream-cases.json` ([upstream corpus](upstream-corpus.md)) |
| a path | that file |

```sh
PYTHONPATH=tools python3 tools/differential/jaic_sweep.py --jaic target/debug/jaic corpus negative regressions upstream examples --timeout 900
```

Options: `--filter`, `--verbose`, `--timeout`, `--jobs`, `--memory-limit GIB` (default 3), `--native`, `--opt O0..O3`, `--sanitize address,undefined` ([sanitizers](../native/sanitizers.md)), `--headless`, `--allow-stale`.

- The memory limit reaches `jaic` as `JAIC_MEMORY_LIMIT`; it exits 120 past it.
- The sweep refuses a `jaic` older than the compiler sources.
- `upstream` needs `PYTHONPATH=tools python3 tools/fetch_upstreams.py` first.
- `cargo test -p jaic-cli --test examples` checks every program in `examples/` for each OS its row in `PROGRAMS` lists. A new example fails until it has a row and a case in `tests/corpus/examples.json`.

Adding cases:

- Must be rejected: `tests/corpus/negative/` plus a `kind: negative` manifest entry with `sha256` and a `negative.check` string found in the diagnostic.
- Regression: a self-checking `tests/compiler/regressions/<name>.jai` (assert or return non-zero).
- Must stop with a runtime error: `tests/corpus/positive/` with `"runtime": {"error": "<message>", "stdout": "<output before the error>"}`.
- Upstream entry point: `tools/upstream-cases.json`. A case may list `setup`, `copy`, `link`, `run_after`, `files` and an `expect` record (values from the project, never from jaic's output).

Output must not depend on threads or hash order: [determinism](determinism.md). Every diagnostic needs a test: [diagnostic coverage](diagnostic-coverage.md).

## Differential testing

`tools/differential/jaic_diff.py` compares jaic with itself: each program runs through backends that share the front end and little else, and results must match.

| Backend | What runs |
| --- | --- |
| `interp` | `jaic -interp` |
| `native` / `native-O2` | `jaic -O0` / `-O2`, then the executable |
| `wasm` | the browser engine (`crates/jai-wasm`) via `jaic_diff_wasm.mjs` |
| `wasm-native` | `jaic -os wasm` under node's WASI; only when named in `--backends`; needs `wasm-ld` and node 24 |

```sh
cargo build --release -p jaic-cli
PYTHONPATH=tools python3 tools/browser/build_scripting_wasm.py --release --output /tmp/wasm
PYTHONPATH=tools python3 tools/differential/jaic_diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 corpus regressions
PYTHONPATH=tools python3 tools/differential/jaic_diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 gen:70000:3000
```

Sets: `corpus`, `regressions`, `modules`, a file, or `gen:SEED:COUNT` (programs from `jaigen.py`, deterministic, defined behaviour only). Verdicts are `agree`, `DISAGREE`, `invalid` and `skip`; exit 1 on `DISAGREE` or `invalid`. Stdout must match exactly. A failure is compared as a fact only, not its wording. Run one heavy job at a time, and use seeds no earlier campaign used.

- Reduce a split with `jaic_reduce.py prog.jai --jaic ... --wasm ...`.
- A found case: reduce it, fix the wrong backend, keep the reduced program as a `tests/corpus/positive` case.
- It cannot find bugs all backends share, since they share a front end.
- To change it: a new backend goes in `ALL_BACKENDS` and `Runner.run`; a new set in `cases()`; `OUTPUT_VARIES` lists cases that compare status only. CI runs `corpus gen:1:40` per host.

### Comparing two compilers

`tools/differential/compare_compilers.py` takes two binaries A and B and reports where they disagree on many small programs (old against new `jaic`, `-interp` against native, ...).

```sh
PYTHONPATH=tools python3 tools/differential/compare_compilers.py --a OLD --b target/release/jaic --mode interp --jobs 4 grid
```

Sets: `grid` (tables in `compare_grid.py`), `rewrite:PATHS` (text edits of existing programs), `gen:SEED:COUNT`, files and folders. `--mode` is `check`, `interp`, `run` or `run-opt`, per side with `--mode-a`/`--mode-b`. `--sandbox` runs without network. `--reduce` shrinks each cluster. The report is `target/compare/report.md`. To add a grid axis, write a `Program(name, source, tags)` generator in `compare_grid.py` and list it in `AXES`.

## Fuzzing

cargo-fuzz (libFuzzer). Every panic, abort, hang, stack overflow and runaway allocation is a bug: bad input must give a diagnostic. `jaifmt` must also not change what a program means.

`fuzz/` is its own Cargo workspace. `fuzz/harness` has one function per target taking `&[u8]`, plus the grammar-based generator; `cargo test` replays inputs through it. `fuzz/targets.json` is the one list of targets (length, timeout, seed tree). Targets: `lexer`, `parser`, `check`, `interp`, `generated`, `lsp`, `lsp_json`, `lsp_edits`, `lsp_typing`, `lsp_workspace`, `jaifmt`, `jailint_fix`, `jacket_config`, `metaprogram`.

```sh
cargo install cargo-fuzz
fuzz/run.sh check 1200 4        # target, seconds, workers
cargo run --release --manifest-path fuzz/harness/Cargo.toml --example replay -- check fuzz/artifacts/check/crash-*
```

- Compile-time code and `main` run in the playground sandbox. `BLOCK_BUDGET` stops loops. The interpreter uses real memory, so an input building a pointer from an integer can crash the fuzzer without a compiler bug; the compiler's own reads of program memory go through `Interp::read`/`write`.
- Corpora are built by `fuzz/seed_corpus.py` and not committed; CI caches them.
- Every fixed crash gets its minimized input in `fuzz/regressions/<target>/` and a test next to the fix.
- CI: `fuzz-regressions` in `ci.yml` replays the saved inputs; `fuzz.yml` fuzzes nightly, one job per target.
- The fuzz workspace has its own lockfile. After a dependency change of a shared crate, update both.
- Span-reading code must use `SourceMap::snippet`/`try_get`; declaration spans can be `Span::NONE`.

To add a target: `pub fn <name>(data: &[u8])` in `harness/src/lib.rs`, a file in `fuzz_targets/`, an entry in `targets.json`, then `PYTHONPATH=tools python3 tools/fuzz/sync_targets.py`. For the generator, add statement kinds in `stmt`, expressions in `expr`, declarations in `top_decl`; never generate pointers made from integers. Stdlib procedures are fuzzed separately: [stdlib fuzzing](stdlib-fuzzing.md).

## Stdlib runtime tests

Stdlib tests run under [`jacket test`](jacket-test.md) (every folder under `[test] paths` in `jacket.toml`): natively, interpreted, at `-O2`, as WebAssembly on Linux x64 (`tools/ci/stdlib_jacket_test.sh`) and under sanitizers (`stdlib_jacket_sanitize.sh`). Put a module's tests in its `tests/` directory.

`tools/stdlib/stdlib_runtime.py` runs the compiler regressions in `tests/compiler/regressions/` in modes `interp`, `native`, `wasm-interp`, `wasm-native` and `check`. Each top-level `*.jai` is a case that must build and exit 0. The folder's `manifest.json` refines cases (expected output, platform, unsupported modes); `interpreter_only: true` marks a metaprogram that works at compile time.

```sh
PYTHONPATH=tools python3 tools/native/build_native_libs.py
PYTHONPATH=tools python3 tools/stdlib/stdlib_runtime.py --jaic target/release/jaic --modes interp,native
PYTHONPATH=tools python3 tools/stdlib/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib
```

The CI jobs run these with a release build of `jaic` and `jacket`, Windows included: the interpreter runs the tests' programs, and a debug build of it is about six times slower there (200000 allocator blocks take 10 s instead of 1.8 s), so a slow-looking Windows test usually means the wrong build.

`check_stdlib_suites.py` runs the module drivers that need native setup or target matrices; `--suite`, `--exclude` and `--jobs` select and parallelize. `GROUPS` keeps suites with shared mutable resources together. Tooling rebuilds and reconfigures the private native-tool package while bindings snapshots and executes it, so they run consecutively in one group. Portable/platform and audio/window keep their independent groups. When adding a suite, group it with existing readers/writers of the same package, scratch directory or device; `--jobs` limits concurrent groups.

## Stdlib target check

Type-checks every stdlib module for every target, including code nothing calls (dead-code elimination would hide a type error in `#if OS == .MACOS` code). It runs `jaic -check -no_dce -os <os> -cpu <cpu>` for linux, macos and windows on x64 and arm64, and wasm. The first error of each failure must equal `tests/compiler/stdlib-targets.txt` exactly.

```sh
cargo test -p jaic-cli --test stdlib_targets
```

Edit the list only after confirming the restriction is real (`skip Module`, `Module target: file: first error`). A platform-only module should stop on the wrong target with an `#assert`. A new target extends `TARGETS` in the test.

The same CI step also runs `jacket test --doc --check-only --os <os>` for linux, macos, windows and wasm, which type-checks every doc example for each target ([Checking examples for another OS](jacket-test.md#checking-examples-for-another-os)).

## Objective-C selector check

Selectors are strings, so a typo type-checks and fails at run time. `cargo test -p jaic-cli --test objc_selectors` reads literal `sel_registerName` calls: the argument count must equal the number of `:` (every host), and on macOS the method must exist (checked via the Objective-C runtime against `tests/compiler/objc-selectors.txt`). Pass selectors as literals. If a method only exists on newer macOS, list it in that file with a reason.
