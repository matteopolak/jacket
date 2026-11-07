# Compile speed

Where `jaic` spends time turning source into an executable, and the choices that keep it fast. `jaic -timings` prints the phases.

An `-O0` build is: front end (parse, sema, compile-time code), codegen (IR to LLVM to machine code, one thread per unit), then link. On a 10-core Apple machine, 60k generated lines take about 0.3 s and hello world 0.07 s.

## How it works

Choices that matter:

- **FastISel at `-O0`** (`use_fast_isel`, `jaic-llvm/src/lib.rs`). LLVM's default instruction selector dominated codegen.
- **Lazy declarations**: a codegen unit declares only the symbols it references. Look up symbols by name only after `declare_named`, and don't iterate the whole program per unit.
- **Units of 5,000 instructions** (`INSTS_PER_UNIT`), up to three per core (`UNITS_PER_CORE`). Total work matters, and so does balance on a Mac with efficiency cores; see below.
- **Narrow results are extended** so FastISel keeps the function (`zeroext` on `i8`/`i16` Jai results, AArch64).
- **Direct `ld` on macOS** for plain executable links ([native linking](../native/native-linking.md)).
- **Verifier off in release** (`-dev_verify_ir` turns it on), no LLVM value names, no `dsymutil` at `-O0` on macOS.
- **Cheap null checks** (`lower::NullFacts`): a compare and branch to a shared helper, skipped for a pointer already checked in the same block.
- **Real paths are remembered** (`SourceState::canon`); only paths that resolved are kept, since a metaprogram can write a file later.
- **Dead platform branches** of library files are not lexed or parsed (`lexer::lex_for_target`).
- **Missing module names are remembered** per `namespace_generation`, so repeated misses don't re-walk re-exports.
- **Constant data** becomes one `const_string`, not one `ConstantInt` per byte.
- **Leak at exit**: builds and `-check` forget the LLVM objects and the `Compiler` (`LeakOnExit`). `-interp` still drops.
- **Optimized builds are split before the optimizer** into up to 8 parts, each optimized on its own thread (`partition.rs`, [LLVM backend](../native/llvm-backend.md)). `-codegen_units 1` or `-no_split` restores whole-program optimization.
- **Unoptimized runtime**: small copies and fills are emitted as loads and stores, because FastISel turns `memcpy` into a library call.

### Parallel lex and parse

`parse_ahead.rs` lexes and parses files on worker threads while sema runs. At the start it queues the files a compile loads first; workers discover further `#load`/`#import` paths as they parse.

Sema asks for a file through `ParseAhead::claim(canonical path)`:

- finished: it takes the parse and rewrites file ids in the tree (`ast/relocate.rs`);
- still being parsed: it waits;
- not predicted (a computed `#load`), a second instance, or any failure: it parses inline, as before. Parse errors always come from this path.

Output matches a serial run. File ids are assigned when sema claims a file, not when a worker finishes. The editor, non-native file systems and wasm always parse serially.

`-serial_parse` turns the workers off; `-parse_threads N` sets their number (both also in `[experimental]` of `jacket.toml`).

### Interpreter speed

Compile-time code is mostly standard-library code, so its cost is interpreted instructions (`-profile`). Fixing a stdlib path often beats tuning the interpreter: `String_Builder` appends and `array_resize` clearing (one `memset` for plain-data elements) cut one benchmark by two thirds. See [interpreter](interpreter.md#code-form).

### Name lookup in sema

On a 600k-line generated program a third of `-check` was `Compiler::lookup_full`. Every file of a module writes its own `#import "Basic";` into the module scope, so a program of 120 files has about 500 imports there, and a lookup of a procedure's name (which keeps going to find overloads) asked all of them. The fixes, each keeping the answer and the diagnostics the same:

- **Each module once.** A scope with more than 16 imports gets an `import_plan` (`scope/lookup.rs`): the indices to ask, which are every filtered import and the first unfiltered import of each module. It is kept per `namespace_generation` in `NameCaches::import_plans`. Smaller scopes skip repeats inline. The editor's unused-import lint records each import, so it keeps asking all.
- **A scope's imports holding a name is remembered even when the scope declares it** (`scope_misses`): it records that the imports, `using`s and implicit modules add nothing, whether or not `found` was empty.
- Scopes with nothing pending (blocks and procedures) no longer pay for the "is this name final" pre-check.
- `warn_declarations_hiding_imports` asks each imported module once per name instead of once per import.
- `finalize_runtime_configuration` skips its walk over all reachable IR when the temporary storage size is the default and nothing was captured at another size.
- `param_is_code` caches by procedure, type span and name; every call asked it of every parameter of every candidate through a full constant evaluation.

Numbers (median of 9, `-check -serial_parse`, same machine, alternating runs): 600k generated lines 1.48 s to 1.12 s; with parallel parse 1.21 s to 0.85 s. Real code is dominated by parsing and file reads, and moves by 2-4% (all of `stdlib` with `-no_dce`, 0.38 s to 0.37 s). What is left in sema is flat: the biggest items are the remaining scope walk of `lookup_full` (about 9% on the generated program), first-time `signature` and `resolve_entity` work, and constant evaluation's per-call `FnCtx` (`thunk_ctx`, about 1.5%).

To profile sema on one thread with inlined frames, build with `CARGO_PROFILE_RELEASE_DEBUG=1`, record with `xcrun xctrace record --template 'Time Profiler' --launch -- jaic -check -serial_parse main.jai` and symbolicate with `atos -i`; `sample` misses part of a short run.

### Lexer, interner and parse-ahead

Lex and parse are about a quarter of a serial `-check` of the 600k-line generated program (20 MB, 5.9M tokens), so the lexer is tuned on its own (an in-process harness that lexes every file on a fresh thread and takes the minimum of 21 rounds). What changed:

- **The interner hashes a spelling once.** The shards and each thread's cache are the same flat open-addressing `Table` (`intern.rs`) that stores the hash, so a miss in the cache, a hit in a shard and a grow never hash again. Slots are picked from the top bits of a multiply: `FxHasher`'s low bits clustered under linear probing and made it slower than the maps it replaced. A thread's first lex of a file was paying for three hashes and repeated rehashes of its cache (every parse worker starts cold): 162 ms to 105 ms for the 120 files.
- **`at_leading_dot_float` checks for `.digit` first.** It ran for every punctuation token and scanned back over parentheses (`closes_cast`) before looking at the byte.
- **A `Token` is 40 bytes, not 48.** `Tok::Int` holds a `u64` (the lexer rejects larger literals); a `u128` made the whole enum 16-byte aligned.
- **Punctuation is matched with one masked word compare** instead of a `memcmp` call per candidate.
- **`FxHasher` reads a key's last one to seven bytes with two overlapping loads**, not a variable-length copy that called `memcpy` for every identifier.
- **Identifiers, blank runs and line comments are scanned with slice loops.**

Together lexing the 120 files went from 125 ms to 75 ms (warm) and lex plus parse from 289 ms to 240 ms; the parser itself is unchanged at about 165 ms. On the whole compile, medians of 11 alternating runs, `-check`:

| Program | serial before | serial after | default before | default after |
| --- | --- | --- | --- | --- |
| 600k generated lines | 1.02 s | 0.93 s | 0.78 s | 0.77 s |
| most of `stdlib`, `-no_dce` | 0.195 s | 0.190 s | 0.163 s | 0.159 s |
| `examples/game2d/game` | 0.137 s | 0.132 s | 0.110 s | 0.106 s |

**Parse-ahead is not the bottleneck in real programs.** Timing every `claim`: the stdlib program waits 10 ms in total (of 150 ms), `game2d` 2 ms; relocation costs 3 ms. Only the generated program, with 120 files of 5,000 lines asked for within 100 ms of each other, waits (55 ms, 8%), because four workers cannot parse that fast; 8 workers saved 2%. About 9% of claims come back empty because the file is loaded a second time into another module (a new `FileId` and `AstId`s, so it is parsed again on the sema thread). A claimed file stays in the queue and the worker that pops it skips it, instead of an O(queue) scan.

What is left is mostly spread thin: `Lexer::push` and `skip_trivia` (about 6% and 7% of lex plus parse), allocation and freeing of AST nodes (about 10%), `Table::find`, and the parser's lookahead scans (`tagged_member_ahead`, `paren_starts_header`, about 4% together). Five larger redesigns were tried or bounded and none is worth it; see below.

**Larger redesigns, measured.** Release builds, best of 10 (20 where marked) alternating runs of the old and new `jaic`, `-check` unless noted, load average 2.5 to 3.5 (one 10-run pass reached 3.7 at its start). The workloads are the 600k generated lines (`tools/benchgen.py --lines 600000 --files 120 --seed 1`), `examples/game2d/game`, `examples/ui-gallery` and `benchmarks/compile/stdlib-imports.jai`.

| Redesign | Result | 600k generated | game2d | ui-gallery | stdlib |
| --- | --- | --- | --- | --- | --- |
| Queue `#load`/`#import` from the token stream, right after lexing | rejected | 0.698 s to 0.685 s (-1.8%); full build -3.0% | 0.096 s to 0.097 s (+1.7%) | +0.8% | 0.0318 s to 0.0339 s (+6.7%, median +3.2%, 20 runs) |
| Bump arena for worker allocations (global allocator, free is a no-op) | rejected | 0.707 s to 0.695 s (-1.7%, median -0.7%); `-serial_parse` +2.5% | +1.8% | +1.6% | +2.8% |

- **Early discovery.** The token scan only sees imports outside braces, so it queues what parsing would have queued a moment later. It helps the one workload that starves the workers (168 ms of 840 ms waiting in `claim`) and costs 1 to 2 ms on the others (the second scan, the second `canonical`, and a different queue order). Real programs wait 5 to 10 ms in total, so there is little to win.
- **Arena.** Worker parses go faster on the big program, but every `dealloc` in the process now compares the pointer with the arena range, which costs 2 to 3% on the other workloads and on the serial path. Dropped trees would also never return memory.
- **Final `FileId` up front.** `relocate` costs 17 to 27 ms of the generated program (2 to 3%) and at most 2 ms elsewhere. A file's id is its position in sema's load order, and debug info, spans and the type table follow it, so a worker that guessed the id would make output depend on timing. Reserving ids when a parent is declared would be deterministic, but workers are far ahead of sema and have usually parsed the file already, and unused reservations leave holes in the file table. Not built.
- **Clone a second instance.** Re-parsing a file loaded into another module (two files per workload) takes 0.2 to 0.7 ms in total, under 1.3% on every workload; a clone would need `Clone` plus an `AstId` rewrite and cost part of that. Not built.
- **Per-thread interner.** A worker takes a shard lock only when its own cache has not seen a spelling: 181k times for the generated program, 29 ms summed over five threads including the timer's own cost, so under 1% of wall time. Numbering locally would add a translation walk over every tree, the same size as `relocate`, which is more than the locks cost. Not built.


### Debug build: codegen and link

`examples/game2d/game` at `-O0` (340k IR instructions, 10-core Apple machine with 4 performance and 6 efficiency cores) took 300 ms: front end 107, codegen 108, link 75. Numbers are best of 13 alternating runs of the old and new `jaic` at load average 8 to 10 (a background macOS service and another build); a quiet machine shows larger gains for the unit change.

| Program | total before | total after | codegen | link |
| --- | --- | --- | --- | --- |
| hello world | 55 ms | 46 ms | 9 to 9 | 26 to 17 |
| `game2d` | 326 ms | 296 ms | 103 to 93 | 79 to 72 |
| `ui-gallery` | 190 ms | 175 ms | 34 to 33 | 88 to 80 |
| `editor` | 256 ms | 236 ms | 75 to 62 | 90 to 83 |
| generated 60k lines | 293 ms | 253 ms | 125 to 100 | 33 to 24 |

Peak memory of `game2d` went from 383 MB to 326 MB (smaller modules).

**Where codegen time goes.** One unit for all of `game2d` took 300 ms of CPU: building LLVM IR on the Rust side 43 ms (14%) and LLVM's code generator 257 ms. The total CPU is the same for 1, 10 or 40 units (0.78 s to 0.90 s of user time), so wall time is parallel efficiency. With equal shares per core, the units on efficiency cores finished last (per unit: lower 40 to 65 ms, emit 40 to 50 ms, in parallel, against 5 to 9 ms and 20 to 25 ms for units that started later). Three units per core let the fast cores take more of them: `game2d` codegen 108 ms at 10 units, 92 at 20, 85 at 30 to 60. Hello world (4 units) did not change at 6, 8 or 10. A build with one thread keeps one unit.

**FastISel fallbacks.** `llc -O0 -fast-isel -fast-isel-report-on-fallback` on the `-emit_ir` output showed 953 of 2,751 functions in `game2d` going through SelectionDAG. 876 of the first-missed instructions were `ret i8 ...`: AArch64's FastISel only selects a narrow return when the result is marked `zeroext` or `signext`. Marking Jai-convention results `zeroext` leaves 174 fallbacks, 13% of the IR, nearly all functions that return several values (`ret { i64, i64 }`, `extractvalue` of a call). Instruction selection fell from 1.73 G to 1.16 G instructions and a one-unit build from 297 to 281 ms (-5%). C-convention results are not marked: a C callee need not extend, and the call site would assume it did.

**Link.** `ld` alone takes 67 of the 75 ms. A trivial program links in 10 ms; `game2d`'s ten objects add about 15; the rest is reading `.tbd` files for the frameworks (`AppKit` pulls in `Foundation`, whose `.tbd` is 5.6 MB, and the private frameworks they re-export): with `-framework AppKit` the link is 69 ms, without it 25. The frameworks are all needed (every one that still links without it is re-exported by AppKit, so dropping it from the command saves nothing). `ui-gallery` links a large static `libwgpu_native.a`. Going to `ld` without `cc` saves 5 to 8 ms, 15% of a hello world.

**Measured and rejected (debug builds).**

- `ld` flags `-dead_strip` (+2 ms), `-S`, `-no_deduplicate`, `-no_compact_unwind`, `-no_function_starts`, `-no_uuid`, `-fixup_chains`, `-ld_classic`: within 1 ms of the default.
- Fewer or more objects: link time at 10, 30 and 60 objects is 82, 78 and 81 ms (noise).
- `-lm` and `libclang_rt.osx.a` on the `ld` line: no measurable cost, but `cc` passes them, so the direct command does too.
- `ld64.lld` (Homebrew LLD) on `game2d`: 63 ms against 69 for Apple's `ld`, the same `.tbd` parsing; not installed by default.
- Skipping `LiveDebugValues`: 13% of LLVM's time, mostly spent on functions with nothing to track, but no option turns it off per function; `--livedebugvalues-input-dbg-value-limit=0` leaves the object byte-identical. Functions that fell back to SelectionDAG need it for locations in later blocks.
- Line tables only: `opt -passes=strip-nonlinetable-debuginfo` before `llc` saves 6%, and `-no_debug_info` 33% of codegen CPU. Both lose `frame variable` or the line tables backtraces use, so debug info stays.
- `-codegen_units` above 30 for `game2d`: no further gain (85, 86, 85 ms at 30, 40, 60).

**Object reuse is not implemented.** Source-module identity is insufficient for a library cache: generated specializations can depend on user types and constants, internal symbols contain program indices, and `partition::flat` balances the complete program. A module boundary needs a canonical dependency closure and stable symbols before it can be reused independently. Library IR percentage alone does not measure saved wall time.

An alternative is to key the exact LLVM module **after lowering**. This keeps existing names and partitions: an insertion that renumbers symbols causes a miss, while unchanged units can still match. It pays lowering, serialization and hashing on every build; at `-O0` it can save only machine-code emission. A key before the optimizer can also save optimization, but must cover the complete pipeline and imported bodies. Neither design can reuse objects from source timestamps alone, or skip frontend/metaprogram finalization.

Any implementation must include the actual compiler/backend and LLVM identities, resolved target/CPU/features, ABI, optimization/check/sanitizer settings, globals/relocations and debug data in its contract. `jaic_native::backend::stamp` checks ABI compatibility; it is not a full codegen identity. Reused objects must be copied to each output's retained object paths so macOS debug maps survive cache eviction. Compare the current compiler, cache-disabled overhead, cold writes and representative edit sequences before adopting either design. See [LLVM proof boundaries](../native/llvm-backend.md#additional-facts-proof-boundary).

### Exact-unit reuse experiment

A private prototype keyed each exact lowered LLVM module before optimization. It verified cached object hashes and copied objects to the current output paths, keeping the normal partition, optimizer and linker. Disabled, cold and warm runs used the same binary. The prototype did not establish the predeclared gate of a repeatable 10% wall reduction on both nontrivial applications, so no cache implementation was retained.

The test used an Apple M5 with 16 GiB, macOS 27.0, AC power and the LLVM dylib identified by SHA256 prefix `c644c75bd471` (full identity and tool version are retained with the private experiment evidence). The release compiler was based on `49da19c88` plus the private probe (binary SHA256 starts `7d2bbb72c856`). Each row has two warmup pairs and nine alternating measured pairs. Source and output paths stayed fixed, with a fresh user-body literal each round. Both sides used `-timings`; filesystem reads were warm, with no system-cache purge.

Cold means a fresh empty object cache; warm means entries from prior edits. The ratio is the median of paired enabled/disabled wall times, so it can differ from the ratio of the column medians. Lower is better.

| Mode | Program | Cache | Disabled median (ms) | Enabled median (ms) | Paired ratio (95% bootstrap interval) |
| --- | --- | --- | ---: | ---: | --- |
| O0 | game2d | warm | 473 | 394 | 0.825 (0.641–0.954) |
| O0 | ui-gallery | warm | 250 | 648 | 2.400 (0.496–3.503) |
| O0 | game2d | cold | 397 | 704 | 1.773 (1.570–2.243) |
| O0 | ui-gallery | cold | 479 | 606 | 2.086 (0.502–2.768) |
| O2 | game2d | warm | 1065 | 836 | 0.797 (0.689–0.947) |
| O2 | ui-gallery | warm | 910 | 1021 | 1.422 (0.933–1.667) |
| O2 | game2d | cold | 1069 | 1266 | 1.229 (0.995–1.258) |
| O2 | ui-gallery | cold | 1119 | 879 | 0.813 (0.641–1.597) |

These are qualified observations, not accepted speedups. The replacement O0 warm block passed its boundary quiet checks but had large link-time excursions. The O0 cold, O2 warm and O2 cold blocks exceeded the post-load guard (2.83, 2.73 and 4.97 against 2.5). Compilation itself can raise load, and boundary snapshots cannot rule out transient interference. An earlier O0 block with busy indexing was excluded in full; its raw rounds were preserved.

The intervals are imprecise with nine pairs. UI gallery did not show the required benefit, and cold costs were uncertain or unfavorable.

The namespace was precomputed, so timings exclude identity acquisition. Separate Python SHA256 measurements of the compiler and LLVM dylib on warm files took medians of 4.15 ms and 56.27 ms. An immutable embedded identity might amortize that cost, but was not implemented or validated. Per-unit serialization/hash and read/verification/materialization intervals overlap; their sums cannot predict saved wall time.

Fresh body edits reused 28/30 and 24/26 O0 units for game2d and UI, and 7/8 and 5/6 at O2. Adding a genuinely called early procedure shifted symbol indices and missed every O0 unit; removing it recovered most hits. These matches are useful evidence for exact-unit reuse, not proof of a reusable library dependency closure.

All 18 disabled/cold/warm smoke builds succeeded; hello executed identically, matching O0 objects were byte-identical, and the O2 dSYMs passed structural verification with matching executable UUIDs. GUI execution, debugger behavior after eviction, complete invalidation, other supported targets, concurrent writers, corruption recovery and bounded storage were not validated. The rejected prototype does not establish a no-go for canonical library artifacts or other designs with different lowering, keying and materialization costs; those need their own proof and cache-disabled/cold/edit measurements.

### Optimized builds

`game2d` at `-O2` (10-core Apple machine): front end 108 ms, codegen 770 ms, link 87 ms, debug info 45 ms. Per module (8 of them, balanced to within 10%): lowering 70 ms, the optimizer 270-390 ms, machine code 300-370 ms. LLVM's own timing of the whole program on one thread: InstCombine 15%, the inliner 8%, SROA 6%; instruction selection is 38% of machine code. No pass stands out, and turning off loop or SLP vectorization, unrolling or merging functions each moved the optimizer by under 5%. What helped was doing less work and using idle cores:

- **Dead code is not lowered.** About 15% of the IR of `game2d` is code only compile-time execution ran. A split build now lowers only what the entry points reach (`partition::classify`): lowering 72 ms to 57 ms per module, and the modules are balanced on real work.
- **Failure code is built without optimization.** Procedures with the `@Cold` note (`assert_helper`, the default assertion handler) and the check reporter, and what only they reach, form a module written at `-O0`. An empty `main` carried 22,000 IR instructions, 90% of it the `print` machinery behind `assert`.
- **Machine code is split finer.** One optimized module's code generation is split by size at 3,500 instructions, not 10,000. Measured on a 13,000-instruction module, whole build: 1 unit 291 ms, 2: 238, 3: 224, 4: 220, 6: 228, 8: 230, 10: 236.

Best of 9 alternating builds, load average 6-7 from another job (so absolute numbers are higher than on an idle machine; the pairs are what matter), total and codegen in ms:

| Program | before total (codegen) | after total (codegen) |
| --- | --- | --- |
| `main :: () {}` | 317 (250) | 85 (22) |
| `print` and recursion, 13 lines | 320 (258) | 227 (162) |
| `examples/game2d/game` | 963 (726) | 864 (636) |
| `examples/ui-gallery` | 525 (305) | 507 (287) |
| 10,000 generated lines | 313 (222) | 309 (217) |
| 2,500 generated lines | 344 (268) | 344 (268) |

The runtime benchmarks (`benchmarks/runtime`, `runtime-projects`, 15 runs, one binary built by each compiler, interleaved) are within noise: ratios 0.98x to 1.03x, with `jaison` 1.02x (range 0.385-0.513 for both). The allocation counts of `tools/bench/gate.py` are unchanged.

**More modules before the optimizer: measured and rejected.** Compile time keeps improving with the count (`game2d` codegen, 7 runs each: 8 modules 684 ms, 9: 631, 10: 635, 20: 618; a 20,000-instruction program, 9 runs: 1 module 249 ms, 2: 126, 4: 69, 8: 54), but the programs run slower. Building the runtime projects with `-codegen_units` and taking the minimum of 25 runs against the default: `jaison` 1.03x at 2, 1.07x at 4 and 8; `jai-xml` 1.00x at 2, 1.03x at 4 and 8. A larger import budget (40% or 100% of a module, copies up to 400 or 1,000 instructions) did not recover it. So `OPT_INSTS_PER_UNIT` (20,000) and `MAX_OPT_UNITS` (8) stay. Other things measured: `llc -fast-isel` at `-O2` cuts machine code time by a quarter and makes the code 2.4% larger, not tried; an inline threshold of 100 cuts the optimizer by 15% and the code by 15%, a runtime risk not taken; `-time-passes` inside the threaded build is useless (a global lock), measure a single-module `-emit_ir` file with `opt` and `llc` instead.

### Rejected experiments

- Skipping LiveDebugValues at `-O0` loses `frame variable` locations.
- Removing the parse mutex in `split.rs` raises memory for no gain.
- Queuing a parsed file's imports at the front of the parse queue (depth first, as sema loads them) did not change how long `claim` waits.
- Comparing identifiers in the interner with word loads instead of `memcmp` was within noise.
- `-parse_threads` above 4 saves 2% on the generated program and nothing on real ones, so the default stays.
- Overlapping codegen with the front end: nothing in `Program` is final until the workspace completes, because metaprograms can still change it.

### Allocator

The global allocator is `memory_limit::CountingAllocator<A>`, which counts live bytes for `JAIC_MEMORY_LIMIT` and forwards to `A`. `A` is mimalloc through the `jaic-cli` feature `mimalloc`, on with both LLVM features. It is about 8% faster overall at 4-8% more peak memory.

On macOS the `mimalloc` crate's `override` feature also replaces `malloc` and `free`, so LLVM's C++ allocations use mimalloc too. That registers a malloc zone whose `reinit_lock` hook is null, which crashed every `fork` (compile-time `run_command`). `jaic_llvm::fix_malloc_zone_for_fork` (`malloc_zone.rs`) patches it first thing in `main`.

## How to change it

- Measure with `jaic x.jai -exe out -timings` on an idle machine, five or more warm runs. `PYTHONPATH=tools python3 tools/bench/main.py compile` runs corpus projects and `benchmarks/compile/` ([benchmarks](../tools/benchmarks.md)).
- To change what the parse crawl follows, edit `Shared::collect` in `parse_ahead.rs`, and keep it a guess that never changes what sema loads. `parse_ahead::tests::relocated_parse_equals_serial_parse` checks bit-exactness.
- Compare ISel strategies with `llc -O0 -global-isel=0` on `-emit_ir` output. Profile with samply.

## Configuration

- `-codegen_units N`, `-dev_verify_ir`, `JAIC_DSYM`: see [LLVM backend](../native/llvm-backend.md#configuration) and [debug info](../native/debug-info.md).
- `-timings`, `-serial_parse`, `-parse_threads N`.
- `jaic-cli` feature `mimalloc`.

## Dependencies

LLVM 23 through `inkwell`, and the system linker.
