# C ABI, callbacks and C++ methods

## What it is

How `jaic` passes C structs by value, variadic arguments and C++ method receivers across `#foreign` calls and `#c_call` callbacks. The same classification drives the interpreter's native foreign calls (`jaic run`) and the LLVM backend (`jaic build`).

## How it works

Classification lives in `crates/jaic/src/abi.rs` (`classify_arg`, `classify_ret`, `Arch::{Aarch64, X86_64, Win64}`). It only looks at an `AggLayout` (size, alignment, flattened scalar fields) from `crates/jaic/src/ir.rs`. Three conventions are implemented: AArch64 (Apple flavour), x86-64 System V and the Microsoft x64 convention (`Win64`, chosen for any `windows`/`mingw` x86-64 triple by `Arch::from_triple`).

Microsoft x64 (`Win64`): an aggregate of exactly 1, 2, 4 or 8 bytes is passed and returned as one integer (`i64` piece) in a general register, even when its members are floats (`struct { float x, y; }` travels in RCX/RAX, not XMM). Every other size is passed by reference to a caller-made copy (`Passing::Indirect`) and returned through a hidden `sret` pointer. Scalars, the 32-byte shadow space, stack slots beyond the fourth argument and variadic doubles duplicated in integer registers are LLVM's job once the function type is right. The interpreter does not implement this convention (it does not load native libraries on Windows), so `call_with`/`callback_addr` in `interp/native` return an error for `Win64`. See [Windows](windows.md).

Each `Conv::C` signature may carry a `CAbi` (`params: Vec<Option<AggLayout>>`, `ret`, `ret_indirect`) which `sema/procs.rs` fills in from the procedure header. The IR itself passes aggregates by pointer; marshalling happens at the boundary:

- Interpreter: `interp/native.rs` loads register pieces from the aggregate and calls the function pointer; large results use a hidden result pointer.
- LLVM: see [LLVM backend](llvm-backend.md) for the call-site and definition-side rules.

Variadic C calls (`args: ..Any` on a `#foreign` procedure) are passed with the platform's variadic convention, which matters on Apple arm64 where variadic arguments go on the stack. Regression program: `tests/stdlib/c-variadic-foreign-calls.jai`; native test `c_variadic_calls` in `crates/jaic-cli/tests/native.rs`.

`#cpp_method` procedure types are treated as `#c_call` (no implicit `context`) with the object pointer as first argument (`sema/procs.rs`, `sema/expr.rs`). The reflection flag `0x1000` is set for them (`sema/code_export.rs`). Verified in `tests/stdlib/cpp-method-and-array-decay.jai`:

```jai
Vtable :: struct { add: #type (this: *Obj, x: s32) -> s32 #cpp_method; }
add_impl :: (this: *Obj, x: s32) -> s32 #c_call { return this.base + x; }
// vt.add(*o, 2) == 42
```

`#cpp_return_type_is_non_pod` on a foreign procedure makes a struct result always go through the hidden result pointer whatever its size (`CAbi::ret_indirect`, set in `sema/procs.rs`, honored in `interp/native.rs`). The LLVM backend does not read `ret_indirect` today, so this marker only affects `jaic run`.

## How to change it

- New architecture or rule: extend `abi.rs` (it has unit tests), then the LLVM `bind_params`/call lowering in `crates/jaic-llvm/src/lower.rs` and the interpreter in `interp/native.rs`; keep both in step.
- Small signed integers are not sign/zero-extended per the C ABI because the IR does not carry signedness.
- Fixture: `tests/native/c-structs-by-value/` (`structs.c` plus Jai files) is exercised by the `c_structs_by_value` test, which compares interpreter and native output and also calls `#c_call` Jai callbacks from C. It needs `cc` (on Windows `clang` and `llvm-ar`, native output only). `tools/windows_cross.py` builds the same fixture against MinGW GCC or Clang for Windows and checks it there.
- To compare a signature with Clang's lowering: `clang --target=x86_64-pc-windows-msvc -S -emit-llvm structs.c` against `jaic build ... --emit-ir out.ll`.

## Configuration

None.

## Dependencies

- `crates/jaic/src/abi.rs`, `crates/jaic/src/interp/native.rs`, `crates/jaic-llvm/src/lower.rs`.
- System `cc` for the native test fixture.
