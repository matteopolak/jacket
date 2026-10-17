# Procedures, calls and lambdas

## What it is

Procedure declarations, calls, overloads, default and named arguments, multiple returns, variadics, lambdas and header directives (`#caller_location`, `#deprecated`, `#no_debug`).

## How it works

`parser/procedure.rs` parses headers (trailing directives are in a table at the top). `sema/procs.rs` checks declarations; `sema/calls.rs` resolves calls, ranks overloads (`arg_cost`) and expands macros.

```jai
named :: (a: int, b := 7, c := "z") -> int { return a*100 + b; }
pair  :: () -> int, string { return 4, "four"; }
sum   :: (args: ..int) -> int { t := 0; for args t += it; return t; }

named(1, c = "q")        // 107
named(b = 1, a = 2)      // 201
i, s := pair();          // 4 four
sum(1, 2, 3)             // 6

h :: f;  h :: g;         // aliases join one overload set
```

- Two non-polymorphic procedures in one scope with identical parameter types are an error even if never called (`check_identical_overloads`).
- `#caller_location` as a default is evaluated at the call site (`location_operand` in `sema/expr.rs`). `#file` and `#filepath` use `/` on Windows too.
- A `Code` parameter on a plain procedure takes any expression as code; a baked `$args: ..Code` quotes each argument (`quote_code_arg`).
- Parameters are values: aggregates arrive as a pointer and the callee copies them on entry (`lower_body_code`), so it never changes the caller's variable.
- `#deprecated "msg"` is exported to metaprograms but produces no warning.

### Lambdas

`(x) => expr` and `x => expr` take their types from the expected procedure type, or are polymorphic when bound with `::`. `sema/lambda.rs` lowers one to an ordinary literal (`lambda_lit`), checks it (`check_lambda`) and instantiates it per use.

```jai
add   :: (a, b) => a + b;                // add(1.5, 2.0) == 3.5
add10 :: #bake_arguments add(a = 10);
apply(5, x => x + 1)                     // 6
```

Lambdas do not capture locals (``cannot access local `n` of an enclosing procedure``); pass state through parameters, pointers or globals. Inference bugs are usually on the callee side (`infer_lambda_bindings`).

## How to change it

New header directive: add it to the table in `parser/procedure.rs` and the flags in `ast.rs`, then read it in `sema/procs.rs`. Changing `arg_cost` affects every call; run the full sweep. Related: `sema/bake.rs`; `#must` and `#discard` are in [directives and notes](directives-and-notes.md).

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Procedure literals with a body, `(x) => expr` lambdas, and bare `x => expr` {#lambda.1}. A lambda takes its types from the expected procedure type {#lambda.2}, or becomes polymorphic when bound with `::` {#lambda.3}.
- `sema/lambda.rs` lowers a lambda to an ordinary procedure literal (`lambda_lit`), checks it against the expected type (`check_lambda`), instantiates it per use (`lambda_instance`) and infers result types, including a callee's `$R` (`lambda_return_type`, `infer_lambda_bindings`) {#lambda.4}.
- The commented results are what jaic computes {#lambda.5}; the block-bodied `twice` call runs its body three times {#lambda.6}.
- Lambdas do not capture locals. Using one is ``error: cannot access local `n` of an enclosing procedure`` {#lambda.7}; pass state through parameters, pointers or globals.
- The calls give the commented results: defaults and named arguments {#proc.1}, multiple return values {#proc.2} and a variadic `..int` {#proc.3}.
- Overloads are procedures sharing a name; the call picks by argument types {#proc.4}. A constant alias to a procedure joins the overload set {#proc.5}, which is how modules re-export one name for several implementations:
- `#caller_location` as a default value is evaluated at the call site and gives a `Source_Code_Location` (`location_operand` in `sema/expr.rs`) {#proc.6}. `#location()`, `#file` and `#line` give the directive's own position {#proc.7}. `#file` and `#filepath` (the directory, with a trailing `/`) spell paths with `/` on Windows too, so a metaprogram can splice them into a string literal or a `#load`.
- A `Code` parameter on a plain procedure takes any expression as code, like a macro: `convert(1 + 2 * 3)` receives the code of `1 + 2 * 3` {#proc.8}. An argument that already is a `Code` passes its value (`param_value` in `sema/calls.rs`) {#proc.9}. A baked variadic `$args: ..Code` quotes each argument the same way (`quote_code_arg`), giving a constant `[] Code`; this is how Print_Vars takes its expressions.
- Parameters are values: a procedure that changes a struct or string parameter, or passes its address on (`advance(*s, 1)`), changes its own copy, never the caller's variable. Aggregates arrive as a pointer to the caller's value and the procedure copies them on entry (`lower_body_code` in `sema/procs.rs`) {#proc.12}.
- `#deprecated "msg"` is parsed and exported to metaprograms {#proc.10}; a call to such a procedure compiles and runs {#proc.11} but produces no warning. `#no_debug` is a header flag read by `sema/procs.rs` and `sema/code_export.rs`.
