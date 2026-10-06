# C ABI, callbacks and C++ methods

## What it is

How `jaic` passes C structs by value, variadic arguments and C++ method receivers across `#foreign` calls and `#c_call` callbacks. One classification drives both the interpreter's native foreign calls (`jaic run`) and the LLVM backend (`jaic build`).

## How it works

`crates/jaic/src/abi.rs` (`classify_arg`, `classify_vararg`, `classify_ret`) classifies an `AggLayout` (size, alignment, flattened scalar fields, from `ir.rs`) for one of four conventions:

| `Arch` | Convention | Picked for |
| --- | --- | --- |
| `Aarch64` | AAPCS64 | Apple and Linux arm64 |
| `Win64Arm` | AAPCS64 with Windows variadic rules | any `windows`/`mingw` aarch64 triple |
| `X86_64` | System V | Linux and macOS x86-64 |
| `Win64` | Microsoft x64 | any `windows`/`mingw` x86-64 triple |

`Arch::from_triple` picks one for a build, `Arch::host()` for the interpreter.

How aggregates travel:

| | AArch64 | x86-64 System V | Microsoft x64 |
|---|---|---|---|
| in registers | up to 16 bytes as `i64` chunks in x registers; homogeneous float aggregates (HFAs, up to 4 members) as separate `float`/`double` | up to 16 bytes, per eightbyte: `i64`, `double`, `float` or `<2 x float>` | exactly 1, 2, 4 or 8 bytes as one `i64`, even when the members are floats |
| otherwise | caller copy, pointer passed | `byval` pointer | caller copy, pointer passed |
| return | chunks in registers, else `sret` | same | same |

On Microsoft x64, scalars, the shadow space, stack slots past the fourth argument and variadic doubles duplicated in integer registers are LLVM's job once the function type is right.

### Variadic calls

`args: ..Any` on a `#foreign` procedure uses the platform's variadic convention. It matters on Apple arm64 (variadic arguments go on the stack), Win64 (variadic doubles duplicated in integer registers) and Windows arm64.

On Windows arm64 a variadic procedure puts nothing in v registers, for all of its arguments, fixed ones included. `float`/`double` travel as bits in x0-x7 and then on the stack (LLVM does this from the variadic function type), and aggregates are not HFAs: `classify_vararg` makes anything up to 16 bytes one or two `i64` pieces and passes larger ones by reference. An aggregate may straddle x7 and the stack. Results are unaffected. Clang's IR for the same C, which the unit tests mirror:

```
// struct F4 { float a, b, c, d; };  struct D4 { double a, b, c, d; };
declare %struct.F4 @take_f4([4 x float])          ; fixed: HFA in s0-s3
call i32 (i32, ...) @vf(i32 1, double 1.5, [2 x i64] %f4, ptr %d4_copy)   ; variadic
```

The LLVM backend uses `classify_vararg` for every parameter of a variadic signature; the interpreter's `call_with` does the same on a Windows arm64 host (`general_only`).

### Where marshalling happens

`sema/procs.rs` attaches a `CAbi` (`params: Vec<Option<AggLayout>>`, `ret`, `ret_indirect`) to each `Conv::C` signature. The IR itself passes aggregates by pointer, so marshalling happens at the boundary:

- Interpreter: `interp/native.rs` loads register pieces from the aggregate and calls the function pointer; see [interpreter](../compiler/interpreter.md#native-foreign-calls). Callbacks from C into interpreted procedures go through `interp/native/callbacks.rs` (`callbacks/win64.rs` on Windows x64).
- LLVM: the call site and `bind_params`; see [LLVM backend](llvm-backend.md#c-abi).

### `long double`

[`Long_Double`](../language/jaic-extensions.md) is a 16-byte memory-class value flattened into an `AggLayout` with one `Ty::F80` (x87) or `Ty::F128` (binary128) field, so it follows the aggregate rules:

- x86-64 System V: an aggregate that is exactly one `F80` is `Registers([PieceTy::X87])`: passed in a 16-aligned stack slot, returned in `st0`. Any other aggregate containing an `F80` is MEMORY (`byval`/`sret`), matching clang.
- AArch64 Linux: `F128` counts as a float member of an HFA (up to four `q` registers, `PieceTy::F128`).
- MinGW x64: 16 bytes, so by reference and `sret`.

Plain `extern "C"` function pointers can't express x87 or 128-bit vector registers, so `interp/native.rs` hands those calls to `interp/native/wide.rs`: inline-asm call blocks that load the argument registers and stack words, call the target, and store `st0` (`fstp tbyte`) or `q0`-`q3` back. Callbacks with these types are rejected (`callbacks.rs`), and variadic `long double` arguments are a compile error.

### C++ methods

`#cpp_method` procedure types are treated as `#c_call` (no `context`) with the object pointer first (`sema/procs.rs`, `sema/expr.rs`); reflection flag `0x1000` marks them (`sema/code_export.rs`).

```jai
Vtable :: struct { add: #type (this: *Obj, x: s32) -> s32 #cpp_method; }
add_impl :: (this: *Obj, x: s32) -> s32 #c_call { return this.base + x; }
// vt.add(*o, 2) == 42
```

`#cpp_return_type_is_non_pod` on a foreign procedure forces a struct result through the hidden result pointer whatever its size (`CAbi::ret_indirect`; on Windows arm64 that pointer goes in x0, as MSVC does, not x8). The LLVM backend does the same in `lower_sig` (an `sret` parameter, plus `inreg` on Windows arm64 to select x0); `cpp_non_pod_results_use_the_hidden_pointer` in `crates/jaic-cli/tests/native.rs` covers it.

## How to change it

- New architecture or rule: extend `abi.rs` (it has unit tests), then the LLVM `bind_params` and call lowering in `crates/jaic-llvm/src/lower.rs`, and `interp/native.rs`. Keep them in step.
- Compare against clang: `clang --target=x86_64-pc-windows-msvc -S -emit-llvm structs.c` (or `aarch64-pc-windows-msvc`, `arm64-apple-macos`, ...) next to `jaic build ... -target <same triple> --emit-ir out.ll`. Clang emits IR for any target without its libraries, so this works on any host.
- Pieces become separate LLVM parameters (`f32, f32`, not clang's `[2 x float]`). Register assignment matches except when an aggregate needs two registers and only one is left: AAPCS64 puts all of it on the stack, while separate parameters would split it.
- Small signed integers are not sign- or zero-extended per the C ABI, because the IR doesn't carry signedness.

Tests:

- `tests/native/c-structs-by-value/` (`c_structs_by_value` in `crates/jaic-cli/tests/native.rs`) compares interpreter and native output and calls `#c_call` callbacks from C. It needs `cc` (on Windows `clang` and `llvm-ar`, native output only). `tools/windows_cross.py` builds it for Windows with MinGW GCC or Clang.
- `tests/stdlib/c-variadic-foreign-calls.jai` and `c_variadic_calls`.
- `tests/native/c-long-double/` (`c_long_double`, which also runs an x86-64 build under Rosetta on Apple silicon).
- `tests/stdlib/cpp-method-and-array-decay.jai`.

## Dependencies

`crates/jaic/src/abi.rs`, `interp/native.rs`, `crates/jaic-llvm/src/lower.rs`; the system `cc` for the native fixtures.
