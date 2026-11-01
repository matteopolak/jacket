# Compile speed

Where `jaic` spends time turning source into an executable, and the choices that keep it fast. `jaic -timings` prints the phases.

An `-O0` build is: front end (parse, sema, compile-time code), codegen (IR to LLVM to machine code, one thread per unit), then link. On a 10-core Apple machine, 60k generated lines take about 0.3 s and hello world 0.07 s.

## How it works

Choices that matter:

- **FastISel at `-O0`** (`use_fast_isel`, `jaic-llvm/src/lib.rs`). LLVM's default instruction selector dominated codegen.
- **Lazy declarations**: a codegen unit declares only the symbols it references. Look up symbols by name only after `declare_named`, and don't iterate the whole program per unit.
- **Units of 5,000 instructions** (`INSTS_PER_UNIT`). Total work matters more than balance.
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


### Rejected experiments

- Skipping LiveDebugValues at `-O0` loses `frame variable` locations.
- Removing the parse mutex in `split.rs` raises memory for no gain.
- Queuing a parsed file's imports at the front of the parse queue (depth first, as sema loads them) did not change how long `claim` waits.
- Comparing identifiers in the interner with word loads instead of `memcmp` was within noise.
- `-parse_threads` above 4 saves 2% on the generated program and nothing on real ones, so the default stays.
- Running `ld` directly instead of through `cc` saves about 10 ms and needs the SDK path re-derived.
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
