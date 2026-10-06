# Low-level IR

## What it is

`crates/jaic/src/ir.rs` defines the one IR that `sema/lower.rs` produces and both the interpreter (`interp/`) and the LLVM backend (`crates/jaic-llvm`) consume. A whole compilation is one `ir::Program`.

## How it works

Values are scalars in virtual registers (`Val`, class `Ty`: `I8 I16 I32 I64 F32 F64 Ptr`; bools are `I8`), each defined once. Aggregates always live in memory. Locals, including mutable scalars, are stack `Slot`s, so there are no phi nodes; LLVM's mem2reg recovers SSA. Aggregate parameters and results are passed by pointer.

A `Func` is a list of `Block`s, each a list of `Inst` ending in one `Term` (`Jump`, `Branch`, `Switch`, `Ret`, `Unreachable`). Instructions cover constants, `Bin`/`Un`/`Cmp`/`Conv`, addresses (`SlotAddr`, `GlobalAddr`, `FuncAddr`, `ForeignAddr`), `Load`/`Store`, `PtrAdd`, `Copy`/`Zero`, `Call` (`Callee::Func`, `Foreign` or `Indirect`), `Intrinsic` (memcpy, bounds check, math, `CompilerWrite`, `IsCompileTime`, ...), and `Loc` markers for debug info and runtime error positions. `Loc::scope` indexes the function's debug scopes and is 0 without debug info.

`Call` and `Intrinsic` keep their operands in a box (`CallInst`, `IntrinsicInst`), and `Callee::Indirect` boxes its `Sig`, so an `Inst` is 24 bytes (asserted in `ir.rs`) rather than 120. Function bodies are long arrays of instructions, so this is a large share of the compiler's memory; keep new variants small or boxed too.

Calling conventions:

- `Conv::Jai`: optional context pointer, then each parameter (aggregates as a pointer to a caller-owned copy), then one out-pointer per aggregate result. Scalar results return directly.
- `Conv::C`: the platform C ABI. `Sig::c_abi` (`CAbi`, `AggLayout`) describes by-value structs; `c_varargs`/`c_fixed` mark variadic calls. See [C ABI](../native/c-abi.md).

`Program` holds:

- `funcs`, lowered on demand, so an entry is `None` until reachable;
- `globals`: initial bytes plus `Reloc`s for pointer slots. Type descriptors and `__runtime_info` are ordinary globals;
- `foreigns`, `libraries`;
- `reset_globals`: user globals whose compile-time state is discarded before `main` unless `#no_reset`;
- `file_paths`, `stack_trace_offset` (the `stack_trace` field's offset in `Context`) for stack traces;
- `debug_types`, `debug_globals`, and per-function `FuncDebug` side tables for [native debug info](../native/debug-info.md).

`Builder` is the construction API lowering uses (`new_block`, `slot`, `iconst`, `bin`, `call`, `finish`, ...).

## How to change it

- New instruction or intrinsic: add the variant in `ir.rs`, emit it from sema, and implement it in `interp/mod.rs` and `crates/jaic-llvm/src/lower.rs`. A backend that misses a variant fails to compile, which is the point.
- Keep aggregates in memory. Aggregate-valued registers would break both backends and the C ABI handling.
- `jaic build file.jai --emit-ir out.ll` writes the LLVM IR, the quickest way to inspect lowering.

## Configuration

`Options::stack_trace` and `Options::array_bounds_check` decide at lowering time whether stack-trace bookkeeping and bounds checks are emitted.

## Dependencies

Produced by `crates/jaic/src/sema`; consumed by `crates/jaic/src/interp` and `crates/jaic-llvm` (`emit_object(&Program, ...)`).
