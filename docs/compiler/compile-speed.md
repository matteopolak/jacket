# Compile speed

## What it is

Where `jaic` spends time turning source into an executable, and the design choices that keep it fast. `jaic -timings` prints the phases.

## How it works

An `-O0` build is: front end (parse, sema, compile-time code), `codegen` (IR to LLVM to machine code, one thread per codegen unit), then `link`. Rough scale on a 10-core Apple machine: 60k generated lines in 0.3 s, 240k in 1.2 s; hello world 0.07 s. For metaprogram-heavy projects such as Focus the front end is mostly the [interpreter](interpreter.md) running the build program.

Choices that matter:

- **FastISel at `-O0`** (`use_fast_isel`, `crates/jaic-llvm/src/lib.rs`). GlobalISel is LLVM's AArch64 default at `-O0` and its instruction selector dominated codegen; FastISel halved it.
- **Lazy declarations** (`lower::Backend::func/global/foreign`): a codegen unit declares only the symbols it references. Any lookup of a symbol by name must call `declare_named` first, and new per-function steps in `lower.rs` must not iterate the whole program in every unit.
- **Units of 5,000 instructions** (`INSTS_PER_UNIT`). Other sizes were worse: total work matters more than balance.
- **Verifier off in release builds** (`verify_ir`, `JAIC_VERIFY_IR`); **no LLVM value names** (`LLVMContextSetDiscardValueNames`).
- **No `dsymutil` at `-O0` on macOS** ([debug info](../native/debug-info.md)).
- **Cheap null checks** (`lower::NullFacts`, `null_check`): a compare and branch to a shared `jaic.null_fail` helper, skipped for a pointer already checked in the same IR block. Facts are dropped when a slot is stored to or its address escapes.
- **Leaking at exit**: builds and `-check` forget the LLVM objects and the `Compiler` (`LeakOnExit` in `jaic-cli/src/main.rs`) instead of freeing them. `-interp` still drops, since a program can run more than once.
- **Constant data**: a global's bytes become one `const_string` (zero runs `const_zero`), not one `ConstantInt` per byte. A 21 MB embedded network went from seconds to milliseconds.
- **Optimized builds are divided before the optimizer** (`partition.rs`, [LLVM backend](../native/llvm-backend.md)): the call graph is split into up to 8 parts, each optimized and code-generated on its own thread. `JAIC_CODEGEN_UNITS=1` restores whole-program optimization; the division applies whatever `enable_split_modules` says. `partition::inline_cuts` stops any function growing past 4,000 instructions by inlining; without it, a generated driver calling hundreds of procedures once each pulled them all in and loop passes went superlinear (250k lines: 301 s before, 17 s after).
- **Unoptimized runtime**: small copies and fills (up to 64 bytes) are emitted as loads and stores (`Backend::inline_copy`) because FastISel turns `memcpy` into a library call; constant divisors and shifts skip their guards; `x += y` does not go through a hidden address slot; [stack trace nodes](stack-traces.md) are omitted for leaf procedures and line stores only happen when a call can follow. Variables stay in stack slots: `mem2reg` at `-O0` made hot loops slower under the fast register allocator and hides variables from debuggers.

### Interpreter speed

`#run`, metaprograms, `jaic -interp` and the playground run in `interp/` ([interpreter](interpreter.md#the-interpreters-code-form)). It uses one flat op array with a program counter, registers on the interpreter stack (a call is `enter` plus an argument copy), specialized load/store/div/call ops, boxed `Trap` to keep `Res<u64>` at 16 bytes, and a region cache in the memory probe (`interp/probe.rs`) flushed by an epoch that `invalidate()` bumps after any foreign call that can unmap memory. Known remaining costs: `trace_enter` under `jaic -interp`, second-table dispatch for compare ops, and calls recursing on the Rust stack.

### Rejected experiments

- Skipping LiveDebugValues at `-O0`: loses `frame variable` locations.
- Removing the parse mutex in `split.rs`: memory rises for no gain.
- Copying callees above 200 instructions into other parts, or fewer than 8 parts: more codegen time than the runtime it recovers.
- Overlapping codegen with the front end: nothing in `Program` is final until the workspace completes (metaprograms can add code, change procedures and set build options; type and stack-trace tables use the final type set). Only lowering of finished procedures could overlap, and the gain is small at `-O0`.

## How to change it

- Measure with `jaic x.jai -exe out -timings` on an idle machine, 5 or more warm runs (a busy process moves results 10 to 20%). `tools/compile_bench.py` ([benchmarks](../tools/benchmarks.md)) runs corpus projects; for scaling use the corpus-shaped programs from `tools/benchgen.py` (`gen-10k`/`gen-60k`/`gen-240k`).
- To compare ISel strategies, run `llc -O0 -global-isel=0` on `-emit_ir` output.
- Profile with samply ([benchmarks](../tools/benchmarks.md)). `Interp::run_code` is the main item left for metaprogram-heavy code, then `trace_enter`, then `build::call` and `write_item`.
- Ideas left: LiveDebugValues cost at `-O0`; load CSE for null-check facts; fewer `lookup_full` clones of `using` lists in sema; interpreter speed.

## Configuration

- `JAIC_CODEGEN_UNITS=N`, `JAIC_VERIFY_IR`, `JAIC_DSYM`: see [LLVM backend](../native/llvm-backend.md#configuration) and [debug info](../native/debug-info.md).
- `-timings`: phase table on stderr.

## Dependencies

LLVM 23 through `inkwell`, and the system linker.
