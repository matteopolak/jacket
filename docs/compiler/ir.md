# Low-level IR

`crates/jaic/src/ir.rs` defines the one IR that `sema/lower.rs` produces and both the interpreter (`interp/`) and the LLVM backend (`crates/jaic-llvm`) consume. A whole compilation is one `ir::Program`.

## How it works

- Scalars live in virtual registers, each defined once (`I8 I16 I32 I64 F32 F64 Ptr`; bools are `I8`). Aggregates always live in memory. Locals are stack `Slot`s, so there are no phi nodes; LLVM's mem2reg recovers SSA.
- A `Func` is `Block`s of `Inst`s, each ending in one `Term` (`Jump`, `Branch`, `Switch`, `Ret`, `Unreachable`).
- `Conv::Jai` passes an optional context pointer, then parameters (aggregates by pointer to a caller-owned copy), then one out-pointer per aggregate result. `Conv::C` is the platform ABI ([C ABI](../native/c-abi.md)).
- `Program` holds lazily lowered functions, globals (bytes plus relocations), foreigns, the globals reset before `main`, the stack-trace layout and [debug info](../native/debug-info.md) tables.
- `Func::trusted` lists values never null (context pointer, stack-trace addresses, incoming aggregate parameters) so native code skips null checks. `Func::inline` becomes `alwaysinline` / `noinline`.

## How to change it

- New instruction or intrinsic: add the variant in `ir.rs`, emit it from sema, implement it in `interp/` and `crates/jaic-llvm/src/lower.rs`. A backend missing a variant fails to compile.
- Keep `Inst` small (24 bytes, asserted); function bodies are a large share of compiler memory. Box big operands.
- Keep aggregates in memory; aggregate-valued registers would break both backends and the C ABI.
- `jaic file.jai -emit_ir out.ll` writes the LLVM IR, the quickest way to inspect lowering.
