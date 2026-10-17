# Testing tools

Regression sweeps, differential testing, fuzzing and the stdlib checks. Which one to run: `jaic-sweep` for any compiler or stdlib change; `jaic-diff` after touching the interpreter, constant folding, lowering or the LLVM backend; `stdlib_runtime.py` for stdlib behaviour; the two `jaic-cli` checks run in `cargo test`; fuzzing is mostly nightly CI.

## jaic sweep

`tools/jaic-sweep.py`.

Runs `jaic` over sets of programs and reports pass/fail. Each case runs `jaic <mode> <path>` from the file's directory and compares exit code (and stdout where recorded).

| set | source | check |
|---|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record | exact stdout and exit code; a record with `error` expects any failing status and that text on stderr |
| `negative` | `kind: negative` manifest cases | check mode, non-zero exit, recorded text in stderr |
| `regressions` | `tests/compiler/regressions/*.jai`, refined by its `manifest.json` | run, exit 0 or the row's expectation |
| `modules` | `stdlib/**/tests/*.jai`, refined by each directory's `manifest.json` | run, exit 0 or the row's expectation |
| `examples` | `tests/corpus/examples.json` (`examples/tour`) | run with `args`, `stdout_contains`/`stdout_excludes` |
| `upstream` | `tools/upstream-cases.json` ([upstream corpus](upstream-corpus.md)) | `run` or `check`, plus the case's `expect` record |
| a path | that file | run |

```sh
python3 tools/jaic-sweep.py --jaic target/debug/jaic corpus negative regressions modules upstream examples --timeout 900
```

Options: `--filter`, `--verbose`, `--timeout` (default 60; 900 for the full sweep), `--jobs` (default CPU count capped so jobs times memory limit fits in RAM), `--memory-limit GIB` (default 3), `--native` (build each run case and run the executable), `--opt O0..O3`, `--sanitize address,undefined` ([sanitizers](../native/sanitizers.md)), `--headless` (programs importing `Window_Creation` are only checked or built), `--allow-stale`.

- The memory limit is passed to `jaic` as `JAIC_MEMORY_LIMIT`; `jaic` exits 120 at the first allocation past it (LLVM and child processes are not capped). The sweep refuses to run a `jaic` older than the sources in `crates/jaic`, `jaic-cli` or `jaic-llvm`, because a stale build may lack the limits that keep negative cases from exhausting memory.
- The `upstream` set needs `python3 tools/fetch_upstreams.py` first. The sweep builds missing native libraries and sets `JAIC_NATIVE_LIBS`.

**Adding cases:**

- Must be rejected: put it in `tests/corpus/negative/` and add a `kind: negative` manifest entry with `sha256` and a `negative.check` string found in the diagnostic.
- Regression: a self-checking `tests/compiler/regressions/<name>.jai` (assert or return non-zero), or a `tests/` file next to the module it covers.
- Must stop with a runtime error on every backend: `tests/corpus/positive/` with manifest `"runtime": {"error": "<message>", "stdout": "<output before the error>"}`.
- Upstream entry point: add to `tools/upstream-cases.json`. A case may list `setup` argv commands, `copy` directories into a scratch copy (for programs that write into their tree) and read-only siblings to `link` (never link a directory that is written to), `run_after` to run a built program, `files` to map repository files into the copy, and an `expect` record (values from the project, never from jaic's output).

## Differential testing

`tools/jaic-diff.py`.

jaic has no independent implementation to compare with, so it is compared with itself: every program runs through backends that share the front end and little else, and results must match.

| Backend | What runs |
| --- | --- |
| `interp` | `jaic -interp` |
| `native` / `native-O2` | `jaic -O0` / `-O2`, then the executable (LLVM) |
| `wasm` | browser engine (`crates/jai-wasm`) through `tools/jaic_diff_wasm.mjs`: interpreter compiled to wasm32, target OS `.WASM`, `SandboxHost` libc |
| `wasm-native` | `jaic -os wasm` run by node's WASI (`tools/wasi_run.mjs`); only when named in `--backends`; needs `wasm-ld` and node 24 |

```sh
cargo build --release -p jaic-cli
python3 tools/build_scripting_wasm.py --release --output /tmp/wasm
python3 tools/jaic-diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 corpus regressions modules
python3 tools/jaic-diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 --keep /tmp/gen gen:70000:3000
```

Sets are `corpus`, `regressions`, `modules`, a file path, or `gen:SEED:COUNT` (programs from `tools/jaigen.py`). A result is `(status, stdout, stderr)`: status is `exit N` or `runtime error` (only the fact is compared, since backends word failures differently); stdout must match exactly; stderr must match when all exited normally; paths are normalized (`/workspace`, `/stdlib`). Verdicts: `agree`, `DISAGREE` (also when one backend fails to compile what another compiled), `invalid` (no backend compiled it), `skip` (fewer than two ran). Exit 1 on any `DISAGREE` or `invalid`. Run it with one heavy job at a time (two native executables per program) and pick seeds no earlier campaign used.

A backend is declined for a case only for a stated reason taken from existing metadata, not wording: the wasm engine's own diagnostic codes (`unavailable`, `unknown-library`, `static-assert`, `missing-file`), no `main`, no executable written, or a `#compiler` primitive called at run time. A source mentioning `.WASM` may skip work in the browser on purpose, so its `wasm` result compares status only. `OUTPUT_VARIES` lists cases where only status is compared, each with a reason (printed target ABI, measured times, native stack traces).

**Generator.** `jaigen.py SEED [--size S]` prints one deterministic program that keeps an FNV checksum and prints it at the end. It covers integers and floats of every width, casts, structs, `using`, operator overloads, unions, arrays, loops, switches, enums, `defer`, strings, overloads, variadics, `Any`/reflection, polymorphism, `#run` constants and `print` of every type. Programs use only defined behaviour: wrapping integer arithmetic, masked shifts, no divisor of 0 or -1, in-range float to int, canonical NaN, pure procedures inside expressions (evaluation order is unspecified), constant trip counts, no pointer arithmetic or uninitialized reads. `Gen` has `expr_int/float/bool` per type, `stmt` and `program`; check a new form with `--backends interp` first.

**Reducer.** `python3 tools/jaic-reduce.py prog.jai --jaic ... --wasm ...` deletes lines and blocks while the backends still split the same way (default timeout 10 s) and writes `prog.reduced.jai`.

**A found case:** reduce it, fix the wrong backend, and keep the reduced program as a `tests/corpus/positive` case (manifest entry with `sha256` and the interpreter's `runtime` output), plus a targeted test if only one backend or optimization level was wrong (e.g. in `crates/jaic-cli/tests/native.rs`). Differential testing cannot find a bug all backends share, because they share a front end: a cast's target type does not reach an untyped literal beside a typed operand (`cast(float32) (0 - w)` with `w: u16` subtracts in `u16`).

**To change it:** a new backend goes in `ALL_BACKENDS` and `Runner.run` (map failures onto the statuses, add printed paths to `Runner.normalize`); a new set in `cases()` (give a `Case` a `root` when it reads sibling files, `skip={backend: reason}` for principled exclusions). Flags: `--jaic`, `--wasm DIR`, `--backends`, `--jobs` (3), `--timeout`/`--memory-limit` (120 s / 3 GiB), `--gen-size`, `--keep DIR`. CI runs `corpus gen:1:40` on `interp,native,native-O2` (every OS), `interp,wasm` (`scripting-wasm` job) and `interp,wasm-native`. Unit tests: `tools/test_jaic_diff.py`.

## Fuzzing

Coverage-guided fuzzing with cargo-fuzz (libFuzzer). Every panic, abort, stack overflow, hang and runaway allocation is a bug: bad input must produce a diagnostic. One round-trip property is also checked: `jaifmt` must not change what a program means.

`fuzz/` is its own Cargo workspace. `fuzz/harness` has one plain function per target taking `&[u8]` plus the grammar-based generator, with no libFuzzer dependency so `cargo test` replays inputs through the same code; `fuzz` holds one-line libFuzzer binaries in `fuzz_targets/`. Nothing uses `catch_unwind`.

| Target | What runs |
| --- | --- |
| `lexer`, `parser` | `jaic::lexer::lex`, `jaic::parser::parse_file` |
| `check` | the playground compile (bundled stdlib, every `#run`, type checking, lowering); `main` not run |
| `interp` | `check`, then `main` in the interpreter |
| `generated` | a program rendered from a typed grammar (`harness/src/generate.rs`) by `arbitrary`, then as `interp` |
| `lsp`, `lsp_json`, `lsp_edits` | language server sessions: queries at many offsets; raw JSON-RPC; a script of up to 24 edit/request/close steps over two documents |
| `jaifmt` | `Jai_Format` run twice under the interpreter; panics if its token check fails, the second pass changes text, the lexer's tokens or the parser's syntax tree differ, or the budget runs out |

- **Sandbox.** Compile-time code and `main` run in the playground sandbox (target `wasm`, in-memory stdlib, `SandboxHost`). `BLOCK_BUDGET` (2M blocks) stops loops in `#run`/`main`. Compiling harnesses run on a 256 MiB thread. The interpreter uses real memory, so an input that builds a pointer from an integer (`cast(*T)`, `xx`, `---`) can crash the fuzzer without a compiler bug. The compiler's own reads of program memory (freezing `#run` results, metaprogram strings, `compiler_*` out-parameters) must go through `Interp::read`/`write` (`interp/probe.rs`); a SEGV with one of those on the stack is a compiler bug.
- **Corpora.** `fuzz/seed_corpus.py` builds `fuzz/corpus/<target>/` from `tests/compiler/regressions`, `stdlib/**/tests`, `tests/corpus`, `examples`, `fuzz/seeds` and (for lexer/parser) `stdlib/`. Corpora are not committed; CI caches them. `fuzz/jai.dict` is the mutator dictionary.
- **Run:** `cargo install cargo-fuzz`, then `fuzz/run.sh check 1200 4` (target, seconds, workers). It builds without a sanitizer but with debug assertions (ASan is off on purpose) and collects findings in `fuzz/artifacts/<target>/`. Per-target `-max_len`, `-timeout` and `-rss_limit_mb=4096` are in `run.sh`. Throughput varies from about 10,000 executions per second (`lexer`) to under 20 (`jaifmt`); give slow targets longer runs.
- **Triage:** `cargo run --release --manifest-path fuzz/harness/Cargo.toml --example replay -- check fuzz/artifacts/check/crash-*` (`JAI_FUZZ_VERBOSE=1` for diagnostics; `replay show <input>` prints a generated program). Minimize with `cargo fuzz tmin --sanitizer none --debug-assertions`, or by hand.
- **Regressions.** Every fixed crash gets the minimized input in `fuzz/regressions/<target>/` (replayed on every push by `fuzz/harness/tests/regressions.rs`: `cargo test --manifest-path fuzz/harness/Cargo.toml`) and a test next to the fix.
- **CI**: the `fuzz-regressions` job of `ci.yml` replays the saved inputs (and checks `fuzz/Cargo.lock` dependency age; keep it `--locked`); `fuzz.yml` fuzzes nightly at 03:23 UTC or by dispatch (`seconds` input, default 600), one job per target with a cached, minimized corpus; a finding fails the job and uploads `fuzz-artifacts-<target>`.
- The fuzz workspace has its own lockfile: after changing a dependency of a shared crate, update both, e.g. `cargo update --manifest-path fuzz/harness/Cargo.toml -p jailint`.

Limits that exist because fuzzing found them (each has a regression): `MAX_NESTING` (parser, 1000), `MAX_SIZE` (types, 2^48 bytes), `MAX_IMAGE` (compile-time values, 4 GiB), `MAX_INSTANCES` (2000), `MAX_INSERT_DEPTH` (256), `#align` range 0..=2^30, `FORMAT_BUDGET` and `MAX_STEPS` (24, `lsp_edits`) in the harness. Span-reading code must use `SourceMap::snippet`/`try_get` because declaration spans can be `Span::NONE`; `jaic::docs` answers nothing for a stale offset that is not a character boundary.

**To change it:** a new target needs `pub fn <name>(data: &[u8])` in `harness/src/lib.rs`, a file in `fuzz_targets/`, a `[[bin]]` in `fuzz/Cargo.toml`, cases in `run.sh` and `seed_corpus.py`, a test in `harness/tests/regressions.rs`, the replay example's match and the CI matrix. Generator: add statement kinds in `stmt`, expressions in `expr` (type-correct for the requested type) or declarations in `top_decl`; never generate pointers made from integers. `jaifmt` oracles go after the exit-code checks in `harness/src/jaifmt.rs`; rebuild after editing `stdlib/Extensions/Jai_Format`. A new `lsp_edits` request is an arm in `Script::step`.

## Stdlib runtime tests

Every top-level `*.jai` file of a `stdlib/**/tests/` directory, and of `tests/compiler/regressions/`, is a case that must build and exit 0 in each mode. A directory's optional `manifest.json` refines cases (expected output, required platform, native-only services) and can add cases that live in sub-directories; `tools/stdlib_test_catalog.py` documents the row keys. Files in sub-directories (`support/`, `fixtures/`, `native/`) are helpers and never discovered on their own.

```sh
python3 tools/build_native_libs.py
python3 tools/stdlib_runtime.py --jaic target/release/jaic --modes interp,native,wasm-interp,wasm-native --coverage target/stdlib-reachability.txt
python3 tools/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib --jailint target/debug/jailint --jaifmt target/jaifmt
```

`check_stdlib_suites.py` runs every module driver with native setup, child processes or target matrices, continues after a failure and exits nonzero if any failed. `stdlib_runtime.py` takes `--filter`, `--modes`, `--opt O0|O2` and `--report FILE`. Native SDK and graphics families are reported as unavailable under WASM, never as passed; missing SDKs, fonts or display servers on an applicable host are failures. Positive cases compare exact stdout (and stderr when a contract is given); check-fail cases need a nonzero status and a `diagnostic_contains`; privacy cases also compile a positive import baseline. Tests can read the mode to avoid what a mode cannot do. Export counts and `tools/stdlib_coverage.py` reachability are not completeness measures.

Put a module's cases in its `tests/` directory (a manifest row for anything beyond "builds and exits 0"), keep compiler and ABI regressions in `tests/compiler/regressions/`, and put specialised cases in their module driver so native preparation and O0/O2 coverage stay required. Linux needs native headers, Xvfb/Mesa, fonts and audio backends; Windows needs MSVC/LLVM and Mesa.

## Stdlib target check

Type-checks every stdlib module for every target, including code nothing calls. [Dead-code elimination](../language/dead-code-elimination.md) only checks reachable module code, so a type error in `#if OS == .MACOS` code or an uncalled procedure would otherwise hide until a user on that platform hits it.

The test discovers `stdlib/X.jai` files and public `module.jai` facades (`stdlib/internal` is checked through facades to preserve import privacy), writes `#import "X"; main :: () {}` and runs `jaic -check -no_dce -os <os> -cpu <cpu>` for linux, macos and windows on x64 and arm64, and for wasm. The first error of each failure is reduced to `file: message` (no path, line or column) and the set must equal `tests/compiler/stdlib-targets.txt` exactly: an unlisted failure, a changed first error, and a listed module that now passes all fail the test. Listed failures are intentional: modules for other platforms that stop with `#assert`, procedures that stop the build on unsupported targets, and bindings with no wasm build. Checking stops at the first error, so those modules are only checked up to the `#assert`.

```sh
cargo test -p jaic-cli --test stdlib_targets    # --release for speed, --no-default-features without LLVM
```

Change it by editing `tests/compiler/stdlib-targets.txt` only after confirming the restriction is real (`skip Module`, `Module target: file: first error`; `#` comments). A new platform-only module should stop on the wrong target with an explanatory `#assert` and get one line per refused target. A new target extends `TARGETS` in the test. Source checks do not prove native SDK or runtime availability.

## Objective-C selector check

Selectors are strings (`sel_registerName("setTitle:")`), so a typo or missing `:` type-checks and then fails at run time with "unrecognized selector". The test reads literal `sel_registerName` calls (and explicit `#foreign` aliases of that symbol) in the stdlib:

1. **Arity, every host.** The arguments after a selector in a call must equal its number of `:`; a selector-table field must be the selector with `:` written `_`. An empty scan fails.
2. **Existence, macOS only.** The receiver gives the class (`objc_getClass("X")` is a class method; `self` is an instance method of the procedure's `self` type, with `Lightweight*` views mapped as `Objective_C.class` does). A Jai program run under `jaic -interp` asks the Objective-C runtime (`class_getInstanceMethod`, `class_getClassMethod`, protocol method descriptions, declared properties, class-cluster subclasses) and its output must match `tests/compiler/objc-selectors.txt`. Elsewhere it only has to type-check for macOS.

Calls on local objects get the arity check but not the existence check, and selectors stored in variables are not checked at all, so pass selectors as literals. When it reports a missing method, fix the binding; if the method only exists on newer macOS than CI or the class is private, list it in `tests/compiler/objc-selectors.txt` (`missing <class> <instance|class> <selector>`, `unknown <class>`; `#` reason). Remove a listed line once it is found. Run with `cargo test -p jaic-cli --test objc_selectors`.
