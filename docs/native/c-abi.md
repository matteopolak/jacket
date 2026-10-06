# C ABI, callbacks and C++ methods

## What it is

How `jaic` passes C structs by value, variadic arguments and C++ method receivers across `#foreign` calls and `#c_call` callbacks. The same classification drives the interpreter's native foreign calls (`jaic run`) and the LLVM backend (`jaic build`).

## How it works

Classification lives in `crates/jaic/src/abi.rs` (`classify_arg`, `classify_vararg`, `classify_ret`, `Arch::{Aarch64, X86_64, Win64, Win64Arm}`). It only looks at an `AggLayout` (size, alignment, flattened scalar fields) from `crates/jaic/src/ir.rs`. Four conventions are implemented: AArch64 AAPCS64 (Apple and Linux), its Windows variant (`Win64Arm`, any `windows`/`mingw` aarch64 triple), x86-64 System V and the Microsoft x64 convention (`Win64`, any `windows`/`mingw` x86-64 triple). `Arch::from_triple` picks one for a build, `Arch::host()` for the interpreter.

Microsoft x64 (`Win64`): an aggregate of exactly 1, 2, 4 or 8 bytes is passed and returned as one integer (`i64` piece) in a general register, even when its members are floats (`struct { float x, y; }` travels in RCX/RAX, not XMM). Every other size is passed by reference to a caller-made copy (`Passing::Indirect`) and returned through a hidden `sret` pointer. Scalars, the 32-byte shadow space, stack slots beyond the fourth argument and variadic doubles duplicated in integer registers are LLVM's job once the function type is right. The interpreter calls foreign procedures with it through `interp/native/windows.rs`, but C cannot call back into interpreted procedures there (`callback_addr` returns an error for `Win64`). See [Windows](windows.md).

Windows on arm64 (`Win64Arm`) is AAPCS64 for fixed-argument procedures: up to 16 bytes in one or two general registers, homogeneous float aggregates (HFAs, up to four `float`/`double` members, even 32 bytes) in v registers, anything larger by reference to a caller copy, and large results through the hidden pointer in x8 (LLVM's `sret`). A *variadic* procedure is different, and for all of its arguments, fixed ones included: nothing goes in v registers. Scalars `float`/`double` travel as their bits in x0-x7 and then on the stack (LLVM does this from the variadic function type), and aggregates are not HFAs: `classify_vararg` makes any aggregate up to 16 bytes one or two `i64` pieces and passes larger ones by reference; an aggregate may straddle x7 and the stack. Results are unaffected (a variadic procedure still returns an HFA in v0-v3). Clang's IR for the same C, which the unit tests mirror:

```
// struct F4 { float a, b, c, d; };  struct D4 { double a, b, c, d; };
declare %struct.F4 @take_f4([4 x float])          ; fixed: HFA in s0-s3
call i32 (i32, ...) @vf(i32 1, double 1.5, [2 x i64] %f4, ptr %d4_copy)   ; variadic
```

Each `Conv::C` signature may carry a `CAbi` (`params: Vec<Option<AggLayout>>`, `ret`, `ret_indirect`) which `sema/procs.rs` fills in from the procedure header. The IR itself passes aggregates by pointer; marshalling happens at the boundary:

- Interpreter: `interp/native.rs` loads register pieces from the aggregate and calls the function pointer; large results use a hidden result pointer.
- LLVM: see [LLVM backend](llvm-backend.md) for the call-site and definition-side rules.

Variadic C calls (`args: ..Any` on a `#foreign` procedure) are passed with the platform's variadic convention, which matters on Apple arm64 where variadic arguments go on the stack, on Win64 where variadic doubles are duplicated in integer registers, and on Windows arm64 where every argument goes to general registers (above). The LLVM backend uses `classify_vararg` for every parameter of a variadic signature; the interpreter's `call_with` does the same on a Windows arm64 host (`general_only`). Regression program: `tests/stdlib/c-variadic-foreign-calls.jai`; native test `c_variadic_calls` in `crates/jaic-cli/tests/native.rs`.

C `long double` ([`Long_Double`](../language/jaic-extensions.md)) is a 16-byte, memory-class value in the IR, flattened into an `AggLayout` with one `Ty::F80` (x87) or `Ty::F128` (binary128) field, so it goes through the aggregate rules:

- x86-64 System V: an aggregate that is exactly one `F80` (a bare `long double` or `struct { long double v; }`) is classed `Registers([PieceTy::X87])`: passed in memory (a 16-aligned stack slot) and returned in x87 `st0`. Any other aggregate containing an `F80` is MEMORY (byval / sret), which matches clang's `x86_fp80` and `byval` lowering.
- AArch64 (Linux): `F128` counts as a floating member of a homogeneous aggregate (up to four `q` registers, `PieceTy::F128`); a bare `long double` is one `q` register.
- Microsoft x64 with MinGW (x87, 16 bytes): falls under "every other size", so by reference / sret.

The interpreter cannot express x87 or 128-bit vector registers with plain `extern "C"` function pointers, so `interp/native.rs` hands such calls to `interp/native/wide.rs`: inline-asm call blocks that load the integer and vector argument registers and the stack words, call the target, and store `st0` (`fstp tbyte`) or `q0`-`q3` back. Callbacks from C into interpreted code with these types are rejected (`callbacks.rs`). Variadic `long double` arguments are a compile error. Fixture: `tests/native/c-long-double/` (test `c_long_double`, which also runs an x86-64 build under Rosetta on Apple silicon).

`#cpp_method` procedure types are treated as `#c_call` (no implicit `context`) with the object pointer as first argument (`sema/procs.rs`, `sema/expr.rs`). The reflection flag `0x1000` is set for them (`sema/code_export.rs`). Verified in `tests/stdlib/cpp-method-and-array-decay.jai`:

```jai
Vtable :: struct { add: #type (this: *Obj, x: s32) -> s32 #cpp_method; }
add_impl :: (this: *Obj, x: s32) -> s32 #c_call { return this.base + x; }
// vt.add(*o, 2) == 42
```

`#cpp_return_type_is_non_pod` on a foreign procedure makes a struct result always go through the hidden result pointer whatever its size (`CAbi::ret_indirect`, set in `sema/procs.rs`, honored in `interp/native.rs`; on Windows arm64 that pointer goes in x0, as MSVC does, rather than x8). The LLVM backend does not read `ret_indirect` today, so this marker only affects `jaic run`.

## How to change it

- New architecture or rule: extend `abi.rs` (it has unit tests), then the LLVM `bind_params`/call lowering in `crates/jaic-llvm/src/lower.rs` and the interpreter in `interp/native.rs`; keep both in step.
- Small signed integers are not sign/zero-extended per the C ABI because the IR does not carry signedness.
- Fixture: `tests/native/c-structs-by-value/` (`structs.c` plus Jai files) is exercised by the `c_structs_by_value` test, which compares interpreter and native output and also calls `#c_call` Jai callbacks from C. It needs `cc` (on Windows `clang` and `llvm-ar`, native output only). `tools/windows_cross.py` builds the same fixture against MinGW GCC or Clang for Windows and checks it there.
- To compare a signature with Clang's lowering: `clang --target=x86_64-pc-windows-msvc -S -emit-llvm structs.c` (or `aarch64-pc-windows-msvc`, `arm64-apple-macos`...) against `jaic build ... -target <same triple> --emit-ir out.ll`. Clang emits IR for any target without its libraries, so this works on any host.
- Pieces become separate LLVM parameters (`f32, f32` rather than Clang's `[2 x float]`, `i64, i64` rather than `[2 x i64]`). Register assignment is the same except when an aggregate needs two registers and only one is left: AAPCS64 then puts all of it on the stack, while separate parameters would split it.

## Configuration

None.

## Dependencies

- `crates/jaic/src/abi.rs`, `crates/jaic/src/interp/native.rs`, `crates/jaic-llvm/src/lower.rs`.
- System `cc` for the native test fixture.
