# C ABI, callbacks and C++ methods

## What it is

How `jaic` passes C structs by value, variadic arguments and C++ method receivers across `#foreign` calls and `#c_call` callbacks. One classification drives both the interpreter's native foreign calls and the LLVM backend.

## How it works

`crates/jaic/src/abi.rs` (`classify_arg`, `classify_vararg`, `classify_ret`) classifies an `AggLayout` (size, alignment, flattened scalar fields) for one of four conventions, picked by `Arch::from_triple` (builds) or `Arch::host()` (interpreter):

| `Arch` | Convention |
| --- | --- |
| `Aarch64` | AAPCS64 (Apple and Linux arm64) |
| `Win64Arm` | AAPCS64 with Windows variadic rules |
| `X86_64` | System V (Linux, macOS) |
| `Win64` | Microsoft x64 |

`agg_layout` (`sema/procs.rs`) flattens fields and drops repeats of the same offset and type, so overlaid members (`Math`'s `Vector2` with `component: [2] float` over `x, y`) count once: two floats to C, not four.

| | AArch64 | x86-64 System V | Microsoft x64 |
|---|---|---|---|
| in registers | up to 16 bytes as `i64` chunks; homogeneous float aggregates (up to 4) as separate floats | up to 16 bytes, per eightbyte | exactly 1, 2, 4 or 8 bytes as one `i64` |
| otherwise | caller copy, pointer passed | `byval` pointer | caller copy, pointer passed |
| return | chunks in registers, else `sret` | same | same |

- `#no_padding` structs and `#align 1` members are C's packed attribute. System V puts an aggregate with a misaligned member in MEMORY however small (`classify_registers` rejects it); AAPCS64 and Microsoft x64 look only at size.
- **Register exhaustion**: an aggregate needing more registers than remain goes entirely on the stack. On x86-64 `abi::RegBudget` counts registers in `lower_sig` and turns it into `byval`. On AArch64 a multi-piece aggregate is one array parameter (`ParamPlan::AggArray`) that gets all its registers or none, as Clang does; separate scalars would split between x7 and the stack.
- **Variadic calls** (`args: ..Any` on a `#foreign` procedure) use the platform's variadic convention: Apple arm64 puts variadic arguments on the stack, Win64 duplicates doubles into integer registers, Windows arm64 puts nothing in v registers (`classify_vararg`: anything up to 16 bytes is one or two `i64` pieces, larger by reference; an aggregate may straddle x7 and the stack). A struct passed through `...` travels by value (`c_vararg_layouts` in `sema/calls.rs`); the interpreter's `call_with` and the LLVM backend both use `classify_vararg`.
- **Where marshalling happens**: `sema/procs.rs` attaches a `CAbi` (`params`, `ret`, `ret_indirect`) to each `Conv::C` signature. The IR passes aggregates by pointer, so the boundary converts: `interp/native.rs` ([interpreter](../compiler/interpreter.md#native-foreign-calls)) and `bind_params` plus call lowering in the LLVM backend ([LLVM backend](llvm-backend.md)).
- **Struct return shapes in the interpreter**: every register shape the classifier produces works (one or two integer/SSE eightbytes, HFAs, `rax` results, hidden pointers). Callback thunks are fixed-prototype `extern "C"` functions that cannot see AArch64's `x8`, so those results use naked assembly stubs (`a64_sret_stub`). Results over 512 bytes (`SRET_WORDS`) pass their pointer explicitly (first integer argument on x86-64, `x8` via `wide.rs` on AArch64). "unsupported C aggregate return shape" remains only for piece lists the classifier cannot produce.

### `long double`

[`Long_Double`](../language/long-double.md) is a 16-byte value flattened to an `AggLayout` with one `Ty::F80` (x87) or `Ty::F128` field. x86-64 System V passes a lone `F80` in a 16-aligned stack slot and returns it in `st0` (any other aggregate with an `F80` is MEMORY). AArch64 Linux treats `F128` as a float HFA member. MinGW x64 passes it by reference. Plain `extern "C"` function pointers cannot express x87 or 128-bit vector registers, so `interp/native/wide.rs` uses inline-asm call blocks. Callbacks with these types are rejected and variadic `long double` is a compile error.

### C++ methods

`#cpp_method` procedure types are treated as `#c_call` (no `context`) with the object pointer first; reflection flag `0x1000` marks them.

```jai
Vtable :: struct { add: #type (this: *Obj, x: s32) -> s32 #cpp_method; }
```

`#cpp_return_type_is_non_pod` on a foreign procedure forces a struct result through the hidden result pointer whatever its size (`CAbi::ret_indirect`; `x8` on arm64, `rdi` on x86-64, `x0` on Windows arm64, via `sret` plus `inreg` in `lower_sig`).

## How to change it

- New architecture or rule: extend `abi.rs` (it has unit tests), then LLVM `bind_params` and call lowering in `crates/jaic-llvm/src/lower.rs`, and `interp/native.rs`. Keep them in step.
- Compare against clang: `clang --target=<triple> -S -emit-llvm structs.c` next to `jaic ... -target <same triple> -emit_ir out.ll`. Clang emits IR for any target without its libraries.
- Pieces become separate LLVM parameters (`f32, f32`, not `[2 x float]`); assignment matches except when an aggregate needs two registers and one is left.
- Small signed integers are not sign- or zero-extended per the C ABI, because the IR carries no signedness.
- Tests (in `crates/jaic-cli/tests/native.rs`, fixtures in `tests/native/`): `c-structs-by-value/`, `c-aggregate-abi/`, `c-variadic-aggregates/`, `c-struct-returns/` (58 shapes), `c-long-double/`, plus `tests/compiler/regressions/c-variadic-foreign-calls.jai` and `cpp-method-and-array-decay.jai`. They need `cc` (on Windows `clang` and `llvm-ar`).

## Dependencies

`crates/jaic/src/abi.rs`, `interp/native.rs`, `crates/jaic-llvm/src/lower.rs`; the system `cc` for native fixtures.
