# Procedures and calls

## What it is

Procedure declarations, calls, overload sets, default and named arguments, multiple returns, variadics, and the header directives `#caller_location`, `#deprecated` and `#no_debug`.

## How it works

`parser/procedure.rs` parses headers; the trailing directives it recognises are in a table at the top of the file. `sema/procs.rs` checks declarations; `sema/calls.rs` resolves calls, ranks overloads and expands macros.

```jai
named :: (a: int, b := 7, c := "z") -> int { return a*100 + b; }
pair  :: () -> int, string { return 4, "four"; }
sum   :: (args: ..int) -> int { t := 0; for args t += it; return t; }

named(1, c = "q")        // 107
named(b = 1, a = 2)      // 201
i, s := pair();          // 4 four
sum(1, 2, 3)             // 6
```

The calls give the commented results: defaults and named arguments {#proc.1}, multiple return values {#proc.2} and a variadic `..int` {#proc.3}.

Two non-polymorphic procedures declared in one scope with the same name and the same parameter types (names, results and defaults do not count) are an error, as in Jai, also when nothing calls them: `check_identical_overloads` (`sema/procs.rs`) runs on every overload set of the program's own files when compilation finishes, and on a set when a call first uses it. Polymorphic overloads are told apart by calls and are not compared.

Overloads are procedures sharing a name; the call picks by argument types {#proc.4}. A constant alias to a procedure joins the overload set {#proc.5}, which is how modules re-export one name for several implementations:

```jai
h :: f;   // f :: (x: int) -> int
h :: g;   // g :: (x: float) -> int
h(1), h(1.0)   // 1 2
```

`#caller_location` as a default value is evaluated at the call site and gives a `Source_Code_Location` (`location_operand` in `sema/expr.rs`) {#proc.6}. `#location()`, `#file` and `#line` give the directive's own position {#proc.7}. `#file` and `#filepath` (the directory, with a trailing `/`) spell paths with `/` on Windows too, so a metaprogram can splice them into a string literal or a `#load`.

A `Code` parameter on a plain procedure takes any expression as code, like a macro: `convert(1 + 2 * 3)` receives the code of `1 + 2 * 3` {#proc.8}. An argument that already is a `Code` passes its value (`param_value` in `sema/calls.rs`) {#proc.9}. A baked variadic `$args: ..Code` quotes each argument the same way (`quote_code_arg`), giving a constant `[] Code`; this is how Print_Vars takes its expressions.

Parameters are values: a procedure that changes a struct or string parameter, or passes its address on (`advance(*s, 1)`), changes its own copy, never the caller's variable. Aggregates arrive as a pointer to the caller's value and the procedure copies them on entry (`lower_body_code` in `sema/procs.rs`) {#proc.12}.

`#deprecated "msg"` is parsed and exported to metaprograms {#proc.10}; a call to such a procedure compiles and runs {#proc.11} but produces no warning. `#no_debug` is a header flag read by `sema/procs.rs` and `sema/code_export.rs`.

`#must` and `#discard` are in [must and discard](must-and-discard.md).

## How to change it

- New header directive: add it to the table in `parser/procedure.rs`, add a field to the header flags in `ast.rs` (`no_debug` is the pattern), and consume it in `sema/procs.rs`.
- Overload ranking is `arg_cost` in `sema/calls.rs`. It affects every call; run the full sweep after changing it.

Tests: `tests/stdlib/proc-alias-overloads.jai`, `modern-library-conversions.jai`, `aggregate-parameter-copy.jai`.

## Dependencies

`Source_Code_Location` comes from the preload via `preload_type` in `sema/expr.rs`.
