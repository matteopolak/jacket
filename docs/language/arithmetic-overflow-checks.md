# Arithmetic overflow checks and `#no_aoc`

## What it is

`Build_Options.arithmetic_overflow_check` (`.OFF` by default, `.NONFATAL`, `.FATAL`) makes integer `+`, `-` and `*` (and `+=`, `-=`, `*=`) report results that do not fit the operand type {#overflow.1}. `#no_aoc` turns the check off for code that overflows on purpose, like hashes and random number generators {#overflow.2}.

## How it works

`check_binary` in `sema/expr.rs` emits the operation, then calls `emit_overflow_check` when the option is on, the frame is not `no_aoc`, and the type is a non-enum integer. That emits `Intrinsic::{S,U}{Add,Sub,Mul}Overflow` (`(a, b, width_bytes) -> bool`) and, if it is set, a call to `__arithmetic_overflow(left, right, type_code, line, filename)` in `stdlib/Runtime_Support.jai`.

The interpreter computes the intrinsic in 128-bit arithmetic; LLVM uses `llvm.{s,u}{add,sub,mul}.with.overflow`.

`__arithmetic_overflow` prints to stderr {#overflow.3}:

```
file.jai:12: arithmetic overflow computing 250 + 10 as u8: the result does not fit
```

Bit 15 of `type_code` marks fatal mode, which then panics {#overflow.4}; `.NONFATAL` returns and the program continues with the wrapped value {#overflow.5}.

`#no_aoc` goes where `#no_abc` goes: on a procedure (`ProcFlags::no_aoc`, setting `FnCtx::no_aoc` in `sema/procs.rs`), on a block `#no_aoc { }`, and on `while`, `if` and `for` (`Block::no_aoc`, `sema/stmt.rs`) {#overflow.6}.

The option reaches sema as `Options::arithmetic_overflow_check` (0, 1, 2) via the workspace option in `stdlib/Compiler/options.jai` and `build.rs`. It applies to workspaces a metaprogram creates; the program's own workspace keeps the default {#overflow.7}.

Not checked: constant folding (wraps), unary minus, shifts, pointer arithmetic, floats, and arithmetic the compiler emits for indexing and `#asm` {#overflow.8}.

## How to change it

- A new path that lowers `+ - *` itself must call `emit_overflow_check` after the operation and honour `no_aoc`.
- Stdlib code that overflows on purpose needs `#no_aoc`; `Hash`, `PCG`, `Crc`, `xxHash`, `Random` and `Runtime_Support` have it. `Basic/Int128.jai` avoids overflow instead.
- Tests: `tests/stdlib/arithmetic-overflow-check.jai` (each mode, interpreter), `tests/corpus/negative/aoc-fatal-*.jai`, and `arithmetic_overflow_checks` in `crates/jaic-cli/tests/native.rs` (LLVM).

## Dependencies

`sema/expr.rs`, `sema/stmt.rs`, `sema/procs.rs`, `ir.rs`, `interp/mod.rs`, `jaic-llvm/src/lower.rs`, `stdlib/Runtime_Support.jai`, `build.rs`.
