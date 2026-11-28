# Compile speed

Where `jaic` spends time turning source into an executable, and the choices that keep it fast. `jaic -timings` prints the phases.

An `-O0` build is: front end (parse, sema, compile-time code), codegen (IR to LLVM to machine code, one thread per unit), then link. Optimized builds add the optimizer and split the program into modules first.

## How it works

Choices that matter at `-O0`:

- **FastISel** (`use_fast_isel`, `jaic-llvm/src/lib.rs`). LLVM's default instruction selector dominated codegen.
- **Narrow results are extended** so FastISel keeps the function: Jai-convention `i8`/`i16` results are `zeroext` (AArch64 FastISel selects a narrow return only when marked). C-convention results are not marked, because a C callee need not extend and the call site would assume it did. Find remaining fallbacks with `llc -O0 -fast-isel -fast-isel-report-on-fallback` on `-emit_ir` output; most are functions returning several values.
- **Lazy declarations**: a codegen unit declares only the symbols it references. Look up symbols by name only after `declare_named`, and don't iterate the whole program per unit.
- **Unit size** is 5,000 instructions (`INSTS_PER_UNIT`), up to three units per core (`UNITS_PER_CORE`). Total CPU is the same for any unit count, so wall time is parallel efficiency: on a machine with efficiency cores, equal shares per core leave the slow cores finishing last, and extra units let fast cores take more. A build with one thread keeps one unit.
- **Direct `ld` on macOS** for plain executable links ([native linking](../native/native-linking.md)).
- **Verifier off in release** (`-dev_verify_ir` turns it on), no LLVM value names, no `dsymutil` at `-O0` on macOS.
- **Cheap null checks** (`lower::NullFacts`): a compare and branch to a shared helper, skipped for a pointer already checked in the same block.
- **Constant data** becomes one `const_string`, not one `ConstantInt` per byte.
- **Small copies and fills** in unoptimized runtime code are emitted as loads and stores, because FastISel turns `memcpy` into a library call.
- **Leak at exit**: builds and `-check` forget the LLVM objects and the `Compiler` (`LeakOnExit`). `-interp` still drops.

Choices in the front end:

- **Real paths are remembered** (`SourceState::canon`); only paths that resolved are kept, since a metaprogram can write a file later.
- **Dead platform branches** of library files are not lexed or parsed (`lexer::lex_for_target`).
- **Missing module names are remembered** per `namespace_generation`, so repeated misses don't re-walk re-exports.

### Parallel lex and parse

`parse_ahead.rs` lexes and parses files on worker threads while sema runs. At the start it queues the files a compile loads first; workers discover further `#load`/`#import` paths as they parse.

Sema asks for a file through `ParseAhead::claim(canonical path)`:

- finished: it takes the parse and rewrites file ids in the tree (`ast/relocate.rs`);
- still being parsed: it waits;
- not predicted (a computed `#load`), a second instance of a file loaded into another module, or any failure: it parses inline. Parse errors always come from this path.

Output matches a serial run. File ids are assigned when sema claims a file, not when a worker finishes, because debug info, spans and the type table follow the id. The editor, non-native file systems and wasm always parse serially.

`-serial_parse` turns the workers off; `-parse_threads N` sets their number (both also in `[experimental]` of `jacket.toml`). Real programs wait only milliseconds in `claim`, so the default thread count stays.

### Name lookup in sema

Every file of a module writes its own `#import "Basic";` into the module scope, so a program of many files has hundreds of imports in one scope, and a lookup of a procedure's name (which keeps going to find overloads) asked all of them. Fixes that keep answers and diagnostics the same:

- **Each module once.** A scope with more than 16 imports gets an `import_plan` (`scope/lookup.rs`): the indices to ask, which are every filtered import and the first unfiltered import of each module. It is kept per `namespace_generation` in `NameCaches::import_plans`. The editor's unused-import lint records each import, so it keeps asking all.
- **A scope's imports holding no match is remembered** (`scope_misses`), whether or not the scope declares the name.
- Scopes with nothing pending (blocks and procedures) skip the "is this name final" pre-check.
- `warn_declarations_hiding_imports` asks each imported module once per name.
- `finalize_runtime_configuration` skips its walk over reachable IR when the temporary storage size is the default and nothing was captured at another size.
- `param_is_code` caches by procedure, type span and name.

### Lexer and interner

- **The interner hashes a spelling once.** The shards and each thread's cache are the same flat open-addressing `Table` (`intern.rs`) that stores the hash, so a cache miss, a shard hit and a grow never hash again. Slots come from the top bits of a multiply, because `FxHasher`'s low bits cluster under linear probing.
- `at_leading_dot_float` checks for `.digit` before scanning back over parentheses.
- A `Token` is 40 bytes: `Tok::Int` holds a `u64` (the lexer rejects larger literals), since a `u128` makes the enum 16-byte aligned.
- Punctuation is matched with one masked word compare; `FxHasher` reads a key's tail with two overlapping loads; identifiers, blank runs and line comments are scanned with slice loops.

### Interpreter speed

Compile-time code is mostly standard-library code, so its cost is interpreted instructions (`-profile`). Fixing a stdlib path often beats tuning the interpreter: `String_Builder` appends and `array_resize` clearing (one `memset` for plain-data elements) are examples. See [interpreter](interpreter.md#code-form).

### Debug builds: link

`ld` dominates the link. Most of it is reading `.tbd` files for the frameworks (`AppKit` pulls in `Foundation` and the private frameworks they re-export), and every framework on the line is needed. Going to `ld` without `cc` saves a few milliseconds. Measured and not worth it: `-dead_strip`, `-S`, `-no_deduplicate`, `-no_compact_unwind`, `-no_function_starts`, `-no_uuid`, `-fixup_chains`, `-ld_classic`, more or fewer objects, and `ld64.lld`. `-lm` and `libclang_rt.osx.a` stay on the direct line because `cc` passes them.

Debug info stays at full detail: line-tables-only loses `frame variable`, and skipping `LiveDebugValues` has no per-function switch and loses locations in functions that fell back to SelectionDAG.

### Optimized builds

No single optimizer pass dominates, and turning off vectorization, unrolling or function merging each moves it very little. What helps is doing less work and using idle cores:

- **Dead code is not lowered.** A split build lowers only what the entry points reach (`partition::classify`); code only compile-time execution ran is skipped.
- **Failure code is built without optimization.** Procedures with the `@Cold` note (`assert_helper`, the default assertion handler) and the check reporter, and what only they reach, form a module written at `-O0`.
- **Optimized builds are split before the optimizer** into up to `MAX_OPT_UNITS` (8) modules of about `OPT_INSTS_PER_UNIT` (20,000) instructions, each optimized on its own thread (`partition.rs`, [LLVM backend](../native/llvm-backend.md)). `-codegen_units 1` or `-no_split` restores whole-program optimization.
- **Machine code is split finer.** One optimized module's code generation is split by size at 3,500 instructions.

More modules compile faster but the programs run slower (measured on `jaison` and `jai-xml` from `benchmarks/runtime-projects`), and a larger import budget for each module did not recover it. So the two constants stay. Time inside the threaded build can't be had from `-time-passes` (a global lock); run `opt` and `llc` on a single-module `-emit_ir` file instead.

### Allocator

The global allocator is `memory_limit::CountingAllocator<A>`, which counts live bytes for `JAIC_MEMORY_LIMIT` and forwards to `A`. `A` is mimalloc through the `jaic-cli` feature `mimalloc`, on with both LLVM features. It is faster overall at the cost of some peak memory.

On macOS the `mimalloc` crate's `override` feature also replaces `malloc` and `free`, so LLVM's C++ allocations use mimalloc too. That registers a malloc zone whose `reinit_lock` hook is null, which crashed every `fork` (compile-time `run_command`). `jaic_llvm::fix_malloc_zone_for_fork` (`malloc_zone.rs`) patches it first thing in `main`.

### Ideas tried and not adopted

- Queuing `#load`/`#import` from the token stream right after lexing: it only helps the one workload that starves the parse workers and costs a little on the rest.
- A bump arena for worker allocations: every `dealloc` in the process then compares against the arena range, slowing the other workloads and the serial path, and dropped trees never return memory.
- Reserving the final `FileId` before parsing: it would make output depend on timing unless ids are reserved when the parent is declared, which leaves holes in the file table. `relocate` is small enough to keep.
- Cloning a second instance of a file instead of re-parsing it, and a per-thread interner: both cost about what they save.
- Removing the parse mutex in `split.rs`: more memory, no gain.
- Queuing a parsed file's imports at the front of the parse queue, and comparing identifiers in the interner with word loads: no change.
- Overlapping codegen with the front end: nothing in `Program` is final until the workspace completes, because metaprograms can still change it.
- Reusing objects between builds: not implemented. Source-module identity is insufficient for a library cache (generated specializations can depend on user types and constants, internal symbols contain program indices, and `partition::flat` balances the complete program). Keying the exact LLVM module after lowering keeps names and partitions, but a symbol-renumbering insertion misses every unit, it pays lowering, serialization and hashing on every build, and a prototype did not reach a repeatable 10% wall reduction on both nontrivial applications. A key must cover the compiler and LLVM identities, target/CPU/features, ABI, optimization, check and sanitizer settings, globals, relocations and debug data; reused objects must be copied to each output's retained object paths so macOS debug maps survive. `jaic_native::backend::stamp` checks ABI compatibility only. See [LLVM proof boundaries](../native/llvm-backend.md#additional-facts-proof-boundary).

## How to change it

- Measure with `jaic x.jai -exe out -timings` on an idle machine, five or more warm runs, alternating the old and new compiler and comparing medians. `PYTHONPATH=tools python3 tools/bench/main.py compile` runs corpus projects and `benchmarks/compile/` ([benchmarks](../tools/benchmarks.md)); `tools/benchgen.py --lines 600000 --files 120 --seed 1` makes a large generated program.
- Profile sema on one thread with inlined frames: build with `CARGO_PROFILE_RELEASE_DEBUG=1`, record with `xcrun xctrace record --template 'Time Profiler' --launch -- jaic -check -serial_parse main.jai` and symbolicate with `atos -i` (`sample` misses part of a short run). Otherwise use samply.
- To change what the parse crawl follows, edit `Shared::collect` in `parse_ahead.rs`, and keep it a guess that never changes what sema loads. `parse_ahead::tests::relocated_parse_equals_serial_parse` checks bit-exactness.
- Compare ISel strategies with `llc -O0 -global-isel=0` on `-emit_ir` output.
- Runtime performance must not regress: compare `benchmarks/runtime` and `runtime-projects` binaries built by each compiler, interleaved, and check the allocation counts of `tools/bench/gate.py`.

## Configuration

- `-codegen_units N`, `-dev_verify_ir`, `JAIC_DSYM`: see [LLVM backend](../native/llvm-backend.md#configuration) and [debug info](../native/debug-info.md).
- `-timings`, `-serial_parse`, `-parse_threads N`.
- `jaic-cli` feature `mimalloc`.

## Dependencies

LLVM 23 through `inkwell`, and the system linker.
