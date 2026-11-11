# Procedures, calls and lambdas

Procedure declarations, calls, overloads, default and named arguments, multiple returns, variadics, lambdas and header directives (`#caller_location`, `#deprecated`, `#no_debug`).

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

The calls give the commented results: defaults and named arguments {#proc.1}, multiple returns {#proc.2} and a variadic `..int` {#proc.3}.

## Overloads

- Overloads are procedures sharing a name; the call picks by argument types {#proc.4}. A constant alias to a procedure joins the overload set {#proc.5}, which is how modules re-export one name for several implementations.
- Two non-polymorphic procedures in a scope with identical parameter types are an error even if never called. Aliases of the same procedure count as one.
- A candidate is ranked by the conversion cost of its arguments, then by how well its parameters fit untyped literals (see [casts](casts-and-conversions.md)). For struct parameters, a named member the type lacks, too many positional values, or too few for members without initializers rule it out.
- A tie is `Procedure call matches N possible overloads.`. `.{}`, `null`, `.[..]` and two equally polymorphic headers have no better reading. An untyped number fits every numeric parameter alike, so the first declared wins.
- Among equal candidates, fixed arity beats variadic, and an inner scope beats an outer one.
- A procedure the calling module declares itself beats an imported one that fits equally well, unless a `using` at or around the call brings the import in (that `using` is nearer). In the program's own files the official compiler rejects the pair, so the declaration warns (`declaration-hides-import`, see [leniencies](leniencies.md)).
- An overloaded name used as a value (`v := over;`) is ``error: ambiguous overloaded procedure used as a value``, or ``N overloads fit procedure type ..`` when a type is wanted (`ambiguous-procedure-value`, in modules too) {#proc.14}. A target type only one overload fits picks it.

## Parameters

- `#caller_location` as a default is evaluated at the call site and gives a `Source_Code_Location` {#proc.6}. `#location()`, `#file` and `#line` give the directive's own position {#proc.7}. `#file` and `#filepath` use `/` on Windows too.
- A `Code` parameter on a plain procedure takes any expression as code, like a macro: `convert(1 + 2 * 3)` receives the code of `1 + 2 * 3` {#proc.8}. An argument that already is `Code` passes its value {#proc.9}. A baked `$args: ..Code` quotes each argument.
- A scalar parameter is a mutable local. An aggregate parameter (struct, string, fixed array, view) is read-only: `p.x = 1`, `s = "zz"`, `a[0] = 5` on a fixed array are errors, `Can't assign to an immutable argument.` Writing through a pointer inside it, taking its address, and writing an element of a view or string stay legal. Copy first to change it: `s := s;`. A member brought in by `using o: Outer` counts as part of `o`. A `#compile_time` procedure is exempt {#proc.12}.
- An aggregate parameter is the caller's value, read in place: `cast(*T) p.bytes.data`, `*p.field` and `for *p.items` point into the caller's variable, and a write to it through another pointer shows in `p`. Taking `*p` of the whole parameter gives the procedure its own copy, made on entry {#proc.19}.
- A bare builtin type name followed by a comma in a parameter list is a nameless parameter: in `(sep: string, string, reversed := false)` the second is an unnamed `string` {#proc.15}. Followed by `:` it is a parameter named like a builtin type, which is rejected (decl.25).
- A procedure value keeps the names, defaults and variadic `..` of the procedure or procedure type it comes from, so `g := f; g(1, 2, 3)` packs `2, 3` as a direct call would. A variable declared `(a: int, rest: ..int)` makes a procedure taking `[] int` callable that way {#proc.17}.
- A block nested in a procedure (such as `#insert -> string { ... }`) may take the `.count` of an enclosing procedure's fixed array via `result.words.count`: the count is part of the type. Any other use of an enclosing local is an error {#proc.16}. Constants of an enclosing local's struct type, its struct parameters among them, are readable as `b.N` or, through `using b`, as `N` {#proc.18}.

## Headers

- `#deprecated "msg"` is parsed and exported to metaprograms {#proc.10}; calls compile and run {#proc.11} with no warning. `#no_debug` is a header flag.
- A returned procedure type without parentheses can't be followed by a body: `f :: () -> (A, B) -> C { ... }` is a parse error; write `-> ((A, B) -> C) { ... }`.
- In an argument or field list a comma normally ends a return type. When the tokens after the comma run up to a `{` before any `,`, `;` or closing bracket, they are another return type and the `{` starts a body, so `proc = (data: *u8) -> bool, Pad { ... }` parses {#proc.13}.

## Lambdas

`(x) => expr`, `(x) => { ... }` and `x => expr` are procedure literals {#lambda.1}. A lambda takes its types from the expected procedure type {#lambda.2}, or is polymorphic when bound with `::` {#lambda.3}.

```jai
add   :: (a, b) => a + b;                // add(1.5, 2.0) == 3.5
add10 :: #bake_arguments add(a = 10);
apply(5, x => x + 1)                     // 6
```

- A lambda lowers to an ordinary literal, is checked against the expected type, instantiated per use, and its result types are inferred, including a callee's `$R` {#lambda.4}.
- The commented results are what jaic computes {#lambda.5}; the block-bodied `twice` call runs its body three times {#lambda.6}.
- Lambdas don't capture locals: ``error: cannot access local `n` of an enclosing procedure`` {#lambda.7}. Pass state through parameters, pointers or globals.

## How to change it

New header directive: add it to the table in `parser/procedure.rs` and the flags in `ast.rs`, then read it in `sema/procs.rs`. Overload ranking is `arg_cost` in `sema/calls/mod.rs`; changing it affects every call, so run the full sweep. Lambdas are in `sema/lambda.rs` (inference bugs are usually on the callee side, `infer_lambda_bindings`). `#must` and `#discard` are in [directives and notes](directives-and-notes.md).
