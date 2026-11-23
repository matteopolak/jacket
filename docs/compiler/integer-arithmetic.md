# Integer arithmetic

One definition of integer `+ - * / % & | ^ << >>` and rotates for a given width and signedness lives in `crates/jaic/src/int_arith.rs` (`int_binop`, `wrap`, `float_to_int`). Constant folding, the interpreter and the editor's interval analysis all compute through it, so a folded constant, a `#run` and the compiled program agree.

## Semantics

- `+ - *` wrap to the type's width.
- Division or remainder by zero is an error for the caller: a compile error in a constant, a trap at run time. `MIN / -1` is `MIN`; `MIN % -1` is `0`.
- A shift by the width or more gives `0`; a signed `>>` fills with the sign bit (`-128 >> 200` is `-1`).
- A rotate takes its amount modulo the width.
- Float to integer truncates toward zero and saturates; `NaN` is `0`. Integer narrowing keeps the low bits.

## How it works

`int_binop(op, lhs, rhs, bits, signed)` takes values already in the type and returns the result or `DivideByZero`. Untyped constants use 128 bits, signed. Callers: `sema/binary/fold.rs` (folding and its diagnostics), `interp/arith.rs`, `ide/interval.rs` (endpoints only), and `sema/convert/casts.rs`.

## How to change it

Change the rule in `int_arith.rs`, then update `crates/jaic-llvm/src/lower/inst.rs`, the one place that lowers independently. `tests/compiler/semantics/integer-constant-arithmetic.jai` compares folded, `#run` and compiled results; unit tests in `int_arith/tests.rs` check every width against Rust.

## Interpreter division

`Interp::divide` uses native 64-bit division (`divide_64` in `interp/arith.rs`); every interpreted type is at most 64 bits wide. It must agree with `int_binop`, the shared definition: `arith/tests.rs` compares them over the edge words, and `tests/interp_division.rs` runs `MIN / -1`, truncation and division by zero through programs.
