# Stdlib fuzzing

## What it is

A type-driven fuzzer for stdlib procedures. It reads a module's exported procedure signatures, generates arguments from the parameter types, calls each procedure many times with edge-biased values and reports crashes, failed asserts, bounds/cast/overflow check failures, hangs and violated properties. It complements the cargo-fuzz targets in `fuzz/` (which fuzz the compiler itself, see [testing](testing.md)); it finds bugs in *Jai code*, not in `jaic`.

`#import "Extensions/Fuzz";` loads the jaic-specific fuzzing helpers. The former root import `Fuzz` is no longer available.

Two modes share the generators: a seeded mode under `jaic -interp` (fast to start, deterministic, no native toolchain) and a coverage-guided mode (`--engine`) that builds the same harness natively with `-sanitize address,fuzzer` and lets libFuzzer mutate the input bytes the generators consume.

```sh
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String Unicode Base64 Hash Crc --jaic target/release/jaic --count 2000
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String --engine --time 300                           # coverage-guided
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String --only slice --seed 2 --count 1 --max-len 0   # reproduce
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py --replay                                              # stored cases
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

**Generating values.** `stdlib/Extensions/Fuzz/values.jai`. `gen(r, T)` fills a `T` from `type_info(T)` with `fill`, one arm per `Type_Info` kind:

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

**Input bytes.** `Rng` has two sources. Seeded (default): SplitMix64 from the case seed. Input (`from_input`): every choice reads bytes from the engine's input, like a `FuzzedDataProvider`: `take(r, n)` reads n bytes (zeros past the end, which select the first alternative: empty, zero, first member), and `below(r, n)` reads only as many bytes as `n` needs, so one-byte choices stay one byte and libFuzzer's byte mutations map onto generator decisions. The generators do not know which source they use. The first input byte picks the case.

**Calling.** For each procedure the metaprogram emits a thunk (`thunk_N :: (r: *Rng) { a0 := gen(r, string); ...; Target.proc(a0, ...); }`) and a `Case` table; `run_cases` (`stdlib/Extensions/Fuzz/runner.jai`) calls them. The argument types are spelled from the type info (`type_text` in `discover.jai`); struct and enum types are usable only when the module exports them under their own name. Where the type info cannot show the difference (variadic `..T` looks like `[] T`; a `$constant` parameter is not flagged polymorphic), the harness fails to compile; the driver reads the error line, denies that procedure and rebuilds, and lists it under `skip`.

**Oracles.**

- *Crash:* any runtime error of the interpreter (failed `assert`, array-bounds, null-pointer, cast and arithmetic overflow checks), a signal (`exit -11`), or a timeout (`--timeout`, default 60 s per run).
- *Property hooks:* `tools/fuzz/properties/<Module>.jai` defines `fuzz_*` procedures `(r: *Rng)` that generate inputs themselves and `assert` a property (`base64_decode(base64_encode(x)) == x`, UTF-8 encode/decode round trip, `sdbm_hash` chaining, `split` then `join`). The harness runs them as extra cases. They are also the way to call things generation cannot, such as pointer-and-length pairs (`sdbm_hash(*void, size)`), variadics and overloads.

**Driving inputs.** The harness prints `CASE <proc> <seed> <max_len>` before every call; the last such line before a death names the culprit. Cases are `(proc, seed)`: `rng_seed` makes generation deterministic, so a case reproduces with `--only <proc> --seed <seed> --count 1 --max-len <n>`. The driver runs procedure by procedure and restarts after a crash at the next seed (at most 40 restarts per procedure, one report per distinct signature). *Minimizing* retries the failing seed with length bounds 0, 1, 2, 4, ... 128 and keeps the smallest that fails with the same signature. New findings are written to `fuzz/regressions/stdlib/<proc>.<seed>.json` (`--write-regressions`); `--replay` re-runs them all and prints `fixed` or `STILL FAILS`. Commit the file with the fix.

**Exclusions.** `tools/fuzz/targets.json`, per module:

| Key | Meaning |
|---|---|
| `deny` / `allow` | `deny`: `{ "proc": "why it is skipped" }`, procedures to skip (every entry needs its reason; a unit test checks); `allow`: a list of procedures to fuzz exclusively |
| `pointers` | procedures that may receive pointers (default: a procedure with a pointer parameter is skipped) |
| `nullable` | procedures that also accept null pointers |
| `arity` | `{ "proc": N }` passes only the first N arguments (the rest default), for parameters that take a validated table or a compile-time constant |
| `hooks` | property file, relative to `tools/fuzz/` |
| `known` | `{ "proc": ["text of a signature"] }`: expected precondition failures, reported as `known` and not as new findings |

A module path with a slash (`Extensions/Postgres`) names its scratch files with dashes (`harness-Extensions-Postgres.jai`).

Only add modules whose procedures are pure computation over their arguments. Skip anything with side effects: files, network, processes, allocator or `context` mutation, global state (random seeding, `set_*`, `init_*`). The module's exports come from the names declared in its `module.jai` (`probe_<name> :: Target.<name>;` makes the compiler typecheck each). `Fuzz` never inspects bodies, so this is a review decision per module, recorded as `deny` or by leaving the module out of `targets.json`.

## Postgres client

`Extensions/Postgres` has no generated cases: its exports need a connection. `properties/Postgres.jai` builds one whose `Io.Reader` serves the case's bytes as the server's reply (in pieces of 1 to 8 bytes, or 16 KiB) and whose `Io.Writer` keeps what the client sends, so the real message reader, decoders and COPY code run without a server. Every call runs inside `Io.io_run`. The hooks:

| Hook | Property |
|---|---|
| `fuzz_pg_reply_random_bytes` | any bytes as the reply to `query_dynamic` / `query_binary`: a result or an error |
| `fuzz_pg_reply_cut_short` | a well-formed reply cut anywhere: an error, never a hang |
| `fuzz_pg_reply_mutated` | a reply with bytes changed, inserted and dropped, and notices, notifications and parameter changes mixed in |
| `fuzz_pg_decode_value` | one value of any bytes as a column of any type, text or binary, read with every type `get` takes |
| `fuzz_pg_round_trip_scalars` / `_arrays` | a value sent as a parameter, answered with the bytes the client sent (same type OID and format), reads back as itself |
| `fuzz_pg_copy_out_random` / `_fields` | binary COPY data into a struct: random bytes, then a valid header with random fields |
| `fuzz_pg_copy_round_trip` | rows written with `copy_in` come back the same through `copy_out` |

The fiber has temporary storage of its own, freed when it ends: a hook that hands values back copies them into the caller's with `in_caller_temp`.

## Coverage-guided mode

`--engine` builds `engine-<Module>` with `jaic -O0 -sanitize address,fuzzer` ([sanitizers](../native/sanitizers.md)): SanitizerCoverage edge counters, a PC table and traced comparisons, ASan, and libFuzzer's engine without its `main`. The harness `main` (`ENGINE_BUILD :: true`) calls `run_engine`, which passes libFuzzer its arguments and `engine_input` as the callback through `LLVMFuzzerRunDriver`; `engine_input` is `#c_call`, so it enters a fresh context, points the `Rng` at the input and runs one case. `case_end` frees the case's allocations and resets temporary storage, in both modes. A Jai runtime error ends the process, which libFuzzer reports as a crash.

- Corpus: `fuzz/corpus/stdlib/<Module>/`; crashes, timeouts and OOMs: `fuzz/artifacts/stdlib/<Module>/` (both gitignored). The run uses `-fork=1 -ignore_crashes=1`, so it continues after a crash and the driver reproduces each artifact with `--trace` (prints the case name per input) to attribute and classify it. `--only NAME` fuzzes one procedure; `--time`, `--engine-max-len` (input size bound), `--opt` and `--timeout` tune the run.
- A finding is the crash file; replay it with `engine-<Module> --trace --only <case> <file>`. Copy it into `fuzz/regressions/stdlib/` with the fix. The report line shows the file, the case and the signature (a Jai runtime error, an ASan report with its top frame, a timeout, OOM).
- Checked: with the old `find_index_of_any_from_left` the engine finds the out-of-bounds read in seconds (`AddressSanitizer: SEGV in find_index_of_any_from_left bytes.jai:81`).
- Needs the Homebrew `llvm` (or `libclang-rt` on Linux), found the way the ASan runtime is; see [sanitizers](../native/sanitizers.md). Memory: ASan reserves a lot of virtual space, so run one module at a time on small machines. A module that links native libraries (Postgres links mbedtls) needs `JAIC_NATIVE_LIBS` somewhere the sandbox can read, such as a copy in the scratch folder.

## Sandbox

The driver wraps every `jaic` run with `tools/bench/sandbox.py` (Seatbelt on macOS, bubblewrap on Linux): no network, reads limited to the repository and the compiler's folder, writes limited to the scratch folder (`--scratch`, default a fresh temporary one). `--no-sandbox` skips that, which is required when the driver itself already runs inside another sandbox (macOS cannot nest Seatbelt profiles). Generated pointers point into allocations the harness owns; the interpreter's own sandbox keeps file and process access closed.

## How to change it

- *New module:* add an entry to `targets.json`, run it, and sort the first findings into fix, `known` or `deny`. Prefer a property hook over a `known` entry when the surprise is a contract rather than a bug.
- *New value kind or distribution:* `stdlib/Extensions/Fuzz/values.jai`; keep `can_fill` and `has_pointer` in step with `fill`. `stdlib/Extensions/Fuzz/tests/generators.jai` checks determinism, bounds and edge coverage.
- *New oracle:* add to `signature_of` / the property hooks. Output or `stderr` checks belong in the driver, value checks in a hook.
- Fuzz hooks and the harness are plain Jai; run one by hand with `jaic -interp <scratch>/harness-<Module>.jai -import_dir stdlib -- --only <proc> --seed N --count 1`. Harness options: `--seed`, `--count` (seeds per case), `--only`, `--max-len`, `--list`.

Gotchas: casts check overflow, so generator code must use `cast,trunc`; `#no_abc` procedures read out of bounds without an error and typically die as a signal (`exit -11`/`-10`) rather than a diagnostic. A crash while the argument was legitimately invalid is a precondition, not a bug: decide whether the procedure should check (fix it), or document it (`known`).

## Findings so far

- `find_index_from_left` / `find_index_of_any_from_left` (`#no_abc`) read before the string for a negative `start_index` (SIGSEGV/SIGBUS under `jaic -interp`). Fixed by clamping to 0; stored as `fuzz/regressions/stdlib/String.find_index_of_any_from_left.{4,12}.json`, tested in `stdlib/String/tests/basics.jai`.
- Postgres: a binary array value whose header claims more elements than its bytes hold (up to 2^31 - 1) made the client allocate them all before failing, gigabytes per value. It now checks the count against the bytes first; tested in `tests/fake_server.jai`.
- `slice` asserts `index >= 0 && count >= 0`, which its doc comment now states. The generic cases deny it and `fuzz_string_slice` draws non-negative arguments instead, so `String` has no `known` entries.

## Later phases

1. **Richer shapes.** Pointer-and-length pairs from the type (a `*T` followed by an integer length), `Code`-free procedure arguments (generated callbacks), struct and enum spelling for non-exported types, resizable arrays, and `context` mutation in hooks.
2. **Value minimization.** (Engine crashes can be shrunk with libFuzzer's `-minimize_crash=1`; value-level shrinking is the missing part.) Shrinking the generated arguments themselves (shorter strings, zeroed fields) rather than only the length bound; printing a failing call as a ready-to-paste Jai test.
3. **CI and reports.** A coverage report tool reading the PC table and counters (`__sancov_pcs`, `__sancov_cntrs`) is planned on the same instrumentation. A nightly job next to `fuzz.yml` running `--engine` per module and failing on new findings.

## Dependencies

`jaic` (metaprogram message loop, `Type_Info_Procedure`, `-sanitize fuzzer`), LLVM compiler-rt's libFuzzer archive, `stdlib/Extensions/Fuzz`, `Compiler` and `Basic`, `tools/bench/sandbox.py`, Python 3 standard library.
