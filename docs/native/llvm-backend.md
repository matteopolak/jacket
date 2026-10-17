# LLVM backend

## What it is

`crates/jaic-llvm` translates the `jaic::ir::Program` into LLVM IR, writes native object files, and links them with the system `cc`. The CLI exposes it as `jaic <file.jai> [-o out]`; `jaic -interp` uses the interpreter and `jaic -check` only type-checks.

## How it works

`emit_object(program, options, path)` in `lib.rs`:

1. Creates a target machine for the host or `Options::target` (e.g. `x86_64-pc-windows-gnu` for `-os windows`). The CPU is the oldest one the triple runs on (`baseline_cpu`: `x86-64`, `apple-m1` for arm64 macOS, else `generic`), never the build machine's, because release archives are built on one CI runner and run on others (an AVX-512 build once died with SIGILL elsewhere). `Build_Options.llvm_options.target_system_cpu` / `target_system_features` override it; `"native"` targets the build machine (`cpu_and_features`).
2. `lower::lower_program` declares every lowered function, foreign symbol and global, fills in initialisers, then defines function bodies.
3. Verifies the module (debug builds, or `JAIC_VERIFY_IR=1`), optionally runs the `default<On>` pipeline, and writes the object.

At `-O0` the target machine uses FastISel (`use_fast_isel`): LLVM enables GlobalISel by default for unoptimised AArch64 and it was a third of codegen time. jaic sets no process-wide LLVM options. If an LLVM bug needs one, set it once before the first target machine (in `target_machine`), document the upstream issue, and remove it after moving past the fix (the LLVM 22 `-unroll-add-parallel-reductions=false` workaround was removed for LLVM 23; the corpus case `unrolled-sub-reduction` still guards it).

### Codegen units

`emit_objects` splits a large program into units, one LLVM context and module per thread.

- Unit count: one for a sanitized build, else `JAIC_CODEGEN_UNITS` if set. Otherwise unoptimised builds get one per 5,000 IR instructions (`INSTS_PER_UNIT`) and optimised ones one per 20,000 (`OPT_INSTS_PER_UNIT`), capped at the core count (and 8 when optimised, `MAX_OPT_UNITS`). `-emit_ir`, sanitizers, `output_*` options and wasm always use one. `enable_split_modules = false` stops only the unoptimised split.
- Unoptimised: functions go largest first to the least loaded unit (`partition::flat`); unit 0 also defines the globals. Optimised: divided by call graph (below).
- Each module defines its own functions and declares only what it uses (`lower::Shard`, `Backend::func/global/foreign`). A lookup by name (`libc_call`, `__multi3`, `llvm.*`) must first call `declare_named` or LLVM renames the later symbol (`write.1`) and the link fails. Internal symbols another unit refers to become hidden external (`name.index`); use `Backend::internal_linkage` for new ones.
- Objects are `path`, `path.1.o`, `path.2.o`, ...; the CLI links them and deletes them (unoptimised macOS builds with debug info keep them, see [debug info](debug-info.md)). `-o x.o` stays one module.

#### Dividing before the optimizer (`partition.rs`)

The optimizer is most of an `-O2` build and serial for a one-module program, so a program of at least two units is divided first and each part runs the whole pipeline and codegen on its own thread. Splitting with every symbol external saved little and slowed the code; what works:

- **Grouping by referrer** (`partition::plan`): a function or global with exactly one referrer hangs under it; the forest is cut where a group exceeds 1/(2 x units) of the program. Most functions have all callers in their own module and stay `internal` there, so single-caller inlining, dead code removal and constant propagation still work.
- **Copies for inlining** (`Plan::imports`): a function of another module with at most 200 IR instructions (`IMPORT_MAX`; plus a budget, `IMPORT_BUDGET_PERCENT`) is also defined as `available_externally`. `#program_export` functions are not copied. Read-only globals up to 16 KiB are imported the same way; an all-zero read-only global without pointers gets a private copy per module (`partition::replicated`) so the optimizer can turn a copy into `memset` (otherwise a default struct value added 2.8 MB to an executable).
- Cost: about 1% to 3% more instructions in a search-heavy program, none in numeric code. Raise `IMPORT_MAX` to trade compile time for inlining.

#### Keeping one function from absorbing the program (`partition::inline_cuts`)

The inliner takes any single-caller function bottom up, so a driver (`main`, `init`, a test runner) can grow to tens of thousands of IR lines, and loop passes (`LoopFullUnroll`, `LoopVectorize`, ...) are worse than linear in function size. On a 120k-line program one unit took 24 s where the others took 6 s, and the call-graph division cannot help because the function is one node. `inline_cuts` runs once before lowering and estimates inlined size (a callee counts at each site when internal with one caller, `inline`, or at most `INLINE_SMALL` = 50 instructions); when a function passes `INLINE_CAP` (4,000) its largest inlinable callees are marked `noinline`. `no_inline` and `inline` procedures stay as declared; `-O0` skips it.

#### Splitting after the optimizer (`split.rs`)

An optimized program too small to divide is optimized as one module, then `split::emit` runs machine code generation in parallel: one unit per 10,000 LLVM instructions (`MAX_UNITS` = 4). Internal definitions become hidden external, each extra unit's thread parses the bitcode into its own `Context`, turns other units' bodies into declarations and writes its object. Applies whatever `enable_split_modules` says, without `-emit_ir`, a sanitizer or `JAIC_CODEGEN_UNITS`.

Gotchas: `strip_body` must delete whole blocks, not erase instructions one at a time. An erased instruction's debug records move into a context-wide table keyed by block address; a later block allocated at the same address picks up another function's variables and `DwarfDebug` crashes, about one build in four. Parsing and cutting down are serialized under a mutex to bound peak memory. `Target::initialize_all` runs once behind a `Once`; calling it concurrently crashed LLVM's registry.

### Lowering rules (`lower.rs`)

- Every function has `"frame-pointer"="non-leaf"` so frame-pointer stack walks (macOS `backtrace`, profilers) see jaic frames.
- Each IR `Val` is an SSA value; blocks are emitted in reverse post-order. `Slot`s become allocas in a first `allocas` block. Pointers are opaque; `PtrAdd` is an `i8` GEP.
- Integer semantics match the interpreter: division by zero traps, `INT_MIN / -1` wraps, shifts of `>= bits` give 0, float-to-int saturates.
- Globals are packed structs of byte runs and relocations; `#program_export` names stay external. Every foreign call is an indirect call with a type built from its `Sig`, so one symbol can be called with several signatures.
- Intrinsics: `Memcpy` is `memmove`; `CompareAndSwap` is seq_cst `cmpxchg`; `CycleCounter` reads `cntvct_el0` / `rdtsc`; `Syscall` is inline asm on x86-64 only.
- **Unoptimised code**: `Copy`/`Zero` of up to 64 bytes are loads and stores (`inline_copy`), constant divisors and shifts skip guards (`bin`), a compare's `i8` is widened only where a non-branch uses it (`get`). Null checks (`null_check`) branch to a trap block calling `jaic.null_fail`; `FnState.facts` (`NullFacts`) remembers checked pointers per IR block. Any new instruction that writes a slot, changes a pointer or lets a slot address escape must update or reset the facts. Contexts discard value names, and `emit_objects` leaks modules and machines at exit. See [compile speed](../compiler/compile-speed.md).
- **C ABI**: `Conv::C` signatures with `Sig::c_abi` use the real convention for by-value aggregates ([C ABI](c-abi.md)). Call sites copy into a scratch temp, load pieces and pass them separately; variadic calls use a vararg type with only the declared parameters (`Sig::c_fixed`). `#c_call` definitions do the reverse in `bind_params`.

### SIMD lowering (`crates/jaic-llvm/src/simd/`)

Turns `ir::Intrinsic::ByteLookup(width)` and `SignMask(width)` ([intrinsics](../language/intrinsics.md#native-lowering-of-the-simd-intrinsics)) into the best instruction the target has; every target computes the same result.

```
simd/  mod.rs (Cx helpers, one dispatch per intrinsic)  features.rs (Feature/FeatureSet/Target)
       x86.rs (pshufb: SSSE3/AVX2/AVX-512BW)  aarch64.rs (NEON tbl1)  wasm.rs (swizzle)  generic.rs (fallback and reference semantics)
```

Each dispatch function tries the backends in order; a backend returns `Ok(true)` once it emitted the operation and `Ok(false)` if its architecture or features do not apply, so `generic` always ends the chain. `Target::resolve(arch, cpu, features)` uses the CPU name and feature string of the actual `TargetMachine`: the CPU gives a starting set (`cpu_features`, the one table, keyed by LLVM CPU names; unknown CPUs get the architecture baseline), `+x` adds a feature and what it implies, `-x` removes it and everything implying it, unknown or other-architecture names are ignored. The macOS x86-64 baseline `core2` has SSSE3; the Linux/Windows `x86-64` baseline does not.

To add an instruction set: add `Feature` variants in `features.rs` (`ALL`, `name`, `family`, `implies`) and CPUs in `cpu_features` with a test, add the path in the architecture file guarded by `cx.target.has(..)` (reuse `Cx::map_blocks`, `call`, `repeat_table`, `splat`), more specific first. To add an intrinsic: the `ir::Intrinsic` variant and sema/interpreter parts, then `generic::<name>` first, a dispatch function and a call in `lower.rs`. An LLVM intrinsic with a wrong name is caught at lowering, but one the target cannot select fails only in codegen: check with `-emit_ir` and by building an object. SVE has no backend yet.

## How to change it

- **New IR instruction or intrinsic**: `Backend::inst` / `Backend::intrinsic` in `lower.rs`, with semantics identical to `interp/mod.rs`.
- **New architecture**: an `Arch` variant and classification in `abi.rs`, plus inline-asm intrinsics in `lower.rs`.
- **Debug info** is in `debuginfo.rs`; `lower.rs` only calls its hooks.
- **Module-level things** (a global, a constructor list) are emitted once in unit 0 and declared elsewhere. A new way for lowered code to refer to another function or global (beyond `GlobalAddr`, `FuncAddr`, `ForeignAddr`, calls) must be added to `partition::references`.
- **Testing units**: CI runs `JAIC_CODEGEN_UNITS=4 cargo test -p jaic-cli --test native --test optimized_split --test debug_info --test embedded_data`. With several units `-emit_ir` writes `x.n.ll` per unit; tests reading IR must read every unit (`read_ir_parts`) or pin `JAIC_CODEGEN_UNITS=1`. Force the post-optimizer split with `JAIC_SPLIT_UNITS=4`. A declaration made from a definition must lose its body, personality and `!dbg` (`strip_body`) or the verifier rejects it.
- **Windows** (`Arch::Win64`): `#program_export` is `dllexport`, `CompilerWrite` calls `_write` ([Windows](windows.md)). **WebAssembly** (`Arch::Wasm64`): foreign procedures become imports, `#program_export` gets `wasm-export-name`, a weak `__multi3` is emitted ([wasm target](wasm-target.md)).

## Configuration

- `JAIC_VERIFY_IR=1` runs the verifier in a release jaic (`0` disables it in debug).
- `JAIC_CODEGEN_UNITS=N` forces the unit count; `1` turns all splitting off for whole-program optimisation. `JAIC_SPLIT_UNITS=N` forces the post-optimizer count.
- Constants: `INSTS_PER_UNIT`, `OPT_INSTS_PER_UNIT`, `MAX_OPT_UNITS` (`lib.rs`); `INSTS_PER_UNIT`, `MAX_UNITS` (`split.rs`); `IMPORT_MAX`, `IMPORT_BUDGET_PERCENT`, `GLOBAL_IMPORT_MAX`, `INLINE_CAP`, `INLINE_SMALL` (`partition.rs`).
- `jaic_llvm::Options { opt_level, target, emit_ir, debug_info, sanitize, cpu, features }`, from `-O0..-O3`, `-emit_ir file.ll`, `-no_debug_info`, `-sanitize` ([sanitizers](sanitizers.md)), `-os`, `-target triple`. Wasm always gets `+bulk-memory`; `+simd128` enables the swizzle.
- Building needs `LLVM_SYS_231_PREFIX` pointing at LLVM 23 ([LLVM setup](../tools/llvm-setup.md)).

## Dependencies

`inkwell` (LLVM 23), the system `cc` for linking ([native linking](native-linking.md)), `jaic` for the IR and `jaic::abi::Arch`. SIMD tests: `cargo test -p jaic-llvm simd` and `tests/stdlib/intrinsic-byte-lookup.jai`.
