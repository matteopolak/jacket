# Testing tools

Regression sweeps, differential testing, fuzzing and the stdlib checks. Which one to run: `jaic_sweep` for any compiler change; `jaic_diff` after touching the interpreter, constant folding, lowering or the LLVM backend; [`jacket test`](jacket-test.md) for stdlib module behaviour; `stdlib_runtime.py` for the compiler regressions; the two `jaic-cli` checks run in `cargo test`; fuzzing is mostly nightly CI. Rust test helpers and the snapshot workflow (`INSTA_UPDATE=always`) are in [the Rust test harness](test-harness.md).

## jaic sweep

`tools/differential/jaic_sweep.py`.

Runs `jaic` over sets of programs and reports pass/fail. Each case runs `jaic <mode> <path>` from the file's directory and compares exit code (and stdout where recorded).

| set | source | check |
|---|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record | exact stdout and exit code; a record with `error` expects any failing status and that text on stderr |
| `negative` | `kind: negative` manifest cases (run by CI in the O0 sanitizer sweep) | check mode, non-zero exit, recorded text in stderr; a `disabled` reason switches a case off and is reported |
| `regressions` | `tests/compiler/regressions/*.jai`, refined by its `manifest.json` | run, exit 0 or the row's expectation |
| `examples` | `tests/corpus/examples.json` (one folder per example in `examples/`) | run with `args`, `stdout_contains`/`stdout_excludes` |
| `upstream` | `tools/upstream-cases.json` ([upstream corpus](upstream-corpus.md)) | `run` or `check`, plus the case's `expect` record |
| a path | that file | run |

```sh
PYTHONPATH=tools python3 tools/differential/jaic_sweep.py --jaic target/debug/jaic corpus negative regressions upstream examples --timeout 900
```

Options: `--filter`, `--verbose`, `--timeout` (default 60; 900 for the full sweep), `--jobs` (default CPU count capped so jobs times memory limit fits in RAM), `--memory-limit GIB` (default 3), `--native` (build each run case and run the executable), `--opt O0..O3`, `--sanitize address,undefined` ([sanitizers](../native/sanitizers.md)), `--headless` (programs importing `Window_Creation` are only checked or built), `--allow-stale`.

- The memory limit is passed to `jaic` as `JAIC_MEMORY_LIMIT`; `jaic` exits 120 at the first allocation past it (LLVM and child processes are not capped). The sweep refuses to run a `jaic` older than the sources in `crates/jaic`, `jaic-cli` or `jaic-llvm`, because a stale build may lack the limits that keep negative cases from exhausting memory.
- The `upstream` set needs `PYTHONPATH=tools python3 tools/fetch_upstreams.py` first. The sweep builds missing native libraries and sets `JAIC_NATIVE_LIBS`.

Every program in `examples/` is also checked by `cargo test -p jaic-cli --test examples`: `jaic -check -os <os>` for each OS its row in `PROGRAMS` lists, with the reason when one is left out (the HTTP and window examples are native only). A file another example `#load`s (a workspace's sections, `http/notes.jai`) is checked through that program; a new example fails the test until it has a row and a case in `tests/corpus/examples.json`.

**Adding cases:**

- Must be rejected: put it in `tests/corpus/negative/` and add a `kind: negative` manifest entry with `sha256` and a `negative.check` string found in the diagnostic.
- Regression: a self-checking `tests/compiler/regressions/<name>.jai` (assert or return non-zero), or a `tests/` file next to the module it covers.
- Must stop with a runtime error on every backend: `tests/corpus/positive/` with manifest `"runtime": {"error": "<message>", "stdout": "<output before the error>"}`.
- Upstream entry point: add to `tools/upstream-cases.json`. A case may list `setup` argv commands, `copy` directories into a scratch copy (for programs that write into their tree) and read-only siblings to `link` (never link a directory that is written to), `run_after` to run a built program, `files` to map repository files into the copy, and an `expect` record (values from the project, never from jaic's output).

Output must not depend on threads or hash order: [determinism checks](determinism.md).

Every compiler diagnostic needs a test that expects its text: [diagnostic coverage gate](diagnostic-coverage.md).

Rust line coverage: [Rust code coverage](rust-coverage.md).

Stdlib procedures are fuzzed from their types by `tools/fuzz/stdlib_fuzz.py`, see [stdlib fuzzing](stdlib-fuzzing.md).

## Differential testing

`tools/differential/jaic_diff.py`.

jaic has no independent implementation to compare with, so it is compared with itself: every program runs through backends that share the front end and little else, and results must match.

| Backend | What runs |
| --- | --- |
| `interp` | `jaic -interp` |
| `native` / `native-O2` | `jaic -O0` / `-O2`, then the executable (LLVM) |
| `wasm` | browser engine (`crates/jai-wasm`) through `tools/differential/jaic_diff_wasm.mjs`: interpreter compiled to wasm32, target OS `.WASM`, `SandboxHost` libc |
| `wasm-native` | `jaic -os wasm` run by node's WASI (`tools/wasi_run.mjs`); only when named in `--backends`; needs `wasm-ld` and node 24 |

```sh
cargo build --release -p jaic-cli
PYTHONPATH=tools python3 tools/browser/build_scripting_wasm.py --release --output /tmp/wasm
PYTHONPATH=tools python3 tools/differential/jaic_diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 corpus regressions
PYTHONPATH=tools python3 tools/differential/jaic_diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 --keep /tmp/gen gen:70000:3000
```

Sets are `corpus`, `regressions`, `modules`, a file path, or `gen:SEED:COUNT` (programs from `tools/differential/jaigen.py`). A result is `(status, stdout, stderr)`: status is `exit N` or `runtime error` (only the fact is compared, since backends word failures differently); stdout must match exactly; stderr must match when all exited normally; paths are normalized (`/workspace`, `/stdlib`). Verdicts: `agree`, `DISAGREE` (also when one backend fails to compile what another compiled), `invalid` (no backend compiled it), `skip` (fewer than two ran). Exit 1 on any `DISAGREE` or `invalid`. Run it with one heavy job at a time (two native executables per program) and pick seeds no earlier campaign used.

A backend is declined for a case only for a stated reason taken from existing metadata, not wording: the wasm engine's own diagnostic codes (`unavailable`, `unknown-library`, `static-assert`, `missing-file`), no `main`, no executable written, or a `#compiler` primitive called at run time. A source mentioning `.WASM` may skip work in the browser on purpose, so its `wasm` result compares status only. `OUTPUT_VARIES` lists cases where only status is compared, each with a reason (printed target ABI, measured times, native stack traces).

**Generator.** `jaigen.py SEED [--size S]` prints one deterministic program that keeps an FNV checksum and prints it at the end. It covers integers and floats of every width, casts, structs, `using`, operator overloads, unions, arrays, loops, switches, enums, `defer`, strings, overloads, variadics, `Any`/reflection, polymorphism (procedures and structs, `#modify` binding a result type or rejecting a call, `#bake_arguments` of procedures and structs), `#run` constants, `#add_context` fields read through `push_context`, a counting allocator wrapped around the default one, and `print` of every type. Programs use only defined behaviour: wrapping integer arithmetic, masked shifts, no divisor of 0 or -1, in-range float to int, canonical NaN (bare float literals pass through `opaque_*` so the compiler cannot fold ones that overflow or divide by zero), pure procedures inside expressions (evaluation order is unspecified), constant trip counts, no pointer arithmetic or uninitialized reads. `Gen` has `expr_int/float/bool` per type, `stmt` and `program`; check a new form with `--backends interp` first.

**Reducer.** `PYTHONPATH=tools python3 tools/differential/jaic_reduce.py prog.jai --jaic ... --wasm ...` deletes lines and blocks while the backends still split the same way (default timeout 10 s) and writes `prog.reduced.jai`.

**A found case:** reduce it, fix the wrong backend, and keep the reduced program as a `tests/corpus/positive` case (manifest entry with `sha256` and the interpreter's `runtime` output), plus a targeted test if only one backend or optimization level was wrong (e.g. in `crates/jaic-cli/tests/native.rs`). Differential testing cannot find a bug all backends share, because they share a front end: a cast's target type does not reach an untyped literal beside a typed operand (`cast(float32) (0 - w)` with `w: u16` subtracts in `u16`).

**To change it:** a new backend goes in `ALL_BACKENDS` and `Runner.run` (map failures onto the statuses, add printed paths to `Runner.normalize`); a new set in `cases()` (give a `Case` a `root` when it reads sibling files, `skip={backend: reason}` for principled exclusions). Flags: `--jaic`, `--wasm DIR`, `--backends`, `--jobs` (3), `--timeout`/`--memory-limit` (120 s / 3 GiB), `--gen-size`, `--keep DIR`. CI runs `corpus gen:1:40` once per host in the `test` job's `diff` leg, on `interp,native,native-O2`; Linux x64 adds `wasm` (the leg builds the browser bundle first with `build_scripting_wasm.py --release --output target/ci-wasm` and passes `--wasm`) and `wasm-native`. Unit tests: `tools/differential/test_jaic_diff.py`.

### Comparing two compilers

`tools/differential/compare_compilers.py` takes two opaque binaries, A and B, and reports where they disagree on many small programs: an old `jaic` against a new one, `-interp` against a native build of the same binary, or any other pair. It targets pairs of features (a `.{...}` literal as an operator operand, `ifx` with an enum shorthand) that random programs rarely combine.

```
PYTHONPATH=tools python3 tools/differential/compare_compilers.py --a OLD --b target/release/jaic --mode interp --jobs 4 grid
PYTHONPATH=tools python3 tools/differential/compare_compilers.py --a target/release/jaic --mode-a interp --b target/release/jaic --mode-b run grid
PYTHONPATH=tools python3 tools/differential/compare_compilers.py --a OLD --b NEW --reduce --sandbox rewrite:tests/compiler gen:90000:100 prog.jai dir/
```

Sets: `grid[:context,poly,using]` (the tables in `compare_grid.py`), `rewrite:PATHS` (text-level edits of existing programs from `compare_rewrite.py`: erase a literal's type, swap commutative operands, inline a single-use local, `x: T = e` to `cast`, `x := e` to `type_of`, ...; default paths `tests/compiler/{semantics,language}`; some rewrites are tagged `noisy:` because they can change meaning), `gen:SEED:COUNT` (`jaigen.py`), files and folders. `--mode` is `check` (default), `interp`, `run` or `run-opt` (build, then execute), per side with `--mode-a`/`--mode-b`; `--args-a`/`--args-b` add arguments; `--sandbox` runs every compile and program without network and with minimal file access (it cannot start inside another sandbox); `--only TEXT`, `--limit N` pick a subset. Each side's command line comes from `tools/bench/dialects.py`, chosen by probing the binary's `-help`, so a binary without jaic's flags gets the portable command line.

Per program each side records compiled or not, exit code, stdout and the first error line without paths, positions or colour; a non-zero exit of the program is compared only as a fact. A pair is `agree`, `both_reject` (also agreement), `a_only`, `b_only`, `output_differs` or `failure` (timeout or crash; the run continues). Exit 1 on any disagreement, 2 when a generated program does not parse (`jaic -check` on B, or `--check-with`, runs at generation time: a syntax error there is a template bug, `--no-self-check` skips it).

Read `target/compare/report.md` (`--out` moves it; `results.json` has the same data): counts per class, disagreements clustered by error text with the smallest example of each (`--reduce` shrinks it by deleting lines while the pair still splits the same way), clusters by feature tag (`kind:`, `context:`, `form:`, `rewrite:` ...) with their share, and the programs both sides reject that were not marked `variant:near_miss`, which are either a template mistake or a gap both share.

**To add a grid axis:** write a generator of `Program(name, source, tags)` in `compare_grid.py` (tags start with `axis:NAME`; every program prints a deterministic line) and list it in `AXES`. A new row or column of an existing table is one entry in `KINDS`/`CONTEXTS`, `FORMS`/`ARGS` or `SETUPS`/`ACCESSES`. Unit tests (`PYTHONPATH=tools python3 tools/differential/test_compare_compilers.py`; the end-to-end ones need `JAIC` or a built `target/release/jaic`) check the grid is well formed. Limits: rewrites are regex based, so they miss multi-line constructs and can edit text inside block comments; classification compares stdout only (stderr of a program is not).

## Fuzzing

Coverage-guided fuzzing with cargo-fuzz (libFuzzer). Every panic, abort, stack overflow, hang and runaway allocation is a bug: bad input must produce a diagnostic. One round-trip property is also checked: `jaifmt` must not change what a program means.

`fuzz/` is its own Cargo workspace. `fuzz/harness` has one plain function per target taking `&[u8]` plus the grammar-based generator, with no libFuzzer dependency so `cargo test` replays inputs through the same code; `fuzz` holds one-line libFuzzer binaries in `fuzz_targets/`. Nothing uses `catch_unwind`.

| Target | What runs |
| --- | --- |
| `lexer`, `parser` | `jaic::lexer::lex`, `jaic::parser::parse_file` |
| `check` | the playground compile (bundled stdlib, every `#run`, type checking, lowering); `main` not run |
| `interp` | `check`, then `main` in the interpreter |
| `generated` | a program rendered from a typed grammar (`harness/src/generate.rs`: polymorphic procedures, `#modify`, `#bake_arguments`, `#add_context` with `push_context`, macros, edge shapes) by `arbitrary`, then as `interp` |
| `lsp`, `lsp_json`, `lsp_edits` | language server sessions: queries at many offsets; raw JSON-RPC; a script of up to 24 edit/request/close steps over two documents |
| `lsp_typing` | the [typing-session checker](lsp-fuzzing.md) with the fuzzer's bytes choosing the keystrokes: answers must match a fresh session and the original text after undo |
| `jaifmt` | `Jai_Format` run twice under the interpreter; panics if its token check fails, the second pass changes text, the lexer's tokens or the parser's syntax tree differ, the program compiles or prints differently (numbers blanked, since lines move), or the budget runs out |
| `jailint_fix` | `source.fixAll.jailint` applied until nothing is left: the result must parse, the fixes must settle within 8 passes, add no compile errors to a program that had some, and leave a program that compiled printing the same output and exit code |
| `jacket_config` | a `jacket.toml`: loading never panics, loading and parsing agree, and settings serialize and read back identically |
| `metaprogram` | a `#run` built from the bytes that creates workspaces, sets build options, intercepts messages and feeds them generated sources (with `#insert` and `compiler_get_nodes`), including misuse such as waiting without intercepting or using a destroyed workspace, run in the playground sandbox: no panic, hang or abort |
| `lsp_workspace` | up to three NUL-separated files (`main.jai`, `a.jai`, `b.jai`, loading one another) in one `Session`, then edits in any file: while the workspace is error-free, renaming a name to a fresh one (applied in every file it touches) leaves it error-free and renaming back restores every file byte for byte; after the edits every file's diagnostics equal a fresh session's on the same texts (inputs that leave a literal unclosed are not compared) |

- **Sandbox.** Compile-time code and `main` run in the playground sandbox (target `wasm`, in-memory stdlib, `SandboxHost`). `BLOCK_BUDGET` (2M blocks) stops loops in `#run`/`main`. Compiling harnesses run on a 256 MiB thread. The interpreter uses real memory, so an input that builds a pointer from an integer (`cast(*T)`, `xx`, `---`) can crash the fuzzer without a compiler bug. The compiler's own reads of program memory (freezing `#run` results, metaprogram strings, `compiler_*` out-parameters) must go through `Interp::read`/`write` (`interp/probe.rs`); a SEGV with one of those on the stack is a compiler bug.
- **Corpora.** `fuzz/seed_corpus.py` builds `fuzz/corpus/<target>/` from `tests/compiler/regressions`, `stdlib/**/tests`, `tests/corpus`, `examples`, `fuzz/seeds` and (for lexer/parser) `stdlib/`. Corpora are not committed; CI caches them. `fuzz/jai.dict` is the mutator dictionary.
- **Targets.** `fuzz/targets.json` is the one list of targets, with each one's `-max_len`, `-timeout` and seed tree and size limit. `run.sh` and `seed_corpus.py` read it, and `fuzz.yml` builds its matrix from `python3 fuzz/seed_corpus.py --matrix`. `PYTHONPATH=tools python3 tools/fuzz/sync_targets.py` generates the `[[bin]]` entries in `fuzz/Cargo.toml` and the per-target tests in `harness/tests/regressions.rs` from it (`--check` fails when they differ; the Python tool tests run it).
- **Run:** `cargo install cargo-fuzz` (CI uses `taiki-e/install-action`), then `fuzz/run.sh check 1200 4` (target, seconds, workers). It builds without a sanitizer but with debug assertions (ASan is off on purpose) and collects findings in `fuzz/artifacts/<target>/`. Per-target `-max_len` and `-timeout` come from `targets.json`; `-rss_limit_mb=4096` is in `run.sh`. Throughput varies from about 10,000 executions per second (`lexer`) to under 20 (`jaifmt`); give slow targets longer runs.
- **Triage:** `cargo run --release --manifest-path fuzz/harness/Cargo.toml --example replay -- check fuzz/artifacts/check/crash-*` (`JAI_FUZZ_VERBOSE=1` for diagnostics; `replay show <input>` prints a generated program). Minimize with `cargo fuzz tmin --sanitizer none --debug-assertions`, or by hand.
- **Regressions.** Every fixed crash gets the minimized input in `fuzz/regressions/<target>/` (replayed on every push by `fuzz/harness/tests/regressions.rs`: `cargo test --manifest-path fuzz/harness/Cargo.toml`) and a test next to the fix.
- **CI**: the `fuzz-regressions` job of `ci.yml` replays the saved inputs (and checks `fuzz/Cargo.lock` dependency age; keep it `--locked`); `fuzz.yml` fuzzes nightly at 03:23 UTC or by dispatch (`cargo fuzz cmin` runs only in the Sunday run and by dispatch) (`seconds` input, default 600), one job per target with a cached, minimized corpus; a finding fails the job, uploads `fuzz-artifacts-<target>` and writes the panic message and the input (hex dump and escaped text, cut to 512 bytes and 2000 characters) of the first three findings to the job summary (`tools/fuzz/crash_summary.py` replays each through `fuzz/harness/examples/replay.rs` with debug assertions and overflow checks on; a hang or an input that does not reproduce outside the fuzzer is said so instead of a message). **Time.** libFuzzer's fork mode replays the whole corpus in one process before `-max_total_time` starts counting, and `cargo fuzz cmin` replays it again, so a slow target (about 2 executions per second on `lsp_json`) with 1,600 corpus files spent 13 minutes in each and passed the 45-minute job timeout. `targets.json` gives such targets a `corpus_max`; `run.sh` keeps a random sample of that many files (`seed_corpus.py --trim`; the seeds are re-added every night, so the sample rotates). The `Fuzz` and `Minimize the corpus` steps have their own timeouts (25 and 10 minutes) so the corpus is still saved. The release build of the target takes about 2 minutes and is not the problem.
- The fuzz workspace has its own lockfile: after changing a dependency of a shared crate, update both, e.g. `cargo update --manifest-path fuzz/harness/Cargo.toml -p jailint`.

Limits that exist because fuzzing found them (each has a regression): `MAX_NESTING` (parser, 1000), `MAX_SIZE` (types, 2^48 bytes), `MAX_IMAGE` (compile-time values, 4 GiB), `MAX_INSTANCES` (2000), `MAX_INSERT_DEPTH` (256), `#align` range 0..=2^30, `FORMAT_BUDGET` and `MAX_STEPS` (24, `lsp_edits`) in the harness. Span-reading code must use `SourceMap::snippet`/`try_get` because declaration spans can be `Span::NONE`; `jaic::docs` answers nothing for a stale offset that is not a character boundary.

**To change it:** a new target needs `pub fn <name>(data: &[u8])` in `harness/src/lib.rs`, a file in `fuzz_targets/`, an entry in `fuzz/targets.json` (which feeds `run.sh`, `seed_corpus.py` and the CI matrix), then `PYTHONPATH=tools python3 tools/fuzz/sync_targets.py` for the `[[bin]]` and the regression test, and the replay example's match. Generator: add statement kinds in `stmt`, expressions in `expr` (type-correct for the requested type) or declarations in `top_decl`; never generate pointers made from integers. `jaifmt` oracles go after the exit-code checks in `harness/src/jaifmt.rs`; rebuild after editing `stdlib/Extensions/Jai_Format`. A new `lsp_edits` request is an arm in `Script::step`. A target whose seeds are not Jai files names a seed group in `seed_corpus.py` (`configs` builds the `jacket_config` inputs from `fuzz/seeds_config/*.toml` and the repository's config files). `Meaning` in `harness/src/lib.rs` is the shared "what does this program do" oracle (compile errors plus `main`'s output); a program that names `alloc`, `reserve` or `New` is only compiled, because the sandbox has no memory cap.

## Stdlib runtime tests

The stdlib modules' tests run under [`jacket test`](jacket-test.md): every folder under `[test] paths` in `jacket.toml`. CI runs them natively, interpreted, at `-O2` and, on Linux x64, as WebAssembly (`tools/ci/stdlib_jacket_test.sh`), and under the sanitizers (`tools/ci/stdlib_jacket_sanitize.sh`). Tests that open windows or use the clipboard run with `--needs display` (Linux gets a virtual display), and the data files some tests read are written first (`python3 tools/portable/build_portable.py data`). `tools/portable/check_portable_stdlib.py --check-target OS/CPU` type-checks the modules' golden tests that apply to another target without running them.

`tools/stdlib/stdlib_runtime.py` is the Python runner that is left. It runs the compiler regressions in `tests/compiler/regressions/` (and any `stdlib/**/tests/` folder that is not under `[test] paths`; there is none now) in the modes `interp`, `native`, `wasm-interp` and `wasm-native`, plus `check` for a cross-target source check. Every top-level `*.jai` file of the folder is a case that must build and exit 0 in each mode. The folder's `manifest.json` refines cases (expected output, required platform and CPU, unsupported modes) and can add cases that live in sub-directories; `tools/stdlib/stdlib_test_catalog.py` documents the row keys. Files in sub-directories (`support/`, `fixtures/`, `native/`) are helpers and never discovered on their own. A row with `"interpreter_only": true` marks a metaprogram that does all its work at compile time (it creates workspaces of its own): the `interp` mode, the interpreted sweep and the browser run include it, the native, wasm, differential and cross-build runs do not. Every regression that calls `compiler_create_workspace` or `compiler_begin_intercept` carries it (`test_stdlib_runtime.py` checks). The regressions stay here because the sweep, the differential tester, the Windows cross build and the browser runner share their manifest; moving them to `jacket test` means giving each of those a new source first.

```sh
PYTHONPATH=tools python3 tools/native/build_native_libs.py
PYTHONPATH=tools python3 tools/stdlib/stdlib_runtime.py --jaic target/release/jaic --modes interp,native,wasm-interp,wasm-native
PYTHONPATH=tools python3 tools/stdlib/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib --jailint target/debug/jailint --jaifmt target/jaifmt
```

`check_stdlib_suites.py` runs every module driver with native setup, child processes or target matrices, continues after a failure and exits nonzero if any failed. `--suite NAME` (repeatable) and `--exclude NAME` choose suites; `--jobs N` runs up to N independent groups at once (`GROUPS` in the script: suites that share scratch directories, the X display or the audio device stay serial inside one group; output of a concurrent group is printed when it ends). Put a new suite in the group whose scratch directories it shares, or in its own. `stdlib_runtime.py` takes `--filter`, `--modes`, `--opt O0|O2` and `--report FILE`. Native-only cases are reported as unavailable under WASM, never as passed. Positive cases compare exact stdout (and stderr when a contract is given); check-fail cases need a nonzero status and a `diagnostic_contains`, and a `positive_baseline` case in the same folder must still check clean. Tests can read the mode to avoid what a mode cannot do. Export counts are not a completeness measure. `--tooling-part K/N` hands `--part K/N` to the `tooling` suite alone, so CI can run its units on N jobs (see [tools layout](tools-layout.md)).

Put a module's tests in its `tests/` directory as `jacket test` goldens or `@test`s, keep compiler and ABI regressions in `tests/compiler/regressions/`, and put specialised cases in their module driver so native preparation and O0/O2 coverage stay required. Linux needs native headers, Xvfb/Mesa, fonts and audio backends; Windows needs MSVC/LLVM and Mesa.

## Stdlib target check

Type-checks every stdlib module for every target, including code nothing calls. [Dead-code elimination](../language/dead-code-elimination.md) only checks reachable module code, so a type error in `#if OS == .MACOS` code or an uncalled procedure would otherwise hide until a user on that platform hits it.

The test discovers `stdlib/X.jai` files and public `module.jai` facades (`stdlib/internal` is checked through facades to preserve import privacy), writes `#import "X"; main :: () {}` and runs `jaic -check -no_dce -os <os> -cpu <cpu>` for linux, macos and windows on x64 and arm64, and for wasm. The first error of each failure is reduced to `file: message` (no path, line or column) and the set must equal `tests/compiler/stdlib-targets.txt` exactly: an unlisted failure, a changed first error, and a listed module that now passes all fail the test. Listed failures are intentional: modules for other platforms that stop with `#assert`, procedures that stop the build on unsupported targets, and bindings with no wasm build. Checking stops at the first error, so those modules are only checked up to the `#assert`.

```sh
cargo test -p jaic-cli --test stdlib_targets    # --release for speed, --no-default-features without LLVM
```

`JACKET_STDLIB_TARGETS_PART=K/N` checks only every N-th module and target pair from the K-th (CI splits it this way) and compares only those pairs with the list; a listed pair that passes is still reported by the shard that checked it.

Change it by editing `tests/compiler/stdlib-targets.txt` only after confirming the restriction is real (`skip Module`, `Module target: file: first error`; `#` comments). A new platform-only module should stop on the wrong target with an explanatory `#assert` and get one line per refused target. A new target extends `TARGETS` in the test. Source checks do not prove native SDK or runtime availability.

## Objective-C selector check

Selectors are strings (`sel_registerName("setTitle:")`), so a typo or missing `:` type-checks and then fails at run time with "unrecognized selector". The test reads literal `sel_registerName` calls (and explicit `#foreign` aliases of that symbol) in the stdlib:

1. **Arity, every host.** The arguments after a selector in a call must equal its number of `:`; a selector-table field must be the selector with `:` written `_`. An empty scan fails.
2. **Existence, macOS only.** The receiver gives the class (`objc_getClass("X")` is a class method; `self` is an instance method of the procedure's `self` type, with `Lightweight*` views mapped as `Objective_C.class` does). A Jai program run under `jaic -interp` asks the Objective-C runtime (`class_getInstanceMethod`, `class_getClassMethod`, protocol method descriptions, declared properties, class-cluster subclasses) and its output must match `tests/compiler/objc-selectors.txt`. Elsewhere it only has to type-check for macOS.

Calls on local objects get the arity check but not the existence check, and selectors stored in variables are not checked at all, so pass selectors as literals. When it reports a missing method, fix the binding; if the method only exists on newer macOS than CI or the class is private, list it in `tests/compiler/objc-selectors.txt` (`missing <class> <instance|class> <selector>`, `unknown <class>`; `#` reason). Remove a listed line once it is found. Run with `cargo test -p jaic-cli --test objc_selectors`.

The optimized (`-O2`) sanitizer run of the stdlib modules (`tools/ci/stdlib_jacket_sanitize.sh O2`) covers only the modules in `tests/compiler/optimized-modules.txt` (code generation that differs from -O0); the sweep's `corpus` and `regressions` run whole.
