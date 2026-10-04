# Procedures and calls

## What it is

How `jaic` handles procedure declarations, calls, overload sets, default and named arguments, multiple returns, variadics, and the header directives `#caller_location`, `#deprecated`, `#no_debug` and `#discard`.

## How it works

Procedure headers are parsed in `crates/jaic/src/parser/procedure.rs` (the recognised trailing directives are listed in a table at its top: `expand`, `no_debug`, `deprecated`, ...). Semantic checking of declarations is in `crates/jaic/src/sema/procs.rs`; call resolution, overload ranking and macro expansion are in `crates/jaic/src/sema/calls.rs`.

```jai
named :: (a: int, b := 7, c := "z") -> int { return a*100 + b; }
pair  :: () -> int, string { return 4, "four"; }
sum   :: (args: ..int) -> int { t := 0; for args t += it; return t; }

named(1, c = "q")        // 107
named(b = 1, a = 2)      // 201
i, s := pair();          // 4 four
sum(1, 2, 3)             // 6
```

Overloads are ordinary procedures sharing one name; the call picks by argument types (`show(1)`, `show(2.5)`, `show("s")` each choose their own overload). A constant alias to a procedure joins an overload set, which is how modules re-export one name for several implementations:

```jai
h :: f;   // f :: (x: int) -> int
h :: g;   // g :: (x: float) -> int
h(1), h(1.0)   // 1 2
```

`tests/stdlib/proc-alias-overloads.jai` covers the module form (`starts_with :: begins_with;` joining a local overload).

`#caller_location` as a default value is evaluated at the call site and yields a `Source_Code_Location` (`location_operand` in `sema/expr.rs`). `#location()`, `#file` and `#line` give the directive's own position:

```jai
where :: (l := #caller_location) { print("%,% in %\n", l.line_number, l.character_number, l.fully_pathed_filename); }
```

`#deprecated "msg"` and `#no_debug` are parsed as header flags (`flags.no_debug` is read in `sema/procs.rs` and `sema/code_export.rs`). A call to a `#deprecated` procedure compiles and runs; no warning was observed in `jaic run` output.

`#discard` on a parameter: the argument is typechecked but never evaluated, and the procedure cannot name the parameter. `#must` on a result: discarding it is an error. Both are described in [must-and-discard.md](must-and-discard.md).

## How to change it

- New header directive: add it to the table in `parser/procedure.rs`, set a field on the header flags in `ast.rs` (`no_debug` is the pattern), then consume it in `sema/procs.rs`.
- Overload ranking lives in `sema/calls.rs` (`arg_cost`); changes there affect every call, so re-run the `tests/stdlib/*.jai` programs afterward.
- `#discard` and `#must` are enforced in `sema/calls.rs` and `sema/stmt.rs`; see [must-and-discard.md](must-and-discard.md).

## Configuration

None.

## Dependencies

`Basic` (`print`) for the examples; `Source_Code_Location` comes from the preload module via `preload_type` in `sema/expr.rs`.
