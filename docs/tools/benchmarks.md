# Benchmarks and profiling

## What it is

`tools/bench.py` times `jaic` on small interpreter and compile-time programs (`benchmarks/*.jai`) and on the larger corpus projects. `JAIC_PROFILE=1` reports where interpreted time goes, per Jai procedure. samply with `tools/profile_report.py` covers the native compiler. Whole-project compile times (check, `-O0`, `-O2`, peak RSS, phases) are measured by the [compile-time benchmark](compile-time-benchmark.md).

## How it works

`bench.py` runs each workload `--repeat` times and keeps the median wall time. It then runs it once more with `JAIC_PROFILE=1` and reads the total interpreted instruction count. Instruction counts barely vary between runs (only pointer-keyed tables shift with ASLR), so they show small changes that wall time hides in noise.

| Workload | What it exercises |
|---|---|
| `compile-time` | polymorphic structs, a `#run` table, `#insert` of 200 generated procedures |
| `interp-calls` | recursive calls (`fib(27)`) |
| `interp-loops` | integer loops, arrays, a sieve |
| `interp-strings` | `tprint` with floats, `String_Builder`, `Hash_Table`, `split` |
| `focus-check` | `jaic check first.jai` in Focus: compiler messages and `Compiler` records |
| `jaison-tests` | jaison's test suite run in the interpreter (U128 math, the memory debugger) |
| `jails-check`, `sgpu-examples-check` | metaprograms of two mid-sized projects |

Vk-Engine is left out: its `Build.jai` rewrites generated bindings inside the corpus and fails at the Jolt step (see [Vk-Engine](../native/vk-engine.md)).

```sh
python3 tools/bench.py --out before.json
# change something, rebuild
python3 tools/bench.py --compare before.json
```

```
workload                  median s    instructions  vs baseline
interp-strings               1.187     460,112,784  0.57x time, 0.48x instructions
focus-check                  3.735   1,158,988,403  0.98x time, 1.00x instructions
```

### Interpreter profile

With `JAIC_PROFILE=1` (any value but `0`), every interpreter counts calls, basic blocks and instructions per procedure. Counts are *self* counts: a call's instructions belong to the callee. `crates/jaic/src/interp/profile.rs` merges the counts of all interpreters (compile-time `#run`, metaprograms, the program) when each one is dropped. The CLI prints the top 40 to stderr after the run:

```
interpreter profile: 460112784 instructions
     %   instructions       blocks      calls  procedure
 22.8%      104963618     10556416    1266718  append
```

Polymorphic instances keep their numbered names (`NewArray#446`). A last line breaks the instructions down by kind (`Load 26.1% IConst 16.8% Store 12.1% Loc 10.3% ...`), which shows whether the IR itself is wasteful. When profiling is off the cost is one `Option` check per call and per block.

### Native profile

For time spent in the compiler itself (parsing, sema, IR lowering, interpreter dispatch), build with the `profiling` profile (release with line tables in a packed dSYM) and record with [samply](https://github.com/mstange/samply):

```sh
cargo build --profile profiling -p jaic-cli
samply record --save-only --unstable-presymbolicate -o prof.json.gz -- target/profiling/jaic check first.jai
python3 tools/profile_report.py prof.json.gz --top 30
```

`profile_report.py` uses the busiest thread. Natively that is the 1 GiB compiler worker, not the main thread. It prints self and inclusive sample shares per function, with symbols from the `.json.syms.json` that samply writes next to the profile. `samply load prof.json.gz` opens the same file in the Firefox Profiler.

## How to change it

- **New micro-benchmark:** add `benchmarks/<name>.jai` with a `main`. It must exit 0 and print something that depends on the work, so nothing is optimized away later.
- **New corpus workload:** add a row to `CORPUS` in `bench.py`. Only add workloads that succeed and do not write into `corpus/upstream`.
- **Profile columns:** the interpreter counts per frame in `run_blocks` (`frame_blocks`, `frame_insts`). `Interp::exec` saves and restores them around each call, so recursion still gives self counts.

### Known hot spots

Past profiles found the same patterns repeatedly, so check for them first:

- **Hashing.** A weak or constant hash turns a `Table` into one long chain. Pointer keys go through `knuth_hash`; the Rust side uses `jaic::fxhash` instead of SipHash, since every name lookup hashes a `Sym` several times.
- **Linear scans in the stdlib.** `Default_Allocator`'s ledger and the memory debugger's index are hash sets, and compiler records are found through a pointer-to-id index (`record_slots`). A list scanned on every call is the usual cause of a quadratic metaprogram.
- **Compiler records.** Filling `Code_Node`/`Type_Info` structs member by member through reflection used to dominate Focus builds. Records are now written natively from a cached per-type plan, from big chunks rather than one allocation each ([compiler records](../metaprogramming/compiler-records.md)).
- **Wide-integer and float printing.** `U128` has a 64-bit fast path and 32-bit limbs; float digit generation runs on a `u64` and widens only on overflow.

What remains is the interpreter's dispatch (`Interp::exec` is about half of native time, spread over ordinary instructions) and sema spread thin over many functions. The kind breakdown points at `Loc` (source positions) and `IConst`/`SlotAddr` as candidates for a denser IR; that would be a redesign rather than a fix.

## Configuration

- `JAIC_PROFILE`: enables the interpreter profile.
- `bench.py`: `--jaic` (default `$CARGO_TARGET_DIR/release/jaic`, else `target/release/jaic`), `--repeat`, `--only`, `--out`, `--compare`.
- `[profile.profiling]` in the root `Cargo.toml`.

## Dependencies

- Python 3.
- The corpus from `tools/fetch_upstreams.py`, for the project workloads.
- samply and the Xcode `llvm-symbolizer`/dSYM tooling, for native profiles only.
