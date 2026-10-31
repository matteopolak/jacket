# Stdlib fuzzing

A type-driven fuzzer for stdlib procedures. It reads a module's exported procedure signatures, generates arguments from the parameter types, calls each procedure many times with edge-biased values, and reports crashes, failed asserts, check failures, hangs and violated properties. It finds bugs in Jai code, not in `jaic` (for that, see the cargo-fuzz targets in [testing](testing.md)).

Two modes share the generators: a seeded mode under `jaic -interp` (deterministic, no native toolchain) and a coverage-guided mode (`--engine`) that builds the harness natively with `-sanitize address,fuzzer`.

```sh
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String Unicode Base64 --jaic target/release/jaic --count 2000
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String --engine --time 300
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py String --only slice --seed 2 --count 1 --max-len 0   # reproduce
PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py --replay                                              # stored cases
```

Exit status is 1 when a finding is not listed as `known` for its module.

## How it works

- **Discovery.** `tools/fuzz/discover.jai` is a metaprogram that imports the module in a child workspace and reads the type of each declaration in its `module.jai`. Every constant of procedure type is a target. Polymorphic, foreign and compile-time-only procedures and overload sets are skipped.
- **Generation.** `stdlib/Extensions/Fuzz/values.jai` fills each argument from its `Type_Info`: integers and floats at their edges, strings (empty, ASCII, valid and malformed UTF-8, separators), enums (declared and undeclared values), views, arrays, structs, and pointers only for procedures in `pointers`. Resizable arrays, `Any`, `Type`, `Code` and procedure values are not generated; a procedure taking one is skipped.
- **Randomness.** Seeded mode uses SplitMix64 from the case seed. Engine mode reads the choices from libFuzzer's input bytes, so its mutations map onto generator decisions.
- **Calling.** The metaprogram emits a thunk per procedure and a case table; `run_cases` (`stdlib/Extensions/Fuzz/runner.jai`) calls them. If a signature can't be spelled (variadics, `$constant` parameters), the driver denies that procedure and rebuilds.
- **Oracles.** A runtime error, a signal, or a timeout (`--timeout`, 60 s) is a crash. Property hooks in `tools/fuzz/properties/<Module>.jai` define `fuzz_*` procedures that assert a property such as `base64_decode(base64_encode(x)) == x`. They also cover calls generation can't make, like pointer-and-length pairs.
- **Reproducing.** The harness prints `CASE <proc> <seed> <max_len>` before each call. The driver restarts after a crash, minimizes by retrying the seed with smaller length bounds, and with `--write-regressions` stores `fuzz/regressions/stdlib/<proc>.<seed>.json`. `--replay` prints `fixed` or `STILL FAILS`. Commit the file with the fix.
- **Engine mode.** Builds `engine-<Module>` with `jaic -O0 -sanitize address,fuzzer` ([sanitizers](../native/sanitizers.md)). Corpus is in `fuzz/corpus/stdlib/<Module>/` and crashes in `fuzz/artifacts/stdlib/<Module>/` (both gitignored). Replay a crash with `engine-<Module> --trace --only <case> <file>`. ASan reserves a lot of address space, so run one module at a time on a small machine.
- **Postgres.** Its exports need a connection, so `properties/Postgres.jai` builds one whose reader serves the case's bytes as the server reply and whose writer keeps what the client sends. Hooks cover random, truncated and mutated replies, value decoding, round trips and COPY.
- **Sandbox.** Every `jaic` run goes through `tools/bench/sandbox.py` (no network, writes to scratch). Use `--no-sandbox` when the driver already runs inside a sandbox.

## Choosing targets

Per module in `tools/fuzz/targets.json`:

| Key | Meaning |
| --- | --- |
| `deny` / `allow` | procedures to skip, each with a reason / to fuzz exclusively |
| `pointers`, `nullable` | procedures that may get pointers / null pointers |
| `arity` | `{ "proc": N }` passes only the first N arguments |
| `hooks` | property file, relative to `tools/fuzz/` |
| `known` | expected precondition failures, reported as `known` |

Only add modules whose procedures are pure computation. Skip anything touching files, network, processes, allocators, `context` or global state.

## How to change it

- **New module:** add an entry to `targets.json`, run it, and sort the first findings into fix, `known` or `deny`. Prefer a property hook over `known` when the surprise is a contract rather than a bug.
- **New value kind:** edit `values.jai`; keep `can_fill` and `has_pointer` in step with `fill`. `stdlib/Extensions/Fuzz/tests/generators.jai` checks determinism and bounds.
- **Run by hand:** `jaic -interp <scratch>/harness-<Module>.jai -import_dir stdlib -- --only <proc> --seed N --count 1`.
- Gotchas: casts check overflow, so generator code uses `cast,trunc`. `#no_abc` procedures die as signals (`exit -11`) rather than diagnostics.

## Configuration

Harness options: `--seed`, `--count`, `--only`, `--max-len`, `--list`. Engine options: `--time`, `--engine-max-len`, `--opt`, `--timeout`. A module that links native libraries (Postgres) needs `JAIC_NATIVE_LIBS` somewhere the sandbox can read.

## Dependencies

`jaic` with `-sanitize fuzzer`, LLVM's libFuzzer runtime (Homebrew `llvm` or `libclang-rt`), `stdlib/Extensions/Fuzz`, `tools/bench/sandbox.py`.
