# C ABI, callbacks and C++ methods

How `jaic` passes C structs by value, variadic arguments and C++ method receivers across `#foreign` calls and `#c_call` callbacks. One classification drives both the interpreter's native foreign calls and the LLVM backend.

## How it works

`crates/jaic/src/abi.rs` (`classify_arg`, `classify_vararg`, `classify_ret`) classifies an `AggLayout` (size, alignment, flattened scalar fields) under one of four conventions, chosen by `Target::arch` for builds or `Arch::host()` for the interpreter.

| | AArch64 (`Aarch64`; `Win64Arm` with Windows variadics) | x86-64 System V (`X86_64`) | Microsoft x64 (`Win64`) |
|---|---|---|---|
| in registers | up to 16 bytes as `i64` chunks; float aggregates of up to 4 as floats | up to 16 bytes, per eightbyte | exactly 1, 2, 4 or 8 bytes as one `i64` |
| otherwise | caller copy, pointer passed | `byval` pointer | caller copy, pointer passed |
| return | chunks in registers, else `sret` | same | same |

- `agg_layout` (`sema/procs.rs`) drops repeated fields at the same offset and type, so overlaid members (`Vector2`'s `x, y` over `component: [2] float`) count once.
- `#no_padding` and `#align 1` members are C's packed attribute. System V puts an aggregate with a misaligned member in memory however small; the others look only at size.
- An aggregate needing more registers than remain goes entirely on the stack. On x86-64 `abi::RegBudget` counts registers in `lower_sig`; on AArch64 a multi-piece aggregate is one array parameter (`ParamPlan::AggArray`) that gets all its registers or none.
- Variadic calls (`args: ..Any` on a `#foreign` procedure) follow the platform's variadic rules (`classify_vararg`): Apple arm64 puts variadic arguments on the stack, Win64 duplicates doubles into integer registers. A struct passed through `...` travels by value.
- `sema/procs.rs` attaches a `CAbi` (`params`, `ret`, `ret_indirect`) to each `Conv::C` signature. The IR passes aggregates by pointer, so the boundary converts: `interp/native.rs` ([interpreter](../compiler/interpreter.md#native-foreign-calls)) and `bind_params` plus call lowering in the [LLVM backend](llvm-backend.md).
- Callback thunks in the interpreter are fixed-prototype `extern "C"` functions behind a naked entry stub. Results over 512 bytes pass their pointer explicitly.

### `long double`

[`Long_Double`](../language/long-double.md) is a 16-byte value with one `F80` (x87) or `F128` field. System V passes a lone `F80` in a 16-aligned stack slot and returns it in `st0`. AArch64 Linux treats `F128` as a float HFA member; MinGW x64 passes it by reference. `interp/native/wide.rs` uses inline-asm call blocks for these. Callbacks with these types are rejected, and variadic `long double` is a compile error.

### C++ methods

`#cpp_method` procedure types are `#c_call` (no `context`) with the object pointer first.

```jai
Vtable :: struct { add: #type (this: *Obj, x: s32) -> s32 #cpp_method; }
```

`#cpp_return_type_is_non_pod` on a foreign procedure forces a struct result through the hidden result pointer whatever its size (`CAbi::ret_indirect`). For vtable layout see [C++ vtables](cpp-vtables.md).

## How to change it

- New architecture or rule: extend `abi.rs` (it has unit tests), then `bind_params` and call lowering in `crates/jaic-llvm/src/lower.rs`, and `interp/native.rs`. Keep all three in step.
- To compare with clang: `clang --target=<triple> -S -emit-llvm structs.c` next to `jaic ... -target <same triple> -emit_ir out.ll`.
- Pieces become separate LLVM parameters (`f32, f32`, not `[2 x float]`).
- Small signed integers are not sign- or zero-extended per the C ABI, because the IR carries no signedness.
- Tests live in `crates/jaic-cli/tests/native.rs` with fixtures in `tests/native/` (`c-structs-by-value`, `c-aggregate-abi`, `c-variadic-aggregates`, `c-struct-returns`, `c-long-double`). They need `cc` (on Windows `clang` and `llvm-ar`).
