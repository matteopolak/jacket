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

### Second pass (null checks, teardown, split)

Warm medians on the same machine, `tools/compile_bench.py --repeat 5`, release `jaic` before and after:

| Workload | Before | After | Notes |
| --- | --- | --- | --- |
| gen-60k `build` -O0 | 0.40 s | 0.33 s | codegen 0.21 s to 0.16 s, RSS 517 to 459 MiB |
| gen-240k `build` -O0 | 1.50 s | 1.15 s | codegen 0.89 s to 0.62 s, RSS 1.58 to 1.31 GiB |
| gen-240k `check` | 0.46 s | 0.42 s | |
| focus `build` -O2 | 12.7 s | 11.0 s | codegen 6.65 s to 5.05 s |
| chess-jai `build` -O2 | 4.51 s | 3.94 s | codegen 3.93 s to 3.33 s |
| Jails `build` -O2 | 2.17 s | 1.64 s | codegen 1.92 s to 1.39 s |
| focus -O0, chess-jai -O0, getrect -O2, `check` of focus | | | within noise |

What each change gained:

- **Cheaper null checks** (`lower::NullFacts`, `null_check`): a check was an `icmp` plus a call to a trap function with the source location, emitted at every pointer load and store. It is now a compare and branch to a trap block that calls the shared `jaic.null_fail` helper, and a pointer already checked in the same IR block (or a constant offset under 4096 from one) is not checked again. Facts are dropped when a slot is stored to or its address escapes. About 14% of the 240k-line build's wall time and 25% of its CPU; the line and message of a failed check are unchanged (`null_checks_skipped_after_a_check_still_catch_changes` in `crates/jaic-cli/tests/native.rs`).
- **No LLVM value names**: `LLVMContextSetDiscardValueNames` on every context. Names were never read.
- **Leaking at exit**: `jaic build`/`check` forget the LLVM module, context and target machine of each unit and the `Compiler` itself (`LeakOnExit` in `jaic-cli/src/main.rs`) instead of freeing them just before the process ends. `jaic run` still drops, since a program can run more than once in one process. Freeing the compiler was about 9% of `check`.
- **`pending_done` in sema scope expansion**: `expand_pending` and `settled` skip the finished prefix of the pending list instead of scanning it every round.
- **Post-optimizer split ignores `enable_split_modules`**: that option now only limits the pre-optimizer split of unoptimized builds. The split after the passes does not change the machine code except for function order, and Focus and chess-jai (which set it to `false`) generate machine code on four threads now.

Tried and rejected: other codegen unit sizes (5,000 stays best, since the cores are 4 fast and 6 slow and total work matters more than balance); removing the parse mutex in `split.rs` (RSS rises by several hundred MiB for nothing); partitioning before the optimizer with every symbol external (see the third pass for what made it work); skipping LiveDebugValues at -O0 (13 to 15% of `llc` time, but without it the object loses `frame variable` locations).

Where the Focus -O2 "front end" time goes: Focus' `build.jai` runs `hdiutil` and `dsymutil` in its release step, so `front end` there includes about 4 s of external work, plus the metaprogram. It is not compiler overhead; the -O0 figure of 1.2 s is the same program without those steps.

Ideas left: LiveDebugValues cost at -O0; load CSE for the null-check facts (under 7% of the build); a shared trap block per function; cutting the `lookup_full` clones of `using` lists in sema; interpreter speed for metaprogram-heavy front ends (Focus: 1.2 s); streaming lowering while sema runs (see the design in the third pass).

### Third pass (constant data, optimizing in parallel)

Warm medians on the same machine under a busy desktop (a browser kept two or three cores busy, so absolute times are about 20% worse than in the passes above; both columns were measured the same way, interleaved), release `jaic`, `tools/compile_bench.py --repeat 5`, main before the pass and after:

| Workload | Before | After | Notes |
| --- | --- | --- | --- |
| focus `build` -O0 | 1.75 s | 1.57 s | on a quiet machine: front end 1.18 s, codegen 0.43 s to 0.22 s; interleaved on the busy one, codegen 0.69 s to 0.30 s |
| focus `build` -O2 | 13.8 s | 8.3 s | codegen 7.6 s to 3.0 s (alone: 4.75 s to 1.9 s); about 4 s of the rest is Focus' own `hdiutil` and `dsymutil` |
| chess-jai `build` -O0 | 1.25 s | 0.75 s | codegen 0.58 s to 0.15 s; the 21 MB network was one `ConstantInt` per byte |
| chess-jai `build` -O2 | 5.2 s | 2.0 s | codegen 4.5 s to 1.3 s, RSS 839 to 384 MiB |
| Jails `build` -O2 | 2.75 s | 0.94 s | codegen 2.4 s to 0.66 s |
| jaison tests `build` -O2 | 0.81 s | 0.49 s | codegen 0.71 s to 0.40 s |
| getrect `build` -O2 | 2.43 s | 0.73 s | codegen 2.1 s to 0.55 s |
| gen-240k `build` -O0 | 1.87 s | 1.33 s | codegen 1.07 s to 0.71 s |
| peak RSS | | | Focus -O2 1338 to 1337 MiB, Focus -O0 1168 to 1084, Jails -O2 396 to 428, getrect -O2 289 to 261, gen-240k 1338 to 1344 |

- **Embedded data as `ConstantDataArray`** (`Backend::initializer`, `lower.rs`): a global's byte runs were built as one `i8` `ConstantInt` per byte and then a `const_array`; they are now one `const_string` (zero runs `const_zero`). `llvm::ConstantInt::get` and the collection of those values were among the top samples of an `-O0` Focus build, and the 21 MB network of chess-jai is the extreme case. Focus -O0 codegen halves; `global_byte_data_survives_unoptimized_and_optimized_builds` (`embedded_data.rs`) checks the contents at both levels.
- **Dividing optimized programs before the optimizer** (`partition.rs`, [LLVM backend](../native/llvm-backend.md#dividing-before-the-optimizer-partitionrs)): for Focus the passes were 3.2 s of 4.75 s of codegen, on one thread, and spread over the whole pipeline (InstCombine 17%, inliner 11%, GVN 6%, SROA 5%, no single hot pass), so there is no cheap serial fix. The module is now divided by call graph into up to 8 parts (one per 20,000 IR instructions), each lowered into its own module and run through the pipeline and the code generator on its own thread. The CPU time of the passes goes up (two to three times in total on this machine: copies of small callees, 4 fast and 6 slow cores, memory bandwidth), the wall time of the phase falls to 40%.

Why the earlier attempt at this (a plain split with every symbol external) gained nothing: it kept the whole program's declarations in every part, took away the single-caller and dead-code opportunities of internal linkage, and left no way to inline a callee that landed in another part. Three things changed it: functions are grouped so that most of them have all their callers in their own part and stay internal; small callees of other parts are copied in as `available_externally`; and small read-only data (zero default values: a private copy per part) is visible to the code that reads it. Without the data rule, the Chess engine's executable gained a 2.8 MB zero block (the default `ChessGame`) that the whole-module build had folded away.

Runtime of the code produced (same sources built with `JAIC_CODEGEN_UNITS=1`, the old whole-module optimization, and with the division; instructions retired by `/usr/bin/time -l`, run-to-run spread about 1%):

| Program | Whole module | Divided | Difference |
| --- | --- | --- | --- |
| Chess engine, search to depth 15 (7 parts) | 25.8 G | 26.1 to 26.5 G | +1.4% to +3% (varied with the copy limits tried) |
| CPU mix (n-body, `fib`, `Hash_Table`, sort, `String_Builder`), forced into 4 and 8 parts | 49.5 G | 49.5 G | none |
| jaison tests, `-O2` | same output | same output | |

The Chess difference is calls that are no longer inlined: `heapify` and similar procedures above the 200-instruction copy limit that sit in another part than their caller. Raising `IMPORT_MAX` to 400 (with a budget of three times the owned code) recovered about a point of it for more than 50% more codegen time on Focus. The division is on by default whatever `enable_split_modules` says (Focus sets it to `false`); `JAIC_CODEGEN_UNITS=1` restores whole-program optimization.

Tried and rejected in this pass: a post-optimizer split alone (the passes, not the code generator, are the serial part); importing functions up to 400 or 1,000 instructions with a two or three times larger budget (0.5 to 1.5 points fewer instructions in the Chess engine, but 50% to 100% more codegen time on Focus); fewer parts (4 parts: Focus codegen 2.9 s; the optimizer's work per part does not shrink faster than the parts' count grows); sharing zero default values (the executable grew).

Where the rest goes (Focus -O2, alone on the machine): front end 1.2 s, codegen 1.9 s, link 0.1 s, plus Focus' own packaging. The `-O0` build is 1.2 s of front end, 0.2 s codegen and 0.1 s link; the front end is the interpreter running Focus' build metaprogram (`Interp::run_ops` is most of the working thread's samples), so getting under a second needs interpreter work, not codegen.

#### Overlapping codegen with the front end (design, not built)

Code generation starts after every procedure is lowered to IR because nothing in `Program` is final until the workspace completes: metaprograms can still add code (`add_build_string`), change a procedure through the message loop and set build options (`null_pointer_check`, debug info, `check_failed`), and the type table and stack-trace tables (`typeinfo.rs`, `stack_trace.rs`) are built from the final set of types. The one phase that could overlap safely is lowering the IR of procedures that can no longer change, and its gain is bounded by the smaller of the two: about 0.2 s for an `-O0` Focus, up to the 1.2 s of front end for an `-O2` one.

A sketch that keeps the risk contained: (1) a "frozen" flag on `Func`, set when sema has finished the procedure and a `PROCEDURE_BODY_READY` message has been delivered and acknowledged (no metaprogram can change it any more); (2) the front end hands frozen procedures to a worker pool that builds LLVM IR for them into per-thread modules (`Backend::define_function` needs only `&Program` plus lazily declared symbols, which the partitioning in `partition.rs` already isolates); (3) at completion, the remaining procedures, globals and type tables are lowered, and the partition plan (which needs the full reference graph) is computed once. Step (3) is why the pieces cannot be finished early at `-O2`: ownership and `internal` versus `external` linkage depend on all references. For `-O0` they do not, so an `-O0` build could stream into shards chosen by a running size count. The costs are a second copy of the IR living while sema still grows the interner (memory), a lock around `Program` growth, and interactions with `#run` (which also reads `Program`). It was not attempted: the possible gain at the default `-O0` is small next to the 1.2 s of front end, and the risk is spread across sema, metaprogram messages and the interpreter.

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
