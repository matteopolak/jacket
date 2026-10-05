# Benchmarks and profiling

## What it is

`tools/bench.py` times `jaic` on small interpreter and compile-time programs (`benchmarks/*.jai`) and on the larger corpus projects. `JAIC_PROFILE=1` reports where interpreted time goes, per Jai procedure. samply with `tools/profile_report.py` covers the native compiler.

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

What the first round of profiling found (all stdlib fixes; interpreted instructions):

- **Pointer hashing.** `Hash.get_hash` returned a constant for pointers, so every pointer-keyed `Table` was one long chain. Vk-Engine went from 7.6B to 1.0B instructions (15.5s to 3.0s).
- **Compiler records.** `record_of` scanned every record linearly. It now uses a pointer-to-id index (`record_slots`).
- **U128.** Division and multiplication used bit loops. They now have a 64-bit divisor fast path and 32-bit limbs. jaison went from 2.3s to 0.5s, together with the memory debugger index (`md_find` was a linear scan).
- **Float printing.** Exact digit generation now runs on a `u64` first and retries with wide limbs only on overflow, and it scales by powers of two with shifts. `String_Builder` copies with `memcpy` and appends single bytes directly. `interp-strings` went from 2.09s to 1.19s.

A second round on the front end (Focus check: 1.16B to about 0.41B interpreted instructions):

- **Compiler records** were about half of a Focus build: filling hundreds of thousands of `Code_Node`/`Type_Info` structs member by member through reflection, two primitive calls per member. Ints, strings and built pointers are now written natively from a cached per-type plan (see [compiler records](../metaprogramming/compiler-records.md)).
- **`Default_Allocator`** scanned a linked list of every live allocation on each free. It is a hash set now.
- **Lenient body lowering** looked at every previously failed body before each compile-time call; unchanged failures are parked.
- **Interpreter calls** no longer allocate a `Vec` for results or SipHash the stack-trace info.
- **Hashing**: sema, the interner, the type table and the interpreter use `jaic::fxhash` (a multiply-rotate hasher) instead of SipHash; every name lookup hashes a `Sym` several times. Member lookup no longer clones the struct's field list.

What is left is the interpreter's dispatch (`Interp::exec` is about half of native time, spread over ordinary instructions) and sema spread thin over many functions. The kind breakdown shows `Loc` (source positions, 10%) and `IConst`/`SlotAddr` (17% and 9%) as candidates for a denser IR, which would be a redesign rather than a fix.

## Configuration

- `JAIC_PROFILE`: enables the interpreter profile.
- `bench.py`: `--jaic` (default `$CARGO_TARGET_DIR/release/jaic`, else `target/release/jaic`), `--repeat`, `--only`, `--out`, `--compare`.
- `[profile.profiling]` in the root `Cargo.toml`.

## Dependencies

- Python 3.
- The corpus from `tools/fetch_upstreams.py`, for the project workloads.
- samply and the Xcode `llvm-symbolizer`/dSYM tooling, for native profiles only.
