# Lambdas and anonymous procedures

## What it is

Procedure literals with a body, `(x) => expr` lambdas, and bare `x => expr` forms. Lambdas take their types from the expected procedure type, or become polymorphic when bound with `::`.

## How it works

`crates/jaic/src/sema/lambda.rs` lowers a lambda into an ordinary procedure literal (`lambda_lit`), checks it against the expected type (`check_lambda`), instantiates it per use (`lambda_instance`) and infers result types, including `$R` in the callee (`lambda_return_type`, `infer_lambda_bindings`).

```jai
apply   :: (a: int, f: (int) -> int) -> int { return f(a); }
map_sum :: (array: [] $T, f: (T) -> $R) -> R { total: R; for array total += f(it); return total; }
add     :: (a, b) => a + b;                  // polymorphic: add(2, 3) == 5, add(1.5, 2.0) == 3.5
add10   :: #bake_arguments add(a = 10);      // add10(5) == 15
fp: (int, int) -> int = (a, b) => a * b;     // fp(6, 7) == 42

apply(5, x => x + 1)               // 6
map_sum(int.[1,2,3,4], x => x*x)   // 30
```

Block-bodied anonymous procedures work too: `twice(() { n += 1; }, 3)` with a global `n` runs the body three times.

Lambdas do not capture locals. Referencing one fails at check time:

```
error: cannot access local 'n' of an enclosing procedure
```

Use globals or pass state through parameters or pointers.

## How to change it

- Full examples: `tests/stdlib/lang-lambdas.jai`, `proc-type-defaults-ifx.jai`, `type-directive-proc-one-param.jai`.
- Lambda checking compares against whatever procedure type the context supplies, so inference bugs are usually in the callee side (`infer_lambda_bindings`) rather than the lambda body.

## Configuration

None.

## Dependencies

`sema/lambda.rs`, `sema/calls.rs` and `sema/bake.rs` inside `crates/jaic`.
