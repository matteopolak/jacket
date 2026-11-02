# Leniencies: code Jacket accepts and the official compiler rejects

A leniency is code `jaic` accepts that the official Jai compiler rejects. Each is a warning with its own code and a shared note:

```
note: the official Jai compiler rejects this; Jacket accepts it but may stop in a future release
```

A program that compiles without these warnings also compiles with the official compiler (as far as these rules go); each warning names a spelling that works with both. Only code the official compiler rejects is a leniency.

## How it works

- Warnings are made only for the program's own files, never for imported modules, since a module's author can't act on them from the importing program.
- The parser records constructs it accepts by spelling and reports them when the file is declared. The checker warns where it takes a lenient path (member lookup for `loose`, `for_expansion`, identifier lookup, and the sweep for declarations that hide imports).
- Warnings print before errors and never change the exit status. `-quiet` drops every warning; there is no per-code switch.
- Rejected by both compilers, and so errors with their own code rather than leniencies: `implicit-narrowing` ([casts](casts-and-conversions.md)), `ambiguous-procedure-value` ([procedures](procedures.md)), `shadows-builtin-type` ([declarations](declarations-and-constants.md)).

## The list

| Code | Accepted | Portable spelling |
|---|---|---|
| `operator-out-of-scope` | `u + v` on `Math.Vector2` behind `Math :: #import "Math"` | `Math.operator +(u, v)`, or `using Math;` |
| `declaration-hides-import` | a program declaration named like something an import exports (`errno: s32;` beside `#import "Sys"`), or a procedure with the same parameter types as an imported one | rename it, or import the module under a name; overloads with different parameters are fine |
| `namespaced-for-expansion` | `for x` over a type whose `for_expansion` only its module's namespace holds | `using HT;` or a plain `#import` |
| `place-directive` | `#place a; b: T;` | `#overlay(a) b: T;` |
| `loose-enum` | `Enum.loose` as a type | the enum's integer type, and `cast(Enum)` |
| `foreign-library-directive` | `#foreign_library`, `#foreign_system_library` | `#library`, `#system_library` |
| `unnamed-library-directive` | `#library,link_always "x";` without a name | `lib :: #library,link_always "x";` |
| `must-after-return-type` | `-> T #must` | remove it and use the result at every call |
| `ifx-then-semicolon` | `#ifx c then a; else b;` | `#ifx c then a else b;` |
| `address-of-struct-literal` | `*T.{..}` | `*(T.{..})` |
| `float-exponent-without-fraction` | `1e20` | `1.0e20` |
| `struct-align-directive` | `S :: struct #align 64 { .. }` | `member: T #align 64;` on the first member |
| `for-over-array-literal` | `for x: .[1, 2, 3]` | `for x: s64.[1, 2, 3]` |
| `struct-literal-as-dynamic-array` | `a = .{}` where `a` is a `[..]T` | `a = type_of(a).{..}` |
| `target-features-directive` | `#target_features "avx2"` on a procedure | remove it |
| `push-context-without-block` | `push_context ctx run();` | `push_context ctx { run(); }` |
| `main-returns-value` | `main :: () -> s64 { return 1; }` | `main :: () { exit(1); }` |
| `caller-location-outside-default` | `loc := #caller_location;` in a body | `#location()`, or take it as a parameter default |
| `multi-value-in-code-argument` | `f(#code { a, b := 1, 2; })` | separate declarations, or bind the code to a name first |
| `spread-with-other-varargs` | `f(first, ..rest)` | collect into one array: `f(..all)` |
| `exists-on-expression` | `#exists(S.Soa(Item, 3).member)` | name the type first, then `#exists(Packed.member)` |
| `procedure-cast-to-integer` | `cast(u64) some_procedure` | `cast(u64) cast(*void) some_procedure` |
| `address-of-macro-constant` | `*x` in a macro whose parameter the call binds to a constant | `copy := x;` and take `*copy` |
| `positional-literal-defaults` | `.{1, 2}` for a struct whose trailing members have defaults | `.{1, 2, 7}` or `.{a = 1, b = 2}` |
| `note-with-spaces` | `@skip_on(.MACOS, reason = "x y")` | a note without whitespace: `@range(1,64)` |

`declaration-hides-import` is rejected in the program whether or not it is used; in an imported module the same code compiles, and jaic prefers the module's own declaration. A namespaced import never clashes. See [procedures](procedures.md) and [scoping](scoping.md).

## How to change it

- A new leniency: add a variant to `Leniency` (code, `ALL`), warn through `Compiler::leniency_warning` (`sema/calls/lenient.rs`), and add a row above. Constructs recognised by spelling go in the parser (`note_lenient`) and `report_parsed_leniencies`.
- When the compiler stops accepting one, the warning becomes the error with the same help.
- Tests: `crates/jaic/tests/leniency_warnings.rs` has one test per code and checks that the same text in an imported module is silent. `tools/ci/check_diagnostics_covered.py` needs each code named by a test.
