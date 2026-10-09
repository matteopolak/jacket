# Compile speed

## What it is

How fast `jaic build` turns source into an executable, where an unoptimized (`-O0`) build spends its time, and what was changed to speed it up. `jaic --timings` prints the phases; the method and workloads below reproduce the numbers.

## How it works

An `-O0` build is: front end (parse, sema, compile-time code), then `codegen` (lowering IR to LLVM and machine code, on one thread per codegen unit), then `link`. Measured on an Apple M5 (10 cores), warm medians, release `jaic`:

| Workload | Before | After | Notes |
| --- | --- | --- | --- |
| synthetic 60k lines | 0.71 s | 0.42 s | codegen 0.51 s to 0.27 s |
| synthetic 240k lines | 4.1 s | 1.7 s | codegen 3.4 s to 1.2 s, RSS 1.98 to 1.79 GiB |
| `jaifmt/main.jai` | 0.19 s | 0.117 s | codegen 0.09 s to 0.043 s; `dsymutil` 0.024 s gone |
| hello world | 0.12 s | 0.068 s | |
| Jails `check` | 0.325 s | 0.31 s | 90% compile-time interpreter |

Lines per second for the synthetic programs went from about 85k and 59k to about 142k. What each change gained, on the synthetic programs:

- **FastISel instead of GlobalISel** (`use_fast_isel`, `crates/jaic-llvm/src/lib.rs`): the largest change: on its own it took the 240k-line build from 4.8 s to 2.0 s (codegen 4.0 s to 1.3 s). GlobalISel is LLVM's AArch64 default at `-O0` and its instruction selector dominated codegen samples.
- **Lazy declarations** (`lower::Backend::func/global/foreign`): each codegen unit used to declare every function, global and foreign symbol of the program and define only its share. Now it declares what it references. About 5% of the 240k-line build.
- **Verifier off in release jaic** (`verify_ir`): about 3%.
- **5,000 instructions per unit** (`INSTS_PER_UNIT`): helps small projects most (`jaifmt` and hello world go from a few units to all cores); neutral on large ones, which hit the core cap.
- **No `dsymutil` at `-O0` on macOS**: 0.02 to 0.04 s per build, 10 to 15% of a small one. See [debug info](../native/debug-info.md).

Not worth it: units below 5,000 instructions (hello world already uses all cores).

## Compile-time interpreter

`#run`, metaprograms, `jaic run` and the playground all execute in `interp/`. Method: release `jaic` (`--release`, same machine, idle), median of 7 warm runs per workload, before versus after. Workloads are `check` on corpus projects that run metaprograms (Focus, Jails, sgpu examples, Jai-Shader-Transpiler, yield-jai), `run` on jaison's tests, and `run` on CPU-bound programs (`benchmarks/interp-loops.jai`, `interp-strings.jai`, `interp-calls.jai`, and a mixed one).

| Workload | Before | After |
|---|---|---|
| Focus `check first.jai` | 2.10 s | 1.66 s |
| Jails `check build.jai` | 0.292 s | 0.230 s |
| sgpu examples `check` | 0.447 s | 0.470 s |
| Jai-Shader-Transpiler `check` | 0.103 s | 0.098 s |
| yield-jai `check` | 0.090 s | 0.075 s |
| jaison `run tests.jai` | 0.423 s | 0.383 s |
| `interp-loops` | 0.197 s | 0.170 s |
| `interp-strings` | 1.201 s | 1.062 s |
| `interp-calls` | 0.053 s | 0.047 s |
| mixed CPU-bound program | 0.590 s | 0.499 s |

Gains by change:

- **Region cache in the memory probe** (`interp/probe.rs`): the probe used to ask the OS about a page per check. It now keeps a page table and a small cache of the mappings the system described, flushed by a global epoch that `invalidate()` bumps on any foreign call that can unmap memory (and on `ZeroedBlock` drop). The largest single item on Focus (about 15%).
- **Boxed `Trap`**: `Res<u64>` shrank from a large enum to 16 bytes, which speeds every `?` in the dispatch loop.
- **Specialized ops** (`interp/code.rs`): `Load8/32/64`, `Store8/32/64` and their frame forms, `Div`, `DivImm`, `Call` and `BoundsCheck` replace a second switch on the type and the generic `step` path. Loop and call-heavy programs gain 10 to 15%.
- **Compiler primitives** (`build.rs`): `call` returns results inline (`Rets`) and `field_name` reads the name without allocating; the tag lookup is a map. Metaprogram-heavy checks gain a few percent.

sgpu examples got about 5% slower (noise-level but repeatable): it is dominated by native foreign calls, which pay the epoch bump after each non-whitelisted call.

### Second pass: calls, registers and foreign calls

Same method, release builds, median of 7 runs, on the first pass's result (`After` above) versus now:

| Workload | Before | After |
|---|---|---|
| `interp-calls` | 0.048 s | 0.035 s |
| `interp-loops` | 0.168 s | 0.094 s |
| `interp-strings` | 1.058 s | 0.509 s |
| `interp-fib` (`fib(37)`-style recursion, new) | 0.331 s | 0.167 s |
| `interp-loops-long` (new) | 1.132 s | 0.553 s |
| `interp-foreign` (60M `labs` calls, new) | 7.86 s | 2.55 s |

What changed (details in [interpreter](interpreter.md#the-interpreters-code-form)):

- **Calls**: registers live on the interpreter's stack and a call is `enter` plus a copy of the arguments, with no pool, clone or allocation; `FrameExit` is a small struct. A named call runs from the `Call` op without `exec`.
- **Dispatch**: one flat op array with a program counter; block endings are ops and the next block falls through. Source locations left the op stream.
- **Fewer ops**: scalar slots become registers, with store and copy forwarding (`code/promote.rs`).
- **Foreign calls**: per-procedure flags replace two string scans, and results and signatures no longer allocate or clone. `executable_path_foreign` checks the symbol before cloning the path. A wider set of libc functions is known not to release memory, so they no longer flush the probe cache (the suspected cause of the sgpu slowdown; the corpus projects were not rerun in this pass).
- **Intrinsics** return `Rets` instead of a `Vec`.

Left: `trace_enter` is about 16% of `interp-fib` (stack traces are on under `jaic run`; its layout lookups could be cached per frame), compare and arithmetic ops still dispatch through a second table on the operator, and calls still recurse on the Rust stack where an explicit activation stack would save the `run_ops` prologue.

## How to change it

- Measure with `jaic build x.jai -o out --timings` on the same machine, 5 or more warm runs, and check the machine is idle (a stray busy process moves results by 10 to 20%). `tools/compile_bench.py` ([compile-time benchmark](../tools/compile-time-benchmark.md)) runs the corpus projects; for scaling use the corpus-shaped programs from `tools/benchgen.py` ([benchmark generator](../tools/benchmark-generator.md); `gen-10k`/`gen-60k`/`gen-240k` in `compile_bench.py`).
- To compare ISel strategies externally, `llc -O0 -global-isel=0` on `--emit-ir` output.
- A new per-function or per-module step in `lower.rs` should not iterate over the whole program in every shard: use the lazy accessors. Any lookup of a symbol by name must call `declare_named` first.
- Profile with samply as described in [benchmarks](../tools/benchmarks.md); `Interp::run_code` is the main self-time item left for metaprogram-heavy code, then `trace_enter`, then `build::call` and `write_item`, which read and write compiler records through the probe field by field.

## Configuration

- `JAIC_CODEGEN_UNITS=N`, `JAIC_VERIFY_IR`, `JAIC_DSYM`: see [LLVM backend](../native/llvm-backend.md#configuration) and [debug info](../native/debug-info.md).
- `--timings`: phase table on stderr.

## Dependencies

LLVM 23 through `inkwell` (`LLVMSetTargetMachineGlobalISel`/`FastISel`), the system linker.
