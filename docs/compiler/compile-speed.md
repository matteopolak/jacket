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

### Rejected experiments

- Skipping LiveDebugValues at `-O0` loses `frame variable` locations.
- Removing the parse mutex in `split.rs` raises memory for no gain.
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
