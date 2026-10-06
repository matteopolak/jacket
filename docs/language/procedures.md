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

Overloads are procedures sharing a name; the call picks by argument types. A constant alias to a procedure joins the overload set, which is how modules re-export one name for several implementations:

```jai
h :: f;   // f :: (x: int) -> int
h :: g;   // g :: (x: float) -> int
h(1), h(1.0)   // 1 2
```

`#caller_location` as a default value is evaluated at the call site and gives a `Source_Code_Location` (`location_operand` in `sema/expr.rs`). `#location()`, `#file` and `#line` give the directive's own position.

A `Code` parameter on a plain procedure takes any expression as code, like a macro: `convert(1 + 2 * 3)` receives the code of `1 + 2 * 3`. An argument that already is a `Code` passes its value (`param_value` in `sema/calls.rs`).

`#deprecated "msg"` is parsed and exported to metaprograms but produces no warning. `#no_debug` is a header flag read by `sema/procs.rs` and `sema/code_export.rs`.

`#must` and `#discard` are in [must and discard](must-and-discard.md).

## How to change it

- New header directive: add it to the table in `parser/procedure.rs`, add a field to the header flags in `ast.rs` (`no_debug` is the pattern), and consume it in `sema/procs.rs`.
- Overload ranking is `arg_cost` in `sema/calls.rs`. It affects every call; run the full sweep after changing it.

Tests: `tests/stdlib/proc-alias-overloads.jai`, `modern-library-conversions.jai`.

## Dependencies

`Source_Code_Location` comes from the preload via `preload_type` in `sema/expr.rs`.
