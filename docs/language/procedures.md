# Procedures, calls and lambdas

## What it is

Procedure declarations, calls, overloads, default and named arguments, multiple returns, variadics, lambdas and header directives (`#caller_location`, `#deprecated`, `#no_debug`).

## How it works

`parser/procedure.rs` parses headers (trailing directives are in a table at the top). `sema/procs.rs` checks declarations; `sema/calls/` resolves calls, ranks overloads (`arg_cost` in `mod.rs`) and expands macros.

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

- Two non-polymorphic procedures in one scope with identical parameter types are an error even if never called (`check_identical_overloads`). Aliases of the same procedure, even in different scopes of one file (`read_it :: Impl.read_it;` at file scope and again inside `#if`), count as one overload.
- A procedure type returned without parentheses cannot be followed by a body: `f :: () -> (A, B) -> C { ... }` is a parse error; write `-> ((A, B) -> C) { ... }`.
- `#caller_location` as a default is evaluated at the call site (`location_operand` in `sema/expr.rs`). `#file` and `#filepath` use `/` on Windows too.
- A `Code` parameter on a plain procedure takes any expression as code; a baked `$args: ..Code` quotes each argument (`quote_code_arg`).
- Parameters are values: aggregates arrive as a pointer and the callee copies them on entry (`lower_body_code`), so it never changes the caller's variable.
- Inside an argument or field list a comma normally ends a return type, so `f(a, (x: int) -> bool, T { ... })` would read `T` as the next argument. When the tokens after the comma run up to a `{` before any `,`, `;` or closing bracket, they are another return type of a procedure literal and the `{` starts its body, so a struct-literal field such as `proc = (data: *u8) -> bool, Pad { ... }` parses (`return_list_precedes_body` in `parser/procedure.rs`) {#proc.13}.
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

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

- Procedure literals with a body, `(x) => expr` lambdas, and bare `x => expr` {#lambda.1}. A lambda takes its types from the expected procedure type {#lambda.2}, or becomes polymorphic when bound with `::` {#lambda.3}.
- `sema/lambda.rs` lowers a lambda to an ordinary procedure literal (`lambda_lit`), checks it against the expected type (`check_lambda`), instantiates it per use (`lambda_instance`) and infers result types, including a callee's `$R` (`lambda_return_type`, `infer_lambda_bindings`) {#lambda.4}.
- The commented results are what jaic computes {#lambda.5}; the block-bodied `twice` call runs its body three times {#lambda.6}.
- Lambdas do not capture locals. Using one is ``error: cannot access local `n` of an enclosing procedure`` {#lambda.7}; pass state through parameters, pointers or globals.
- The calls give the commented results: defaults and named arguments {#proc.1}, multiple return values {#proc.2} and a variadic `..int` {#proc.3}.
- Overloads are procedures sharing a name; the call picks by argument types {#proc.4}. A constant alias to a procedure joins the overload set {#proc.5}, which is how modules re-export one name for several implementations:
- A candidate is ranked by the conversion cost of its arguments, then by how well its parameters fit the untyped literals passed (see [types from context](casts-and-conversions.md#types-from-context)). A named member the type lacks, more positional values than it has members, or too few for its members without initializers rule a struct out; of the rest, positional values prefer the type with the fewest members left over, and named values leave members with initializers to them. A tie is `Procedure call matches N possible overloads.` (`ambiguous_overloads`): `.{}`, `null`, `.[..]` and two equally polymorphic headers (`$T` and `$U`) have no better reading, while an untyped number fits every numeric parameter alike and the first declared wins. Among otherwise equally matching candidates, a fixed-arity header beats a variadic header, including when the variadic pack is empty; and an overload in an inner scope is nearer than one in an outer scope. A procedure the calling module declares itself beats an imported one that fits equally well when the import comes from the module's files or a body's `#import` (`choose_candidate`), but not when a `using` of the imported module in a body at or around the call brings it in: that `using` is nearer than the module's declarations, so the imported procedure is called (`brought_by_nearer_using`). The same-scope tie rule above applies when both belong to that module or neither does. In the program's own files the official compiler rejects this (the two have identical parameter types), so the declaration warns (`declaration-hides-import`, see [leniencies](leniencies.md)); inside an imported module it compiles there too.
- `#caller_location` as a default value is evaluated at the call site and gives a `Source_Code_Location` (`location_operand` in `sema/expr.rs`) {#proc.6}. `#location()`, `#file` and `#line` give the directive's own position {#proc.7}. `#file` and `#filepath` (the directory, with a trailing `/`) spell paths with `/` on Windows too, so a metaprogram can splice them into a string literal or a `#load`.
- A `Code` parameter on a plain procedure takes any expression as code, like a macro: `convert(1 + 2 * 3)` receives the code of `1 + 2 * 3` {#proc.8}. An argument that already is a `Code` passes its value (`param_value` in `sema/calls/emit.rs`) {#proc.9}. A baked variadic `$args: ..Code` quotes each argument the same way (`quote_code_arg`), giving a constant `[] Code`; this is how Print_Vars takes its expressions.
- Parameters are values. A scalar parameter is a mutable local. An aggregate parameter (struct, string, fixed array, view) is read-only: `p.x = 1`, `p.q.y = 1`, `s = "zz"`, `s.count = 1` and `a[0] = 5` on a fixed array are errors, `Can't assign to an immutable argument.` (`reject_constant_target` in `sema/stmt/assign.rs`, `FnCtx::immutable`). Writing through a pointer inside the aggregate, and taking its address (`advance(*s, 1)`, which changes the procedure's own copy, never the caller's variable), stay legal; so does writing an element of a view or string parameter, whose data lives behind a pointer. To change an aggregate argument, copy it first: `s := s;`. A member brought in by `using o: Outer` is part of `o`, so `x = 9;` and `x += 9;` are errors too. A `#compile_time` procedure is exempt and may assign to its arguments. The check compares the local's own storage, found by name, so a global or a `using` member that shares a name with a parameter of some other signature is never mistaken for an argument {#proc.12}.
- `#deprecated "msg"` is parsed and exported to metaprograms {#proc.10}; a call to such a procedure compiles and runs {#proc.11} but produces no warning. `#no_debug` is a header flag read by `sema/procs.rs` and `sema/code_export.rs`.
