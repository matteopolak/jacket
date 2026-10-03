# Procedures, control flow, data structures and memory

This page specifies callable values, argument and return behaviour, control flow, iteration, allocators, and lifetimes.

## Procedure declarations and calls

A procedure type is written from its parameter list and optional result, for example `(count: s64) -> bool`. A definition adds a body, and procedures are usually bound with `::`. Arguments may be passed positionally or by name; default arguments can be omitted, and varargs are supported. Multiple results are returned as a comma-separated sequence and can be assigned to multiple destinations. A call with multiple return values is not the same as returning a struct.

A typed default (`amount: s64 = 4`) is distinct from an inferred default (`amount := 4`); the latter may participate in polymorphism. Named arguments can skip earlier optional parameters. Results can have defaults and be assigned by name. The evaluation order of side-effecting argument expressions is unspecified.

Ordinary arguments are passed by value, with ABI-dependent transport; large aggregates may be passed indirectly as an implementation detail, and source semantics never depend on whether the backend uses registers or a hidden pointer. Procedures are not closures: a nested procedure cannot capture mutable local variables of its enclosing runtime procedure. Procedures are values and can be passed as callbacks.

`#c_call` selects the C calling convention on a procedure type or declaration. `#foreign` declarations bind to a named `#library` handle or an external symbol name; an unbound foreign import is resolved by the host (for example, a wasm runtime). A foreign declaration describes an ABI boundary and has no Jai body, and a named library must be available to the selected target. Such interfaces use C-compatible fixed-width fields and pointers; Jai's default integer aliases and aggregate layout do not automatically match an arbitrary C declaration.

Lambdas include a concise expression form such as `triple :: x => x * 3`. Lambdas do not capture locals, the same as nested procedures.

## Conditions and branches

Runtime `if` takes a condition and a statement or block; `else` is optional. `then` introduces a compact single statement. `ifx` is an expression form that yields a value, including a block form and an optional else. `if` can bind a value in the condition, so a non-null value is tested and named in the body.

`if x == { case ...; }` switches on enum or discriminant values and supports grouped cases. `#if` is compile-time selection and requires a constant condition. Unlike runtime `if`, discarded branches are not type-checked as ordinary program branches, though their syntax must still parse. This permits target-specific declarations. `#if` branches do not introduce a nested scope.

## Loops, defer, and operators

`while` loops support `break` and `continue`; labeled loops can target an enclosing loop. `defer statement;` schedules the statement for the end of the current scope, including exits through `continue` or `break`. Placement matters: a defer reached only after a guard does not run on a break that happens before it.

In `case` arms, `#through;` explicitly falls through to the next arm; cases do not fall through by default. `#char "x"` converts a one-character literal to a byte value. `#no_context` marks procedures that do not receive the implicit context; it is a signature and call contract.

`for` has range and iterable forms. The iterable form exposes `it`, `it_index`, and optional named loop variables. The `<` prefix iterates in reverse, and `*` iterates by pointer. Custom `for_expansion` hooks change iteration; see the metaprogramming page. A range `a..b` includes both endpoints. Empty and reversed range behaviour is unspecified.

The double-comma argument form (`callee(args,, field=value)`) overrides context fields for that call only, typically the allocator or logger; the override ends with the call. Logging goes through a context-selected callback with opaque callback data and message metadata. This is a callback convention built on the context, not control-flow syntax.

Operators include arithmetic, comparisons, bitwise operations, logical `&&`/`||`, unary `!`, and compound assignment. User types can overload operators, including array operations. Precedence of uncommon operator combinations is unspecified.

## Struct composition and tagged data

Structs, arrays, unions, and enums are specified on the core types page. `using` promotes the members of a contained field into the containing scope, and struct members are addressed by name. A `this`-style parameter is ordinary parameter syntax for code that operates on an object; there are no implicit methods or dynamic dispatch.

## Context and allocation

The implicit `context` is a dynamically passed environment carrying the allocator and other execution state. Context values can be replaced for a lexical region, and allocators are user-definable procedures. A context override affects every operation performed while it is active, including library code that reads the implicit context; it is not a global mutable singleton.

Memory is explicit. The standard library provides default allocation, temporary storage, resizable arrays, pools, and manual `free`. Temporary allocations live until temporary storage is reset and suit short-lived strings or scratch work; a value backed by temporary storage is valid only while that storage is. Heap allocations need a clear owner and cleanup; `defer free(...)` is the usual structured-cleanup pattern.

An allocator is a procedure plus a data pointer. The procedure takes a mode selecting allocate, resize, free, lifecycle, or heap creation and destruction. Allocation may return null when storage cannot be provided, and misuse may be a hard allocator error. This protocol is a runtime API contract, not syntax.

`#no_reset` marks globals whose compile-time-initialized contents are kept in the program's output instead of being reinitialized. Its interaction with multiple workspaces, target startup, and repeated compiler runs is unspecified.

Build options can enable per-procedure linked stack-trace nodes for instrumentation. Inline assembly uses `#asm` blocks with typed operands, register names, and instruction-selection rules. Assembly is x86-64 specific, and registers declared inside a block are visible in its surrounding scope. The compiler's asm documentation specifies the instruction syntax.

## Implementation checklist

- Keep multiple return values a calling-convention construct; do not wrap them in an aggregate.
- Keep compile-time and runtime branches distinct, with declarations visible from the selected `#if` arm.
- Model `defer` as scope-exit behaviour, not a statement placed after the body.
- Propagate the context explicitly in the call and runtime design even where the source omits a context argument.
- Track view and pointer lifetimes conservatively; resizing or freeing may invalidate aliases.
- Argument evaluation order, range iteration corner cases, and the full `this` syntax are unspecified.
