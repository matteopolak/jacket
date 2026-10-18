# Stdlib fuzzing

## What it is

A type-driven fuzzer for stdlib procedures. It reads a module's exported procedure signatures, generates arguments from the parameter types, calls each procedure many times with edge-biased values and reports crashes, failed asserts, bounds/cast/overflow check failures, hangs and violated properties. It complements the cargo-fuzz targets in `fuzz/` (which fuzz the compiler itself, see [testing](testing.md)); it finds bugs in *Jai code*, not in `jaic`.

```sh
python3 tools/fuzz/stdlib_fuzz.py String Unicode Base64 Hash Crc --jaic target/release/jaic --count 2000
python3 tools/fuzz/stdlib_fuzz.py String --only slice --seed 2 --count 1 --max-len 0   # reproduce
python3 tools/fuzz/stdlib_fuzz.py --replay                                              # stored cases
```

Exit status is 1 when a finding is not listed as `known` for its module.

## How it works

```
targets.json ─┐
discover.jai  ├─ jaic -check ──► SKIP lines + harness.jai ── jaic -interp ──► CASE/DONE lines ──► findings
Fuzz module  ─┘   (metaprogram)                (generated)                     (stdout)              │
                                                                                     minimize ◄────────┘
```

**Discovering targets.** `tools/fuzz/discover.jai` is a metaprogram. It opens a child workspace that imports the module, intercepts compiler messages with `compiler_begin_intercept` and `compiler_wait_for_message`, and reads the `TYPECHECKED` declarations that live in the module's own `module.jai`. A constant declaration whose `type` is a `Type_Info_Procedure` is an exported procedure; its `argument_types` drive everything. Polymorphic, foreign and compile-time-only procedures and overload sets are skipped (`SKIP <name> <reason>`). The driver passes the module's config as constants (`MODULE`, `DENY`, `ALLOW`, `POINTERS`, `NULLABLE`, `ARITY`, `HOOKS`) in a generated wrapper that `#load`s `discover.jai`.

**Generating values.** `stdlib/Fuzz/values.jai`. `gen(r, T)` fills a `T` from `type_info(T)` with `fill`, one arm per `Type_Info` kind:

| Kind | Values |
|---|---|
| integer | 0, 1, -1/all ones, signed min and max, small positive and negative, powers of two and their predecessors, 0x7F, 0x80, uniformly random |
| float | ±0, ±1, NaN, ±inf, smallest subnormal, smallest normal, max, lowest, random bits |
| bool | 0 or 1 |
| string | empty; printable ASCII; well-formed UTF-8 over all planes; arbitrary bytes; malformed UTF-8 (stray continuation, overlong, surrogate, above U+10FFFF, truncated, bad lead bytes); separators, whitespace, NUL; base64/number alphabets; one repeated byte. Lengths: 0, 1, a few, anything up to the bound, exactly the bound |
| enum | a declared value (3 in 4) or an undeclared one: a neighbour of a member or arbitrary bits |
| view / fixed array | recursively filled elements; views are empty (null or non-null data), short or up to the bound |
| struct | each non-constant member in place at its offset |
| pointer | null unless the target is in `POINTERS`; then a fresh allocation (a buffer of `max_len` bytes for byte-like and `void` pointees), plus null only if also in `NULLABLE` |

Allocations live in `Rng.live` and are freed by `case_end` after each call. Not generated: resizable arrays, `Any`, `Type`, `Code`, procedure values; a procedure taking one is skipped.

**Calling.** For each procedure the metaprogram emits a thunk (`thunk_N :: (r: *Rng) { a0 := gen(r, string); ...; Target.proc(a0, ...); }`) and a `Case` table; `run_cases` (`stdlib/Fuzz/runner.jai`) calls them. The argument types are spelled from the type info (`type_text` in `discover.jai`); struct and enum types are usable only when the module exports them under their own name. Where the type info cannot show the difference (variadic `..T` looks like `[] T`; a `$constant` parameter is not flagged polymorphic), the harness fails to compile; the driver reads the error line, denies that procedure and rebuilds, and lists it under `skip`.

**Oracles.**

- *Crash:* any runtime error of the interpreter (failed `assert`, array-bounds, null-pointer, cast and arithmetic overflow checks), a signal (`exit -11`), or a timeout (`--timeout`, default 60 s per run).
- *Property hooks:* `tools/fuzz/properties/<Module>.jai` defines `fuzz_*` procedures `(r: *Rng)` that generate inputs themselves and `assert` a property (`base64_decode(base64_encode(x)) == x`, UTF-8 encode/decode round trip, `sdbm_hash` chaining, `split` then `join`). The harness runs them as extra cases. They are also the way to call things generation cannot, such as pointer-and-length pairs (`sdbm_hash(*void, size)`), variadics and overloads.

**Driving inputs.** The harness prints `CASE <proc> <seed> <max_len>` before every call; the last such line before a death names the culprit. Cases are `(proc, seed)`: `rng_seed` makes generation deterministic, so a case reproduces with `--only <proc> --seed <seed> --count 1 --max-len <n>`. The driver runs procedure by procedure and restarts after a crash at the next seed (at most 40 restarts per procedure, one report per distinct signature). *Minimizing* retries the failing seed with length bounds 0, 1, 2, 4, ... 128 and keeps the smallest that fails with the same signature. New findings are written to `fuzz/regressions/stdlib/<proc>.<seed>.json` (`--write-regressions`); `--replay` re-runs them all and prints `fixed` or `STILL FAILS`. Commit the file with the fix.

**Exclusions.** `tools/fuzz/targets.json`, per module:

| Key | Meaning |
|---|---|
| `deny` / `allow` | procedure names to skip / to fuzz exclusively |
| `pointers` | procedures that may receive pointers (default: a procedure with a pointer parameter is skipped) |
| `nullable` | procedures that also accept null pointers |
| `arity` | `{ "proc": N }` passes only the first N arguments (the rest default), for parameters that take a validated table or a compile-time constant |
| `hooks` | property file, relative to `tools/fuzz/` |
| `known` | `{ "proc": ["text of a signature"] }`: expected precondition failures, reported as `known` and not as new findings |

Only add modules whose procedures are pure computation over their arguments. Skip anything with side effects: files, network, processes, allocator or `context` mutation, global state (random seeding, `set_*`, `init_*`). `Fuzz` never inspects bodies, so this is a review decision per module, recorded as `deny` or by leaving the module out of `targets.json`.

## Sandbox

The driver wraps every `jaic` run with `tools/bench/sandbox.py` (Seatbelt on macOS, bubblewrap on Linux): no network, reads limited to the repository and the compiler's folder, writes limited to the scratch folder (`--scratch`, default a fresh temporary one). `--no-sandbox` skips that, which is required when the driver itself already runs inside another sandbox (macOS cannot nest Seatbelt profiles). Generated pointers point into allocations the harness owns; the interpreter's own sandbox keeps file and process access closed.

## How to change it

- *New module:* add an entry to `targets.json`, run it, and sort the first findings into fix, `known` or `deny`. Prefer a property hook over a `known` entry when the surprise is a contract rather than a bug.
- *New value kind or distribution:* `stdlib/Fuzz/values.jai`; keep `can_fill` and `has_pointer` in step with `fill`. `stdlib/Fuzz/tests/generators.jai` checks determinism, bounds and edge coverage.
- *New oracle:* add to `signature_of` / the property hooks. Output or `stderr` checks belong in the driver, value checks in a hook.
- Fuzz hooks and the harness are plain Jai; run one by hand with `jaic -interp <scratch>/harness-<Module>.jai -import_dir stdlib -- --only <proc> --seed N --count 1`. Harness options: `--seed`, `--count` (seeds per case), `--only`, `--max-len`, `--list`.

Gotchas: casts check overflow, so generator code must use `cast,trunc`; `#no_abc` procedures read out of bounds without an error and typically die as a signal (`exit -11`/`-10`) rather than a diagnostic. A crash while the argument was legitimately invalid is a precondition, not a bug: decide whether the procedure should check (fix it), or document it (`known`).

## Findings so far

- `find_index_from_left` / `find_index_of_any_from_left` (`#no_abc`) read before the string for a negative `start_index` (SIGSEGV/SIGBUS under `jaic -interp`). Fixed by clamping to 0; stored as `fuzz/regressions/stdlib/String.find_index_of_any_from_left.{4,12}.json`, tested in `stdlib/String/tests/basics.jai`.
- `slice` asserts `index >= 0 && count >= 0`: a documented precondition, listed as `known`.

## Later phases (not in v1)

1. **Coverage guidance.** Have `jaic` emit LLVM SanitizerCoverage instrumentation (`-fsanitize-coverage=trace-pc-guard,inline-8bit-counters`, plus `trace-cmp` for comparison feedback) on a native build of the harness and link libFuzzer. The harness gets an `LLVMFuzzerTestOneInput` entry that treats the input bytes as the `Rng` stream (a byte reader replaces `next`), so libFuzzer's mutation, corpus and minimization work on generator decisions instead of seeds. Needs a `-sanitize_coverage` build option in `jaic-llvm` and a driver mode that builds natively (`-O0`, checks on) and runs under `fork` mode like `fuzz/run.sh`. Corpus and crash artifacts would live in `fuzz/corpus/stdlib/<Module>` and `fuzz/artifacts/stdlib/`.
2. **Native runs.** `jaic -O0` executables are far faster than the interpreter and catch what only native code shows (allocator behaviour, UB under sanitizers, see [sanitizers](../native/sanitizers.md)); first step before coverage.
3. **Richer shapes.** Pointer-and-length pairs from the type (a `*T` followed by an integer length), `Code`-free procedure arguments (generated callbacks), struct and enum spelling for non-exported types, resizable arrays, and `context` mutation in hooks.
4. **Value minimization.** Shrinking the generated arguments themselves (shorter strings, zeroed fields) rather than only the length bound; printing a failing call as a ready-to-paste Jai test.
5. **CI.** Nightly job next to `fuzz.yml` running the driver with a fresh seed range and failing on new findings.

## Dependencies

`jaic` (metaprogram message loop, `Type_Info_Procedure`), `stdlib/Fuzz`, `Compiler` and `Basic`, `tools/bench/sandbox.py`, Python 3 standard library.
