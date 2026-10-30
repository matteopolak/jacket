# Leniencies: code Jacket accepts and the official compiler rejects

## What it is

A leniency is a place where `jaic` accepts code that the official Jai compiler rejects. Each one is a warning with a code of its own and a shared note:

```
note: the official Jai compiler rejects this; Jacket accepts it but may stop in a future release
```

A program that compiles here without these warnings also compiles with the official compiler (as far as these rules go); a warning names the spelling that works with both. The list is `jaic::diagnostics::Code::LENIENCIES`; each entry's name is the code that tools see (the playground's JSON and the language server's diagnostic code).

## How it works

`Compiler::leniency_warning` (`sema/calls/lenient.rs`) builds every one of these warnings: the `DiagnosticKind::Lenient(..)` kind, the note, and silence outside the program's own files. A warning is made only when the span is in the main module (the program file and what it `#load`s, wherever the file lives, `stdlib/**/tests/` included); an imported module, bundled or not, is never reported, since its author cannot act on it from the program that imports it. Callers add the label, the help and a fix.

Two mechanisms find them:

- **The parser** notes the constructs it accepts by spelling (`Parser::note_lenient`, `ast::File::lenient`) and `report_parsed_leniencies` turns them into warnings when the file is declared (`declare_loaded` in `sema/modules/loading.rs`). A file parsed ahead on a worker thread keeps them (`ast/relocate.rs` rewrites their spans like any other).
- **The checker** warns where it takes the lenient path: the member lookup for `loose`, `for_expansion_procs`, `check_ident`, and `warn_declarations_hiding_imports` (`sema/hidden_imports.rs`), which sweeps the program's declarations once it has settled.

Warnings print before errors and never change the exit status, so a leniency does not stop a build. The only switch is `-quiet`, which drops every warning; the compiler has no per-code allow or deny (jailint's `-A`/`-W`/`-D` levels name lint rules, not compiler warnings).

## The list

| Code | Accepted | Portable spelling | Warns when |
|---|---|---|---|
| `operator-out-of-scope` | `u + v` on `Math.Vector2` behind `Math :: #import "Math"`: the overload is found through the module of the operand's struct | `Math.operator +(u, v)`, or `using Math;` | the overload is not in scope and the module of the operand type supplies it |
| `declaration-hides-import` | a declaration of the program with the name of something one of its imports exports: `errno: s32;` beside `#import "Sys"`, or a procedure with the same parameter types as an imported one (`get_plugin` beside `#import "Program_Print"`) | rename it, or import the module under a name and qualify the imported one; overloads whose parameters differ are fine | a declaration in the program's own files, unused or not; a named import (`S :: #import "M"`) never clashes |
| `namespaced-for-expansion` | `for x` over a type whose `for_expansion` only its module's namespace holds (`HT :: #import "Hash_Table"`, then `for table`) | `using HT;` or a plain `#import "Hash_Table";` | the `for_expansion` is found through the module of the type |
| `place-directive` | `#place a; b: T;` in a struct body | `#overlay(a) b: T;` | each `#place` |
| `loose-enum` | `Enum.loose` as a type | the enum's integer type (`s64` unless named), and `cast(Enum)` where the enum is needed | each use |
| `foreign-library-directive` | `#foreign_library` and `#foreign_system_library` | `#library` and `#system_library` | each use; the fix is machine applicable |
| `unnamed-library-directive` | `#library,link_always "x";` without a name | `lib :: #library,link_always "x";` | an unnamed `#library` or `#system_library` statement |
| `must-after-return-type` | `-> T #must` | none: remove it and use the result at every call | each `#must` after a return type |
| `ifx-then-semicolon` | `#ifx c then a; else b;` | `#ifx c then a else b;` | a `;` before the `else` of a `#ifx`; the fix is machine applicable |
| `address-of-struct-literal` | `*T.{..}` as the address of a literal | `*(T.{..})` | an unparenthesized `*T.{..}`; the fix is machine applicable |
| `float-exponent-without-fraction` | `1e20` | `1.0e20` | a decimal float with an exponent and no `.`; the fix is machine applicable |
| `struct-align-directive` | `S :: struct #align 64 { .. }` | `member: T #align 64;` on the first member, which gives the struct the same alignment | each `#align` between `struct` and `{` |
| `for-over-array-literal` | `for x: .[1, 2, 3]` | `for x: s64.[1, 2, 3]` | a `for` over an array literal that names no element type |
| `struct-literal-as-dynamic-array` | `a = .{}` or `a = .{count = 2, data = p}` where `a` is a `[..]T` | `a = type_of(a).{..}` or `[..]T.{..}` | an untyped `.{..}` whose expected type is a dynamic array |
| `target-features-directive` | `f :: () -> T #target_features "avx2" { .. }` | none: remove it | each `#target_features` in a procedure header |
| `push-context-without-block` | `push_context ctx run();` | `push_context ctx { run(); }` | a `push_context` whose body is not a block |
| `main-returns-value` | `main :: () -> s64 { return 1; }` | `main :: () { exit(1); }` | the program's `main` has a return type |
| `caller-location-outside-default` | `loc := #caller_location;` in a body | `#location()` for the line itself, or `loc := #caller_location` as a parameter default and pass it on | `#caller_location` anywhere but a parameter's default value |
| `multi-value-in-code-argument` | `f(#code { a, b := 1, 2; })` | `a := 1; b := 2;`, or bind the code to a name first | `a, b := x, y;` in a `#code` block that is a call argument |
| `spread-with-other-varargs` | `f(first, ..rest)` into one variadic parameter | collect every value into one array and pass `f(..all)` | a `..xs` next to other arguments for the same variadic parameter |
| `exists-on-expression` | `#exists(S.Soa(Item, 3).member)` | give the type a name (`Packed :: S.Soa(Item, 3);`) and ask `#exists(Packed.member)` | `#exists` of anything but a name or a dotted chain of names |
| `procedure-cast-to-integer` | `cast(u64) some_procedure` | `cast(u64) cast(*void) some_procedure` | a procedure cast straight to an integer type |
| `address-of-macro-constant` | `*x` in a macro whose parameter `x` the call binds to a number or bool constant | `copy := x;` and take `*copy` | the call binds the parameter to a constant and the body takes its address |
| `positional-literal-defaults` | `.{1, 2}` for `L :: struct { a: s32; b: s32; c: u32 = 7; }`: the values stop before trailing members that have defaults | `.{1, 2, 7}`, or name what you set: `.{a = 1, b = 2}` | a positional literal of a struct that gives fewer values than it has members (every member without a default has a value) |

## When the official compiler rejects it

Only what the official compiler rejects is a leniency.

- **`declaration-hides-import`** is rejected in the program, at the declaration, whether or not it is used (`Redeclaration of ...`, or two overloads with identical argument lists). Inside an imported module the same code compiles, and so does an overload set whose members differ in their parameters. A namespaced import (`S :: #import "M"`) never clashes. The rest of this rule is the tie-break in [procedures](procedures.md) and the lookup rule `scope.18` in [scoping](scoping.md): jaic prefers the module's own declaration, which a library depends on, and the warning is for the program's own files only.
- **`namespaced-for-expansion`** and **`operator-out-of-scope`**: the official compiler cannot see `for_expansion` or an operator through a namespaced import, in the program or in a module.
- **`place-directive`** is a deprecation error; `loose-enum` is `'loose' is not a member of ...`; the two library directives are not directives of the official compiler (`#library`, `#system_library` are), and a library has to be named; `#must` is not accepted anywhere in a procedure header; `#ifx c then a; else b;` is `'else' without 'if'`; `*T.{..}` is read as the pointer type `*T` followed by a literal.

- **`float-exponent-without-fraction`**: the official compiler rejects a decimal float that has an exponent and no fraction. **`struct-align-directive`**, **`target-features-directive`**, **`push-context-without-block`**, **`main-returns-value`** and **`caller-location-outside-default`** are rejected where written; the official `main` has no return type and `#caller_location` is only a parameter default.
- **`for-over-array-literal`** is rejected for a literal with no element type; `for x: s64.[..]` is fine.
- **`struct-literal-as-dynamic-array`** is the untyped `.{..}` for a dynamic array `[..]T`. The same literal for an array view `[] T` is accepted by the official compiler, so it carries no warning.
- **`multi-value-in-code-argument`** is rejected only inside a `#code { }` that is a call argument (the comma ends the block); everywhere else `a, b := x, y;` is fine.
- **`spread-with-other-varargs`**: `..xs` has to be the only argument for its variadic parameter.
- **`exists-on-expression`**: `#exists` takes a name or a chain of names.
- **`procedure-cast-to-integer`**: the official compiler needs the pointer in between.
- **`address-of-macro-constant`** is rejected only when the macro parameter is bound to a constant; a variable argument has an address.
- **`positional-literal-defaults`**: the official compiler reports `Not enough values provided in this struct literal` for any positional literal that stops early, defaults or not. A literal that leaves out a member with no default is an error here too.

Rejected by both compilers, so these are errors with a code of their own (in imported modules too), not leniencies: `implicit-narrowing` (an `ifx` of integer constants into a signed integer narrower than 64 bits, see [casts and conversions](casts-and-conversions.md)), `ambiguous-procedure-value` (see [procedures](procedures.md)) and `shadows-builtin-type` (see [declarations and constants](declarations-and-constants.md)).

Looked at and not a leniency, so there is no warning:

- `#assert(size_of(type_of(local)) == ..)` on a local declared the line before: the official compiler accepts every form tried. Hyperserve's case rejects for another reason.
- `get_plugin` in a file run as a program: it is the `declaration-hides-import` case (the `Program_Print` module exports a `get_plugin` with the same parameters).
- A procedure of the program and an imported one that an untyped literal fits alike (`f(x: float32)` against `f(x: float64)`, called with `1`): both compilers accept it.
- Divergences where jaic fixes a bug of the official compiler (`ifx c 1 else 0` for an unsigned target, recursive structs with a variadic parameter, a procedure returning a procedure type with a body) are decisions to differ, not code the official compiler is expected to accept later, and carry no warning.

## How to change it

- A new leniency: add a variant to `Leniency` (code, `ALL`), warn through `leniency_warning` and add a row above. A construct recognized by spelling goes in the parser (`note_lenient`) and `report_parsed_leniencies`.
- Drop one when the compiler stops accepting it: the warning becomes the error, with the same help.
- Tests: `crates/jaic/tests/leniency_warnings.rs` has one test per code, and each checks that the same text inside an imported module is silent. `tools/ci/check_diagnostics_covered.py` needs each new code named by a test (`warning[code]`).

## Configuration

None. Warnings print unless `-quiet` is given.

## Dependencies

`ast::File::lenient` (parser), `ns.main_module`.

See also: [procedures](procedures.md), [scoping](scoping.md), [operator overloading](operator-overloading.md), [the upstream corpus](../tools/upstream-corpus.md).
