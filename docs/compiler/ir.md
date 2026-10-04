# Low-level IR

## What it is

`crates/jaic/src/ir.rs` defines the single IR that `sema/lower.rs` produces and that both the interpreter (`interp/`) and the LLVM backend (`crates/jaic-llvm`) consume. A whole compilation is one `ir::Program`.

## How it works

Values are scalars in virtual registers (`Val`, class `Ty`: `I8 I16 I32 I64 F32 F64 Ptr`; bools are `I8`), each defined once. Every aggregate lives in memory: locals are stack `Slot`s (mutable scalars too, so there are no phi nodes; LLVM's mem2reg recovers SSA), and aggregate parameters and results are passed as pointers.

A `Func` is a list of `Block`s, each a list of `Inst` ending in one `Term` (`Jump`, `Branch`, `Switch`, `Ret`, `Unreachable`). Instructions cover constants, `Bin`/`Un`/`Cmp`/`Conv`, address-of (`SlotAddr`, `GlobalAddr`, `FuncAddr`, `ForeignAddr`), `Load`/`Store`, `PtrAdd`, `Copy`/`Zero`, `Call` (`Callee::Func`, `Foreign` or `Indirect`), `Intrinsic` (memcpy, bounds check, math, `CompilerWrite`, `IsCompileTime`, ...) and `Loc` markers for debug info and runtime error positions.

Calling convention `Conv::Jai`: optional leading context pointer, then each parameter (aggregates by pointer to a caller-owned copy), then one out-pointer per aggregate result; scalar results return directly. `Conv::C` follows the platform C ABI; `Sig::c_abi` (`CAbi`, `AggLayout`) describes by-value structs, and `c_varargs`/`c_fixed` mark C variadic calls.

`Program` holds `funcs` (lowered on demand, so entries may be `None` until reachable), `globals` (initial bytes plus `Reloc`s for pointer slots), `foreigns`, `libraries`, `reset_globals` (user globals whose compile-time state is discarded at run time unless `#no_reset`), `file_paths` for stack traces and `stack_trace_offset` (the `stack_trace` field's offset in `Context`). Type descriptors and `__runtime_info` are ordinary globals.

`Builder` is the construction API used by lowering (`new_block`, `slot`, `iconst`, `bin`, `call`, `finish`, ...).

## How to change it

- New instruction or intrinsic: add the variant in `ir.rs`, lower it in `sema/lower.rs` (or wherever the construct is checked), then implement it in both `interp/mod.rs` and `crates/jaic-llvm/src/lower.rs`. A backend that misses a variant fails to compile, which is the point.
- Keep aggregates in memory; introducing aggregate-valued registers would break both backends and the C ABI handling.
- `jaic build file.jai --emit-ir out.ll` writes the LLVM IR produced from this program, which is the quickest way to inspect lowering results.

## Configuration

None for the IR itself. Stack traces and bounds checks are decided at lowering time by `Options::stack_trace` and `Options::array_bounds_check`.

## Dependencies

Consumed by `crates/jaic/src/interp` and `crates/jaic-llvm` (`emit_object(&Program, ...)`); produced by `crates/jaic/src/sema`.
