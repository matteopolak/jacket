# Low-level IR

## What it is

`crates/jaic/src/ir.rs` defines the one IR that `sema/lower.rs` produces and both the interpreter (`interp/`) and the LLVM backend (`crates/jaic-llvm`) consume. A whole compilation is one `ir::Program`.

## How it works

Values are scalars in virtual registers (`Val`, class `Ty`: `I8 I16 I32 I64 F32 F64 Ptr`; bools are `I8`), each defined once. Aggregates always live in memory. Locals, including mutable scalars, are stack `Slot`s, so there are no phi nodes; LLVM's mem2reg recovers SSA. Aggregate parameters and results are passed by pointer.

A `Func` is a list of `Block`s of `Inst`s ending in one `Term` (`Jump`, `Branch`, `Switch`, `Ret`, `Unreachable`). Instructions cover constants, `Bin`/`Un`/`Cmp`/`Conv`, addresses, `Load`/`Store`, `PtrAdd`, `Copy`/`Zero`, `Call` (`Func`, `Foreign` or `Indirect`), `Intrinsic`, and `Loc` markers for debug info and runtime error positions.

`Call` and `Intrinsic` box their operands, so an `Inst` is 24 bytes (asserted). Function bodies are a large share of compiler memory; keep new variants small or boxed.

Calling conventions: `Conv::Jai` takes an optional context pointer, then each parameter (aggregates as a pointer to a caller-owned copy), then one out-pointer per aggregate result; scalar results return directly. `Conv::C` is the platform ABI (`Sig::c_abi`, see [C ABI](../native/c-abi.md)).

`Program` holds lazily lowered `funcs`, `globals` (bytes plus `Reloc`s; type descriptors and `__runtime_info` are ordinary globals), `foreigns`, `reset_globals` (user globals reset before `main` unless `#no_reset`), `stack_trace` layout, and debug side tables for [native debug info](../native/debug-info.md). `Builder` is the construction API used by lowering.

## How to change it

- **New instruction or intrinsic**: add the variant in `ir.rs`, emit it from sema, implement it in `interp/` and `crates/jaic-llvm/src/lower.rs`. A backend that misses a variant fails to compile, which is the point. Fiber intrinsics are described in [interpreter](interpreter.md#fibers).
- Keep aggregates in memory; aggregate-valued registers would break both backends and the C ABI handling.
- `jaic file.jai -emit_ir out.ll` writes the LLVM IR, the quickest way to inspect lowering.

## Configuration

- `Func::trusted` lists values that are never null (context pointer, addresses the [stack trace](stack-traces.md) pass computes, incoming aggregate parameter addresses); native code does not null-check through them. Without the last, an inlined struct-taking procedure keeps a check per access and LLVM stops vectorizing its loops.
- `Func::inline` (`InlineHint`) becomes `alwaysinline` / `noinline`.
- `Options::stack_trace` and `Options::array_bounds_check` decide at lowering whether stack-trace bookkeeping and bounds checks are emitted.

## Dependencies

Produced by `crates/jaic/src/sema`; consumed by `crates/jaic/src/interp` and `crates/jaic-llvm`.
