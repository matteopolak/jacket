# Macros and custom iteration

## What it is

`#expand` procedures are inlined at the call site {#macro.1}. They reach into the caller with backtick names, `` `return `` and `` `defer `` {#macro.2}, take `Code` arguments to `#insert` {#macro.3}, and implement `for` over user types through `for_expansion` {#macro.4}.

## How it works

`expand_macro` in `sema/calls.rs` inlines the body; `macro_code_args` decides which arguments are passed as `Code` instead of evaluated. `MacroFrame` in `sema/lower.rs` tracks the expansion stack.

```jai
macro_ret :: (v: int) #expand { if v > 2 { `return v * 10; } }
outer :: () -> int { macro_ret(5); return 1; }        // outer() == 50

reads_caller :: () #expand { `counter += 1; }          // bumps the caller's local `counter`
with_code :: (c: Code) #expand { #insert c; }
with_code(print("inserted\n"));                        // also accepts { ... } blocks
```

- A backtick `defer` runs at the end of the caller's block {#macro.5}. A backtick declaration (`` `thing := New(int); ``) is visible to the caller afterward {#macro.6}.
- Backticks can apply per name: `` status, `it := next(); `` declares `it` in the caller and `status` in the macro (`Decl::backtick_names`) {#macro.13}.
- Inside a macro, `` #if #exists(`name) `` asks the caller's scope (`body_static_condition` in `sema/stmt.rs`), so a macro called twice can declare a caller variable once and assign it after {#macro.14}. Epic_Fail's `assert` relies on this.
- A `Code` variable passed to a `Code` parameter passes its value, not its name {#macro.7}.
- `code_of(proc)` returns the `Code` of a procedure including its header, for macros that rewrite it (yield-jai) {#macro.15}.

### for_expansion

`for x: value` on a struct looks up a `for_expansion` macro for the operand type (`for_expansion_procs`, `check_for_expansion` in `sema/stmt.rs`). The macro gets the value (or its address, if the parameter is a pointer), the loop body as `Code`, and `For_Flags`, which is a compile-time constant. It declares `it` and `it_index` with backticks and inserts the body {#macro.8}:

```jai
Bag :: struct { items: [3] int = .[1, 2, 3]; }
for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.items.count-1 { `it := bag.items[i]; `it_index := i; #insert body; }
}
for b { s += it * 10 + it_index; }    // s == 63
```

- `for :walk x, i: b` selects a named expansion `walk` {#macro.9}.
- Naming the index hides the macro's `it_index` from the body {#macro.10}.
- An expansion can forward its body to another one {#macro.11}, and overloads on different types coexist {#macro.12}.
- `break` and `continue` in the inserted body leave through the expansion's innermost loop and run the defers the expansion registered inside it, like a `continue` written in that loop (`insert_for_body`). So `defer i += 1;` before `#insert body;` is the way to advance on `continue`.

## How to change it

Loop-body plumbing (`break`, `continue` and `remove` as inserted bindings) is `ForBody` in `sema/lower.rs`. Tests live in `tests/stdlib/`, one behavior each with a header comment stating the rule: `backtick-defer-names.jai`, `macro-code-variable-arg.jai`, `for-expansion-renamed-index.jai`, `for-expansion-forwarded-body.jai`, `for-expansion-mixed-overloads.jai`, `for-expansion-defer-continue.jai`.

## Dependencies

The preload's `Code` and `For_Flags` definitions.
