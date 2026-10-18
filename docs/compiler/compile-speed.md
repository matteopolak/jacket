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
- **Real paths are remembered** (`Compiler::canon`, over `FileSystem::canonical_exact`): the standard library's modules are named again and again, and each `realpath` walk was a fifth of a hello world's front end. Only paths that resolved are kept, since a metaprogram can write a file later. Use `self.canon`, not `self.fs.canonical`, in new sema code.
- **Intercepted builds export typechecked code with cached lookups** (`ExportState::name_lookups`, `type_names`, dropped whenever the entity count changes). A workspace whose metaprogram intercepts messages (the benchmark harness's phase driver, build programs) pays for every exported identifier otherwise.
- **Dead platform branches of library files are not lexed or parsed** (`lexer::lex_for_target`, `parser::parse_file_for_target`, used by `Compiler::library_platform` for files under the standard library root). A file-level `#if` whose condition is only `OS` / `CPU` tests (`==`, `!=`, `!(..)`, `&&`, `||`) is decided from the target: the lexer steps over a false branch's body byte by byte (comments, strings and notes followed as `run` does; a here-string makes it fall back to lexing normally) and leaves `{ }`, and the parser replaces the condition with a `Bool` so sema never evaluates it. `POSIX/declarations.jai` is 47k generated lines for four platforms and every program loads it; this took an empty program from 43 ms to 34 ms. Gotchas: a syntax error inside a branch the target rules out is not reported (user code is always parsed in full), and a new token that contains `{` must be counted in `skip_block` and in `track_brace`.
- **Names a module lacks are remembered** (`module_lookup`, `Compiler::module_misses`, keyed by `namespace_generation`): a scope asks every module it imports about every name it uses, and each miss walked the module's re-exports and `using` globals. Only names whose `module_names` entry is settled are kept, so anything that changes what a module exports must go through `namespace_changed`. `gen-60k` `-check` 0.195 to 0.170 s.
- **Stack trace instrumentation** (`stack_trace::instrument`, part of `prepare output`) decides which procedures are quiet with one parallel scan of each body plus a worklist over reverse call edges, not repeated whole-program passes: 21 ms to 7 ms on 60k lines, `prepare output` 24 ms to 10 ms.
- **Leaking at exit**: builds and `-check` forget the LLVM objects and the `Compiler` (`LeakOnExit` in `jaic-cli/src/main.rs`) instead of freeing them. `-interp` still drops, since a program can run more than once.
- **Constant data**: a global's bytes become one `const_string` (zero runs `const_zero`), not one `ConstantInt` per byte. A 21 MB embedded network went from seconds to milliseconds.
- **Optimized builds are divided before the optimizer** (`partition.rs`, [LLVM backend](../native/llvm-backend.md)): the call graph is split into up to 8 parts, each optimized and code-generated on its own thread. `JAIC_CODEGEN_UNITS=1` restores whole-program optimization; the division applies whatever `enable_split_modules` says. `partition::inline_cuts` stops any function growing past 4,000 instructions by inlining; without it, a generated driver calling hundreds of procedures once each pulled them all in and loop passes went superlinear (250k lines: 301 s before, 17 s after).
- **Name lookup** (`sema/scope.rs`): `module_using_member` (the `M.name` / scope fallback through a module's exported `using` values) used to run for every import of every scope on each miss. `module_may_have_using_members` now remembers, per `namespace_generation`, that a module and everything it re-exports hold no exported `using` (`Compiler::using_member_free`), so `Basic`/`Math` re-exports cost one hash probe. `lookup_full_raw` also stopped cloning a scope's name list and `using` entries and moves a found entity list into the result instead of copying it. A 60k-line `-check` retires about 14% fewer instructions.
- **Unoptimized runtime**: small copies and fills (up to 64 bytes) are emitted as loads and stores (`Backend::inline_copy`) because FastISel turns `memcpy` into a library call; constant divisors and shifts skip their guards; `x += y` does not go through a hidden address slot; [stack trace nodes](stack-traces.md) are omitted for leaf procedures (and for any procedure that only passes its context on; a regression here once gave every helper a node, 15% of `runtime-compute`) and line stores only happen when a call can follow. Variables stay in stack slots: `mem2reg` at `-O0` made hot loops slower under the fast register allocator and hides variables from debuggers.

### Parallel lex and parse

`parse_ahead.rs` lexes and parses source files on worker threads while sema runs. At the start of `prepare_sources` (`Compiler::start_parse_ahead`) it queues the files a compile loads first: the preload module, `Runtime_Support` and the program's files. Each worker reads and parses a file (standard library files with `parse_file_for_target`, as `library_platform` decides), then scans its top-level `#load` and `#import` statements, resolves them with the functions sema uses (`find_module_in`, relative `dir.join`) and queues files not yet seen. A `#if` on `OS`/`CPU` (combined with `&&`, `||`, `!`) follows only the branch that holds for the target; other conditions follow both branches.

Sema asks for a file in `load_file_inner` through `ParseAhead::claim(canonical path)`:

- finished: it takes the parse, adds the source file, and `PreparedTree::finish(file_id)` rewrites the tree (`ast/relocate.rs`);
- still being parsed: it waits;
- queued but not started, never predicted (a computed `#load` path, a `#run`-generated load), a second instance of an already claimed file, or any failure: it parses inline, exactly as before, and `discover`s that file's imports for the workers. Parse errors always come from this path, so diagnostics are unchanged.

Results match a serial run byte for byte because nothing a worker produces depends on timing. File ids are assigned when sema claims (not when the worker finishes). A worker parses with a placeholder file id, `AstId`s from zero and a per-job symbol table (`intern::begin_job`/`end_job`: the spellings in first-seen order). At claim, on the main thread, those spellings are interned in order (what the serial parse would have interned), `AstId`s are reserved from the compile's counter, and `relocate` rewrites every span, id and symbol. A wrong prediction only wastes a parse. The tree moves between threads through `Handoff`, an `unsafe impl Send` justified by the tree owning all its `Rc`s.

Measured on the 10-core development Mac (sandboxed shell, mimalloc), `-check`, median of 21: hello 33.1 -> 29.2 ms, 60k generated lines 146 -> 120 ms, using up to 4 workers (about 1.3 cores of CPU per check instead of 1). What stays on the main thread is the claim itself (source map entry and relocation, about 5% of a 60k-line check), waiting for the first big file, and the roughly 10 ms of process start and exit that hello world is mostly made of. `-timings` still shows one `front end` phase, now shorter.

The IDE (`Compiler::ide`), non-native filesystems and `wasm32` always parse serially. `JAIC_SERIAL_PARSE=1` turns the workers off (use it to compare outputs); `JAIC_PARSE_THREADS=N` sets their number (default: cores - 1, at most 4).

### Interpreter speed

`#run`, metaprograms, `jaic -interp` and the playground run in `interp/` ([interpreter](interpreter.md#the-interpreters-code-form)). It uses one flat op array with a program counter, registers on the interpreter stack (a call is `enter` plus an argument copy), specialized load/store/div/call ops, boxed `Trap` to keep `Res<u64>` at 16 bytes, and a region cache in the memory probe (`interp/probe.rs`) flushed by an epoch that `invalidate()` bumps after any foreign call that can unmap memory. Constant `#run` thunks are profiled like any other call (`LeakOnExit` flushes the profile before leaking). Remaining costs: call overhead (about a quarter of `interp-strings`; `run_ops` recurses per call), dispatch mispredicts, `trace_enter` under `jaic -interp`, second-table dispatch for compare ops, and calls recursing on the Rust stack.

Compile-time code is mostly the standard library's own code, so its cost is measured in interpreted instructions (`JAIC_PROFILE=1`). `benchmarks/interp-strings.jai` fell from 1.50 to 0.54 billion by fixing three stdlib paths, not the interpreter: `String_Builder` `append` (one byte or a string) copies straight into the buffer when it fits instead of ensure, copy, advance with their assertions; `print_u64_base_n` appends a number once, not a digit at a time; and float formatting (`portable_float`) moves factors of two out of the mantissa and multiplies powers of five in word-sized chunks, so `2.5` costs one big-number multiply instead of fifty-one. When a compile-time workload is slow, read the profile's procedure table before touching `run_ops`.

Child processes (`run_command`): the supervisor that watches a command polls for its exit starting at 1 ms and backing off to 10 ms, since a fixed 10 ms poll added up to 10 ms to every short command (a build metaprogram runs `xcrun` twice and the linker once).

### Rejected experiments

- Skipping LiveDebugValues at `-O0`: loses `frame variable` locations.
- Removing the parse mutex in `split.rs`: memory rises for no gain.
- Copying callees above 200 instructions into other parts, or fewer than 8 parts: more codegen time than the runtime it recovers.
- Cheaper links on macOS: `ld` run directly instead of through `cc` saves about 10 ms of driver start-up but needs the SDK path and platform version re-derived, so it was not adopted; `-no_uuid`, `-ld_classic` and `-no_adhoc_codesign` moved nothing worth the output change. The link phase is 31 ms at best for 60k lines; the occasional 100 ms or more is machine load (other processes writing and linking at once), not a fixed cost.
- Overlapping codegen with the front end: nothing in `Program` is final until the workspace completes (metaprograms can add code, change procedures and set build options; type and stack-trace tables use the final type set). Only lowering of finished procedures could overlap, and the gain is small at `-O0`.

### Allocator

`jaic`'s global allocator is `memory_limit::CountingAllocator<A>`, which counts live bytes for `JAIC_MEMORY_LIMIT` and forwards to `A`. `A` is mimalloc through the `jaic-cli` feature `mimalloc`, which `dynamic-llvm` (the default) and `static-llvm` (release archives) both turn on. A `--no-default-features` build without either uses `System`. Paired A/B runs on an Apple M5 (system build and mimalloc build alternated per run, 10 pairs each, median of the per-pair ratios):

| workload | system s | mimalloc s | change | peak RSS MiB (sys / mi) |
|---|---:|---:|---:|---:|
| focus/check | 1.068 | 1.004 | -6.8% | 723 / 771 |
| focus/build-O0 | 1.455 | 1.393 | -4.4% | 1114 / 1205 |
| focus/build-O2 | 8.829 | 7.713 | -7.4% | 1285 / 1373 |
| jails/check | 0.239 | 0.218 | -7.6% | 209 / 215 |
| jails/build-O0 | 0.387 | 0.349 | -5.4% | 296 / 313 |
| gen-240k/check | 0.516 | 0.449 | -13.3% | 508 / 463 |
| gen-240k/build-O0 | 1.082 | 1.026 | -6.4% | 1124 / 1207 |
| gen-60k/check | 0.177 | 0.156 | -11.3% | 199 / 152 |

The geometric mean is about 8% faster. Peak RSS is 4–8% higher on corpus projects. A later run on the same machine measured the 60k-line check at 154 ms with the system allocator and 137 ms with mimalloc (-11%), with malloc at 23% of samples under the system allocator, so mimalloc is now on in every LLVM build. To re-measure, build once with `--no-default-features --features llvm,jaic-llvm/dynamic` (system) and once by default, and alternate the runs.

**C allocations.** On macOS and Linux the `mimalloc` crate's `override` feature (a `cfg(not(windows))` dependency in `jaic-cli/Cargo.toml`; the crate cannot override on Windows) also replaces `malloc`, `free` and `operator new`'s backing, so LLVM's C++ allocations use mimalloc too. Before, they were `libsystem_malloc`, 8% of samples in a 60k-line build. Medians of 15 alternating runs on an Apple M5 under load: wall 323 to 303 ms, user CPU 940 to 790 ms, `codegen` 111 to 90 ms, hello world unchanged.

On macOS the override registers a malloc zone, and that zone leaves libmalloc's `reinit_lock` hook null while declaring version 10. libmalloc calls it in the child after `fork`, so every `fork` (compile-time `run_command`) crashed. `jaic_llvm::fix_malloc_zone_for_fork` (`malloc_zone.rs`, called first thing in `main`) finds the zone with `malloc_get_all_zones` and fills the slot with a no-op. `malloc_default_zone` is not usable for this: it is not interposed for calls from the executable and names the system zone. Linux and Windows were not run locally: Linux relies on the executable's `malloc` symbols taking precedence, Windows keeps the system allocator for C code.

## How to change it

- Measure with `jaic x.jai -exe out -timings` on an idle machine, 5 or more warm runs (a busy process moves results 10 to 20%). `python3 tools/bench/main.py compile` ([benchmarks](../tools/benchmarks.md)) runs corpus projects, the compile-time-heavy programs in `benchmarks/compile/` and corpus-shaped programs from `tools/benchgen.py` (`gen-2k`/`gen-10k`/`gen-60k`).
- To change what the parse crawl follows, edit `Shared::collect` in `parse_ahead.rs`; keep it a guess (never let it change what sema loads). The bit-exactness test is `parse_ahead::tests::relocated_parse_equals_serial_parse`; for end-to-end checks compare `jaic` output with and without `JAIC_SERIAL_PARSE=1`. Parallel type checking could reuse the same parse cache.
- To compare ISel strategies, run `llc -O0 -global-isel=0` on `-emit_ir` output.
- Profile with samply ([benchmarks](../tools/benchmarks.md)). `Interp::run_code` is the main item left for metaprogram-heavy code, then `trace_enter`, then `build::call` and `write_item`.
- Ideas left: LiveDebugValues cost at `-O0`; load CSE for null-check facts; caching `type_has_member` misses for exported `using` values; `Span`-keyed type maps in `check` (`reserve_rehash`); interpreter speed.

## Configuration

- `JAIC_CODEGEN_UNITS=N`, `JAIC_VERIFY_IR`, `JAIC_DSYM`: see [LLVM backend](../native/llvm-backend.md#configuration) and [debug info](../native/debug-info.md).
- `-timings`: phase table on stderr.
- `JAIC_SERIAL_PARSE=1`, `JAIC_PARSE_THREADS=N`: see [parallel lex and parse](#parallel-lex-and-parse).
- `jaic-cli` feature `mimalloc`: mimalloc under the memory-limit counter (on with `dynamic-llvm` and `static-llvm`, see [Allocator](#allocator)).

## Dependencies

LLVM 23 through `inkwell`, and the system linker.
