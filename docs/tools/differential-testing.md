# Differential testing

## What it is

There is no independent implementation to compare against, so jaic is checked against itself. One program can run through four backends that share a front end but little else after lowering. `tools/jaic-diff.py` runs each program through every backend and requires the same output. `tools/jaigen.py` generates random, well-defined programs to feed it, and `tools/jaic-reduce.py` shrinks a program the backends disagree on.

| Backend | What runs |
| --- | --- |
| `interp` | `jaic run`: the IR interpreter on the host |
| `native` | `jaic build -O0`, then the executable (LLVM) |
| `native-O2` | the same at `-O2`, so LLVM's optimizer is involved |
| `wasm` | the browser engine (`crates/jai-wasm`, driven by node through `tools/jaic_diff_wasm.mjs`): the interpreter compiled to wasm32, target OS `.WASM`, `SandboxHost` libc |
| `wasm-native` | `jaic build -os wasm` (LLVM, wasm64, `Wasi_Runtime`), run by node's WASI through `tools/wasi_run.mjs` ([wasm target](../native/wasm-target.md)). Only when named in `--backends`: it needs `wasm-ld` and node 24 |

`interp` and `wasm` share the interpreter, but they differ in target (32-bit `.WASM` against the host), constant folding host, file system and libc. `native` and `native-O2` share only the IR with the interpreter.

## How it works

```sh
python3 tools/jaic-diff.py --jaic target/release/jaic --wasm <bundle> corpus stdlib modules gen:1:1000
```

Sets:

- `corpus`: the `tests/corpus` manifest cases with a `runtime` expectation.
- `stdlib`: `tests/stdlib/*.jai`.
- `modules`: `stdlib/*/tests` and `stdlib/tests`, with the whole `stdlib/` as the workspace, because tests `#load` their module's files.
- `gen:SEED:COUNT`: COUNT jaigen programs starting at SEED.
- A file path.

Each backend's run gives a result `(status, stdout, stderr)`:

- The status is `exit N` or `runtime error`. The backends report a failed bounds check, a panic or a signal differently, so for a runtime error only the fact is compared.
- stdout must match exactly.
- stderr must match when every backend exited normally.
- Paths are normalized first: the case root becomes `/workspace` and the stdlib becomes `/stdlib`, as the wasm sandbox names them.

Verdicts:

- `agree`: every backend that ran gave the same result.
- `DISAGREE`: they did not, including when one backend fails to compile a program another compiled.
- `invalid`: every backend failed to compile the program.
- `skip`: fewer than two backends could run the program.

The exit code is 1 on any `DISAGREE` or `invalid`. `--json FILE` writes every result, and `-v` prints them all.

### What a backend may decline (`unsupported`)

A backend is excluded from a case only for a stated reason, and as far as possible that reason comes from metadata that already exists rather than a list in the harness:

- `wasm`, any program: the engine's own refusal messages (`foreign procedure ... is not available here`, `unknown library`, `is not supported on wasm`), and a stdlib module that rejects the target with `#assert`.
- `wasm`, missing files: a file the sandbox lacks (the sandbox holds the workspace and the stdlib only). This counts only when no host backend complained about the same file.
- `native`, no `main`: the build fails with `no exported 'main'`, so the program runs only at compile time.
- `native`, no executable: the build wrote nothing at `-o` because the program's metaprogram decides what to write (its own workspaces, `NO_OUTPUT`).
- `native`, compiler primitives: the program calls a `#compiler` primitive at run time, which exists only inside the compiler. The compiled stub says `is a compiler primitive; it runs only at compile time`.

A program whose source mentions `.WASM` (as in `OS == .WASM`) may skip work in the browser on purpose. `tests/stdlib` programs skip processes, native libraries and windows this way, because every one of them must pass in the browser engine ([playground](../browser/playground.md)). For such a program the `wasm` result is compared by status only, and the host backends are still compared in full.

`OUTPUT_VARIES` in the harness lists cases where only the status is compared. Each entry has its reason:

- `jaic-extensions-long-double` prints which `Long_Double` representation the target ABI has: float64 on arm64 macOS, binary128 on wasm32.
- `iprof-runtime-manual` prints measured times.
- `debug-assert-handlers` prints a native stack trace: addresses, and the symbols of whatever executable is running. Under `jaic run` that is the interpreter itself.

### The generator (`tools/jaigen.py`)

`tools/jaigen.py SEED [--size S]` prints one program. A seed always gives the same program. The program keeps a 64-bit FNV checksum `H`, mixes its state into it as it runs, prints intermediate values and ends with `checksum <hex>`.

It covers:

- integers of every width (`s8`..`u64`), with literals at powers of two ±1 and at float32/float64 rounding ties;
- `float32` and `float64`, including `-0.0`, subnormals and values near the largest float;
- arithmetic, bitwise operators, shifts and rotates, comparisons and `&&`/`||`;
- casts: plain, `trunc` and `no_check`, int↔float, guarded float→int;
- `#no_aoc` blocks;
- structs with defaults and nested structs, a polymorphic struct `Pair(T)`;
- `using` struct members (fields read and written through the promoted name) and `using v;` blocks ([using](../language/using.md));
- overloaded `+`, `==` (and the `!=` fallback) and `[]` on generated structs ([operator overloading](../language/operator-overloading.md));
- plain unions punning an integer, a float and bytes (little-endian on every target), and tagged unions whose active variant is read after a switch on the tag ([unions](../language/unions.md));
- fixed arrays (including arrays of structs), views, dynamic arrays with `array_add`;
- `for` over ranges (forward and reverse), over arrays by value and by pointer, bounded `while` loops, `break`/`continue`;
- `if`/`else`, `ifx`, `if x == { case ...; }` switches;
- enums, `enum_flags` and enum casts;
- `defer`, including inside loops;
- strings: literals, `tprint`, `count`, indexing and comparison;
- procedures: pure and effectful, inline, recursive with a depth argument, procedure-valued variables;
- overload sets whose members take one parameter of different types, always called with an argument of exactly one member's type, and `..T` variadics called with 0–4 arguments ([procedures](../language/procedures.md));
- `..Any` arguments and `Any` values printed with their `Type_Info` tag, `type_of`, and reflection on generated types: struct names, sizes, member names, offsets and tags, enum `names`/`values` ([type values](../language/type-values-and-info.md));
- polymorphic procedures with `$T` and `#if` on the type;
- pointers to locals;
- `#run` constants (scalar and struct) compared with the same computation at run time;
- globals;
- `print` of every type, with aggregates that hold floats printed only by count.

Programs use only behaviour that `docs/language/*.md` defines (the docstring has the rules):

- Integer `+ - *` wrap.
- Shift amounts are masked below the width.
- Division and remainder never use a divisor of 0 or -1.
- Float→int casts only see values in range.
- NaN is hashed and printed in a canonical form.
- Procedures called inside expressions are pure, because operand evaluation order is not specified.
- Loops have constant trip counts and recursion has a decreasing depth.
- A union is read through another member only for bit patterns, never for a float that may be NaN (its payload is not specified); a tagged union only through its active variant.
- Overload choice never depends on ranking conversions, and procedure values are never taken of an overloaded name.
- Sizes and offsets are printed only for generated structs, which hold no pointers, so the layout is the same on 64-bit hosts and wasm32. Floats are never printed through `Any`.

### Reducing (`tools/jaic-reduce.py`)

```sh
python3 tools/jaic-reduce.py prog.jai --jaic target/release/jaic --wasm <bundle>
```

The reducer keeps a candidate if it still compiles and the backends still split into the same groups (for example `interp+native+wasm` against `native-O2`). It deletes chunks of lines down to single lines (ddmin-style), then whole `{ }` blocks, then unwraps blocks, and repeats until nothing changes. The result is `prog.reduced.jai`. The timeout defaults to 10 s, because deleting a loop counter's update makes a loop spin forever. A 250-line program typically shrinks to about 50 lines in a few hundred runs.

### Full run

Run this before a release, or after changing the interpreter, sema's constant folding, lowering or the LLVM backend. A generated program costs a few tenths of a second on all four backends with three jobs, so a few thousand take a quarter of an hour or more. Run it with one heavy job at a time, since it builds two native executables per program. Pick seeds no earlier campaign used (the commit that added a generator feature names the ranges it ran):

```sh
cargo build --release -p jaic-cli
python3 tools/build_scripting_wasm.py --release --output /tmp/wasm
python3 tools/jaic-diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 corpus stdlib modules
python3 tools/jaic-diff.py --jaic target/release/jaic --wasm /tmp/wasm --jobs 3 --keep /tmp/gen gen:70000:3000
```

`--keep` keeps the generated programs (`/tmp/gen/programs/gen-N/main.jai`) for reducing. Reproduce one program with `tools/jaigen.py N`.

### Bugs found

Each bug was fixed with a regression test.

| Program | Wrong backend | Bug |
| --- | --- | --- |
| `tests/stdlib/bindings-generator-cpp-classes.jai` | native | `#cpp_return_type_is_non_pod` was ignored, so a one-`int` C++ class came back in registers while the callee wrote through the hidden pointer (`c-abi.md`) |
| `tests/stdlib/compiler-workspace-ids.jai` | native | the build tried to link workspaces that were created but given no source (`workspaces.md`) |
| stdlib programs calling Bindings_Generator at run time | native (diagnostic) | compiled `#compiler` stubs trapped without a word; they now say why |
| generated (`tests/stdlib/poly-infer-from-ifx.jai`) | front end | `$T` could not be inferred from an `ifx` argument |
| targeted probe (`tests/stdlib/int-to-float32-rounding.jai`) | interp, wasm, constant folding | 64-bit integer → `float32` rounded twice, through f64 |
| `tests/stdlib/debug-assert-handlers.jai` | native | functions had no frame records, so macOS `backtrace` (Debug's `backtrace`) found no frames ([LLVM backend](../native/llvm-backend.md)) |
| generated seed 10014 (`tests/corpus/positive/unrolled-sub-reduction.jai`) | native-O2 | LLVM 22's runtime unroller recombined parallel accumulators of an `a -= b` recurrence wrongly (llvm/llvm-project#201065). jaic turned that transformation off until LLVM 23.1.0 fixed it ([LLVM backend](../native/llvm-backend.md)) |
| generated seed 61919 (`tests/stdlib/poly-infer-from-ifx.jai`) | front end | `$T` could not be inferred from an `ifx` whose branches are all `ifx` themselves |
| generated seeds 62030, 62063 (`tests/stdlib/cast-float-of-integer-operator.jai`) | front end | a float cast's target reached an `ifx` operand of `&`/`%`, which widened to float, so the operator failed on `float32` |

The generated programs found 4 of these bugs. Four came from the stdlib sets, and one from a targeted probe of int-to-float conversions (jaigen now generates those edge values too). Run counts, seed ranges and timings belong in the commit messages and reports of each campaign, not here: a generator change gives every seed a different program. One generated program failed to compile everywhere; see Settled questions.

### Settled questions

Differential testing cannot find a bug that every backend shares, because they share a front end. The generator ran into one such front-end behaviour, now settled in `docs/language`:

- **The question.** Should a cast's target type reach an untyped literal on the left of an operator whose other operand is typed?
- **The answer.** No. Types flow up; the literal is matched with the other operand ([numbers](../language/numbers.md), [casts](../language/casts-and-conversions.md), rules `num.16` and `cast.29`). `cast(float32) (0 - w)` with `w: u16 = 15` subtracts in `u16` and gives `65521`. jaic used to compute it in `float32` and give `-15`, which also made `cast(float32) ((0 - w) & v)` fail with `operator BitAnd is not defined for float32` (seed 22650).
- **jaigen** generates `(0 - x)` again; for a while it wrote `cast(T) 0 - x` to avoid the question.

### CI

`.github/workflows/ci.yml` runs a cheap subset ([continuous integration](continuous-integration.md)):

- `test` job, every OS: `--backends interp,native,native-O2 corpus gen:1:40` with the debug `jaic`.
- `scripting-wasm` job: `--backends interp,wasm corpus gen:1:40`, with an LLVM-free host `jaic` (`--no-default-features`) and the bundle the job already builds.
- `test` job, every OS: `--backends interp,wasm-native corpus gen:1:40` in the WebAssembly target step.

## How to change it

- **New backend:** add it to `ALL_BACKENDS` and to `Runner.run`, and map its failures onto the statuses above. Paths it prints need adding to `Runner.normalize`.
- **New set:** extend `cases()`. Give a `Case` a `root` when the program reads sibling files, because the wasm sandbox sees only that directory. Use `skip={backend: reason}` for a principled exclusion that metadata provides.
- **Declining a program:** prefer a message the backend itself prints, or existing metadata, over a list of names. If an entry in `OUTPUT_VARIES` is unavoidable, give it a reason that someone can check.
- **Generator:** `Gen` in `tools/jaigen.py` has `expr_int`/`expr_float`/`expr_bool` per type, `stmt` for statements and `program` for the top level. A new form must stay defined behaviour for every backend. Check validity with `gen:` sets using `--backends interp` (every program should `agree` with itself or at least compile). Never generate pointer arithmetic, uninitialized reads (`---`), or an expression whose result depends on evaluation order.
- **Overflow checks:** `.FATAL`/`.NONFATAL` arithmetic checks apply only to workspaces a metaprogram creates, so the generator does not cover them.
- **A found case:** reduce it and fix the wrong backend. Then keep the reduced program as a `tests/corpus/positive` case: add a manifest entry with `sha256` and the interpreter's `runtime` output. If only one backend or optimization level was wrong, also add a targeted test (for example in `crates/jaic-cli/tests/native.rs`).

## Configuration

| Flag | Default | Meaning |
| --- | --- | --- |
| `--jaic` | `$CARGO_TARGET_DIR/debug/jaic` | compiler under test. It must be newer than `crates/` (`--allow-stale` overrides this) |
| `--wasm DIR` | none | bundle from `tools/build_scripting_wasm.py --output DIR`. Without it the `wasm` backend is off |
| `--backends` | all available | comma-separated subset |
| `--jobs` | 3 | cases in parallel. Each native case builds two executables |
| `--timeout` / `--memory-limit` | 120 s / 3 GiB | per process, as in the [sweep](jaic-sweep.md) |
| `--gen-size` | 1.0 | scales jaigen's procedure and statement counts |
| `--keep DIR` | temp dir | keep generated programs and executables |

The stdlib and modules sets build the [third-party native libraries](native-libs.md) first, as the sweep does.

## Dependencies

- Python 3.9+.
- `tools/jaic-sweep.py`, which provides the process limits and the stale-binary check.
- node, for the wasm backends (node 24 for `wasm-native`), and `wasm-ld` for `wasm-native`.
- The LLVM 23 toolchain behind `jaic build`.
- The browser bundle built by `tools/build_scripting_wasm.py`.
- Unit tests: `tools/test_jaic_diff.py`, which needs no compiler.
