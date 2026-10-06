# Lambdas and anonymous procedures

## What it is

Procedure literals with a body, `(x) => expr` lambdas, and bare `x => expr` {#lambda.1}. A lambda takes its types from the expected procedure type {#lambda.2}, or becomes polymorphic when bound with `::` {#lambda.3}.

## How it works

`sema/lambda.rs` lowers a lambda to an ordinary procedure literal (`lambda_lit`), checks it against the expected type (`check_lambda`), instantiates it per use (`lambda_instance`) and infers result types, including a callee's `$R` (`lambda_return_type`, `infer_lambda_bindings`) {#lambda.4}.

```jai
apply   :: (a: int, f: (int) -> int) -> int { return f(a); }
map_sum :: (array: [] $T, f: (T) -> $R) -> R { total: R; for array total += f(it); return total; }
add     :: (a, b) => a + b;                  // polymorphic: add(2, 3) == 5, add(1.5, 2.0) == 3.5
add10   :: #bake_arguments add(a = 10);      // add10(5) == 15
fp: (int, int) -> int = (a, b) => a * b;     // fp(6, 7) == 42

apply(5, x => x + 1)               // 6
map_sum(int.[1,2,3,4], x => x*x)   // 30
twice(() { n += 1; }, 3)           // block body; n is a global
```

The commented results are what jaic computes {#lambda.5}; the block-bodied `twice` call runs its body three times {#lambda.6}.

Lambdas do not capture locals. Using one is `error: cannot access local 'n' of an enclosing procedure` {#lambda.7}; pass state through parameters, pointers or globals.

## How to change it

A lambda is checked against whatever procedure type the context supplies, so inference bugs are usually on the callee side (`infer_lambda_bindings`), not in the lambda body. Tests: `tests/stdlib/lang-lambdas.jai`, `proc-type-defaults-ifx.jai`, `type-directive-proc-one-param.jai`.

## Dependencies

`sema/lambda.rs`, `sema/calls.rs`, `sema/bake.rs`.
