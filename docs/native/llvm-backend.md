# LLVM backend

`crates/jaic-llvm` translates the `jaic::ir::Program` into LLVM IR and writes object files; `crates/jaic-native` links them with the system `cc` ([native linking](native-linking.md)). A release `jaic` loads `jaic-llvm` from a library beside it on the first native build ([lazy backend](../compiler/lazy-backend.md)). `jaic file.jai` uses it; `-interp` uses the interpreter and `-check` only type-checks.

## How it works

`emit_object` in `lib.rs`:

1. Builds a target machine for the host or `Options::target`. The CPU is the oldest one the triple runs on (`baseline_cpu`), not the build machine's, because release archives are built on one runner and run on others. `Build_Options.llvm_options.target_system_cpu`/`target_system_features` override it; `"native"` targets the build machine.
2. `lower::lower_program` declares every function, foreign symbol and global, fills initialisers, then defines bodies.
3. The module is verified (debug builds or `-dev_verify_ir`), optionally optimised with `default<On>`, and written out.

At `-O0` the target machine uses FastISel, because GlobalISel was a third of codegen time on AArch64. jaic sets no process-wide LLVM options; if a bug needs one, set it once in `target_machine` and note the upstream issue.

### Codegen units

`emit_objects` splits a large program into units, one LLVM context and module per thread.

- Count: one for sanitized builds, wasm, `-emit_ir` and `output_*` options. Otherwise `-codegen_units`, else one per 5,000 IR instructions (20,000 optimised, at most 8), capped at the core count. A huge unoptimised program gets more units than threads (up to 64) so modules are freed as they finish and peak memory stays low.
- Unoptimised: functions go largest first to the least loaded unit. Optimised: divided by call graph (below).
- Each module defines its own functions and declares what it uses. A lookup by name (`libc_call`, `llvm.*`) must call `declare_named` first or LLVM renames the later symbol and the link fails. Internal symbols another unit uses become hidden external; use `Backend::internal_linkage` for new ones.
- Objects are `path`, `path.1.o`, ...; the CLI links and deletes them. `-o x.o` stays one module.

**Dividing before the optimizer** (`partition.rs`). The optimizer is most of an `-O2` build and serial for one module, so a program of two or more units is divided first and each part runs the whole pipeline on its own thread.

- A function or global with exactly one referrer hangs under it, and the forest is cut where a group exceeds its share. Most callers stay in their module, so single-caller inlining and constant propagation still work.
- A small function of another module (at most 200 IR instructions, plus a budget of 12% of the module) is copied as `available_externally` for inlining. `inline` functions are copied everywhere. Small read-only globals are copied too.
- The cost is a few percent more instructions in search-heavy code, none in numeric code. Raise `IMPORT_MAX` or the budget to trade compile time for inlining.
- `inline_cuts` stops one driver function (`main`, a test runner) from absorbing the program: past `INLINE_CAP` (4,000 estimated instructions) its largest inlinable callees are marked `noinline`.

**Splitting after the optimizer** (`split.rs`). A program too small to divide is optimised as one module, then machine code generation runs in parallel (one unit per 10,000 instructions, at most 4). `strip_body` must delete whole blocks, not instructions one at a time, or debug records land on the wrong function and LLVM crashes now and then.

### Lowering rules (`lower.rs`)

- Every function has `"frame-pointer"="non-leaf"` so frame-pointer stack walks see jaic frames.
- Each IR `Val` is an SSA value; blocks are emitted so the likely successor falls through, and check failure blocks go to the end. `Slot`s are allocas in the first block. Pointers are opaque.
- Integer semantics match the interpreter: division by zero traps, `INT_MIN / -1` wraps, shifts of `>= bits` give 0, float-to-int saturates.
- Globals are packed structs of byte runs and relocations. Every foreign call is an indirect call typed from its `Sig`.
- Unoptimised code inlines small copies, skips guards for constant divisors, and drops null checks it has already proven in the same block (`NullFacts`). A new instruction that writes a slot, changes a pointer or lets a slot address escape must update or reset the facts. See [compile speed](../compiler/compile-speed.md).
- `Conv::C` signatures use the real C convention for aggregates: [C ABI](c-abi.md).

### SIMD lowering (`simd/`)

Turns `ByteLookup` and `SignMask` ([intrinsics](../language/intrinsics.md#native-lowering-of-the-simd-intrinsics)) into the best instruction the target has, with the same result everywhere. `x86.rs` uses `pshufb` (SSSE3/AVX2/AVX-512BW), `aarch64.rs` uses NEON `tbl1`, `wasm.rs` swizzle, and `generic.rs` is the fallback and reference. Each dispatch tries backends in order; a backend returns `Ok(false)` if it does not apply. `Target::resolve` reads the real CPU name and feature string, so `+x`/`-x` features work. SVE has no backend yet.

## How to change it

- New IR instruction or intrinsic: `Backend::inst`/`Backend::intrinsic` in `lower.rs`, with semantics identical to `interp/mod.rs`.
- New architecture: an `Arch` variant and classification in `abi.rs`, plus inline-asm intrinsics in `lower.rs`.
- New instruction set: `Feature` variants and `cpu_features` in `simd/features.rs` with a test, then the path in the architecture file guarded by `cx.target.has(..)`, more specific first.
- New SIMD intrinsic: the `ir::Intrinsic` variant with sema and interpreter parts, then `generic::<name>`, a dispatch function and a call in `lower.rs`. A mistyped LLVM intrinsic fails at lowering, but one the target cannot select fails only in codegen: check with `-emit_ir` and build an object.
- Debug info is in `debuginfo.rs`; `lower.rs` only calls its hooks.
- Module-level things (a global, a constructor list) are emitted once in unit 0. A new way for lowered code to refer to another function or global must be added to `partition::references`.
- Testing units: CI runs `JACKET_TEST_CODEGEN_UNITS=4 cargo test -p jaic-cli --test native --test optimized_split --test debug_info --test embedded_data`. With several units `-emit_ir` writes `x.n.ll` per unit; tests reading IR must read every part (`read_ir_parts`) or pin `-codegen_units 1`. `-dev_split_units 4` forces the post-optimizer split.
- Windows: `#program_export` is `dllexport` ([Windows](windows.md)). WebAssembly: foreign procedures become imports ([wasm target](wasm-target.md)).

## Configuration

- `-dev_verify_ir`, `-codegen_units N` (`-no_split` is 1, for whole-program optimisation), `-dev_split_units N`, `[experimental] codegen_units` in `jacket.toml`.
- Constants: `INSTS_PER_UNIT`, `OPT_INSTS_PER_UNIT`, `MAX_OPT_UNITS` (`lib.rs`), `MAX_UNITS` (`split.rs`), `IMPORT_MAX`, `IMPORT_BUDGET_PERCENT`, `INLINE_CAP`, `INLINE_SMALL` (`partition.rs`).
- `-O0..-O3`, `-emit_ir file.ll`, `-no_debug_info`, `-sanitize` ([sanitizers](sanitizers.md)), `-os`, `-target triple`.
- Building needs `llvm-sys` (or `LLVM_PREFIX`) pointing at the pinned LLVM ([LLVM setup](../tools/llvm-setup.md)).

## Dependencies

`inkwell`, the system `cc` for linking, `jaic` for the IR and `jaic::abi::Arch`. SIMD tests: `cargo test -p jaic-llvm simd`.
