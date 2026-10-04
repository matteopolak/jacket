# Arithmetic overflow checks and `#no_aoc`

## What it is

`Build_Options.arithmetic_overflow_check` (`.OFF` default, `.NONFATAL`, `.FATAL`) makes integer `+`, `-` and `*` (and `+=`, `-=`,
`*=`) report a result that does not fit the operand type. `#no_aoc` turns the check off for code that overflows on purpose
(hashes, random number generators).

## How it works

- Sema (`check_binary`, `sema/expr.rs`) emits the plain operation and then, when `Options::arithmetic_overflow_check != 0`,
  the frame is not `f.no_aoc` and the type is an integer (not an enum), `emit_overflow_check`. That calls one of
  `Intrinsic::{S,U}{Add,Sub,Mul}Overflow` (`ir.rs`), `(a, b, width_bytes) -> bool`, and branches to a call of
  `__arithmetic_overflow(left, right, type_code, line, filename)` in `stdlib/Runtime_Support.jai` when it is set.
- The interpreter computes the intrinsic in 128-bit arithmetic (`interp/mod.rs`); the LLVM backend uses
  `llvm.{s,u}{add,sub,mul}.with.overflow` (`jaic-llvm/src/lower.rs`).
- `__arithmetic_overflow` writes `file:line: arithmetic overflow computing 250 + 10 as u8: the result does not fit` to stderr.
  `type_code` bit 15 means fatal, which then panics (a trap); `.NONFATAL` returns and the program continues with the wrapped
  value.
- `#no_aoc` is `ProcFlags::no_aoc` on a procedure (`FnCtx::no_aoc` is set in `sema/procs.rs`), `Block::no_aoc` for `#no_aoc { }` and for
  the bodies of `while` / `if` flagged `#no_aoc`, and a flag on `for` (`sema/stmt.rs`): the same placements as `#no_abc`.
- The option comes from `Build_Options.arithmetic_overflow_check`, forwarded as the workspace option
  `arithmetic_overflow_check` (`stdlib/Compiler/options.jai`, `build.rs`). It applies to workspaces a metaprogram creates;
  the program's own workspace keeps the default.
- Not checked: constant folding (still wraps), unary minus, shifts, pointer arithmetic, floats, and the arithmetic the compiler
  emits for indexing and `#asm`.

## How to change it

- A new arithmetic path that lowers `+ - *` itself should call `emit_overflow_check` after the operation and honour `f.no_aoc`.
- Stdlib code that overflows deliberately needs `#no_aoc`; `Hash`, `PCG`, `Crc`, `xxHash`, `Random` and `Runtime_Support` already have it.
  `Basic/Int128.jai` avoids overflow instead.
- Tests: `tests/stdlib/arithmetic-overflow-check.jai` (workspaces with each mode, interpreter), negative programs
  `tests/corpus/negative/aoc-fatal-*.jai`, and `arithmetic_overflow_checks` in `crates/jaic-cli/tests/native.rs` (LLVM).

## Configuration

`Build_Options.arithmetic_overflow_check`: `OFF`, `NONFATAL`, `FATAL`. `Options::arithmetic_overflow_check` is the sema-side
value (0, 1, 2). `#no_aoc` per procedure, block, loop or `if`.

## Dependencies

`sema/expr.rs`, `sema/stmt.rs`, `sema/procs.rs`, `ir.rs`, `interp/mod.rs`, `jaic-llvm`, `stdlib/Runtime_Support.jai`,
`build.rs`.
