# Integer arithmetic

One definition of integer `+ - * / % & | ^ << >>` and rotates for a type of a given width and signedness, in `crates/jaic/src/int_arith.rs` (`int_binop`, `wrap`, `float_to_int`). Constant folding, the interpreter and the editor's interval analysis all compute through it, so a folded constant, a `#run` and the compiled program give the same value.

## Semantics

The compiled program decides them (`jaic-llvm/src/lower/inst.rs` lowers the same rules):

- `+ - *` wrap to the width of the type.
- Division or remainder by zero is an error for the caller: a compile error in a constant, a trap in the interpreter and compiled code. `MIN / -1` wraps to `MIN` and `MIN % -1` is `0`.
- A shift by the width or more gives `0`; a signed `>>` fills with the sign bit instead (`-128 >> 200` is `-1`). A negative amount is out of range too, but a constant shift by a negative amount is reported as an error before it gets here.
- A rotate takes its amount modulo the width.
- Float to integer truncates toward zero and saturates at the type's range; `NaN` is `0`. Narrowing between integers keeps the low bits (`wrap`).

## How it works

`int_binop(op, lhs, rhs, bits, signed)` takes values already in the type (`rhs` is a plain count for shifts and rotates) and returns the value in the type, or `DivideByZero`. Untyped constants use `bits = 128, signed = true`.

- `sema/binary/fold.rs` maps the operator, keeps the diagnostics (division by zero, negative shift) and calls it with the operand type's width.
- `interp/arith.rs` maps the IR operator to `IntOp` (`int_op`), widens the register words (sign-extending for signed operations) and masks the result. `Interp::divide` and the const propagation in `interp/code/build.rs` (`fold_bin`) use the same `int_bin`.
- `ide/interval.rs` keeps its interval logic (mathematical results, no wrapping) but computes endpoints with the shared operation (`scalar`), and its tests compare against it.
- Casts of constants use `wrap` and `float_to_int` (`sema/convert/casts.rs`); the interpreter's float conversions use `float_to_int`.

## How to change it

Change the rule in `int_arith.rs`, then update `crates/jaic-llvm/src/lower/inst.rs` to match (it is the one place that lowers independently). `tests/compiler/semantics/integer-constant-arithmetic.jai` checks folded, `#run` and compiled results against each other (`crates/jaic/tests/integer_constant_arithmetic.rs`, `crates/jaic-llvm/tests/integer_constant_arithmetic.rs`); the unit tests in `int_arith/tests.rs` check every width against Rust's native operations.

## Dependencies

`sema`, `interp` and `ide` depend on it; it depends on nothing.
