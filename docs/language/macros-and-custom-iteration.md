# Macros and custom iteration

## What it is

`#expand` procedures are inlined at the call site. They reach into the caller with backtick names, `` `return `` and `` `defer ``, take `Code` arguments for `#insert`, and implement `for` over user types through `for_expansion`.

## How it works

`expand_macro` in `sema/calls.rs` inlines the body; `macro_code_args` picks which arguments pass as `Code`. `MacroFrame` in `sema/lower.rs` tracks the expansion stack.

```jai
macro_ret :: (v: int) #expand { if v > 2 { `return v * 10; } }
reads_caller :: () #expand { `counter += 1; }
with_code :: (c: Code) #expand { #insert c; }
with_code(print("inserted\n"));
```

- A backtick `defer` runs at the end of the caller's block; a backtick declaration is visible to the caller. Backticks apply per name: `` status, `it := next(); ``.
- `` #if #exists(`name) `` asks the caller's scope (`body_static_condition`, `sema/stmt.rs`).
- A `Code` variable passed to a `Code` parameter passes its value. `code_of(proc)` returns a procedure's `Code` including its header.

### for_expansion

`for x: value` on a struct finds a `for_expansion` macro for the operand type (`check_for_expansion` in `sema/stmt.rs`). It gets the value (or address), the body as `Code`, and `For_Flags`, and declares `it` and `it_index`:

```jai
Bag :: struct { items: [3] int = .[1, 2, 3]; }
for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.items.count-1 { `it := bag.items[i]; `it_index := i; #insert body; }
}
for b { s += it * 10 + it_index; }
```

`for :walk x, i: b` selects a named expansion; naming the index hides `it_index`; bodies can be forwarded and overloads coexist. `break`/`continue` in the inserted body leave through the expansion's innermost loop and run defers registered inside it (`insert_for_body`), so `defer i += 1;` before `#insert body;` advances on `continue`.

## How to change it

Loop-body plumbing is `ForBody` in `sema/lower.rs`. Tests in `tests/stdlib/`: `backtick-defer-names.jai`, `macro-code-variable-arg.jai`, `for-expansion-*.jai`. `Code` and `For_Flags` come from the preload.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `#expand` procedures are inlined at the call site {#macro.1}. They reach into the caller with backtick names, `` `return `` and `` `defer `` {#macro.2}, take `Code` arguments to `#insert` {#macro.3}, and implement `for` over user types through `for_expansion` {#macro.4}.
- A backtick `defer` runs at the end of the caller's block {#macro.5}. A backtick declaration (`` `thing := New(int); ``) is visible to the caller afterward {#macro.6}.
- Backticks can apply per name: `` status, `it := next(); `` declares `it` in the caller and `status` in the macro (`Decl::backtick_names`) {#macro.13}.
- Inside a macro, `` #if #exists(`name) `` asks the caller's scope (`body_static_condition` in `sema/stmt.rs`), so a macro called twice can declare a caller variable once and assign it after {#macro.14}. Epic_Fail's `assert` relies on this.
- A `Code` variable passed to a `Code` parameter passes its value, not its name {#macro.7}.
- `code_of(proc)` returns the `Code` of a procedure including its header, for macros that rewrite it (yield-jai) {#macro.15}.
- `for x: value` on a struct looks up a `for_expansion` macro for the operand type (`for_expansion_procs`, `check_for_expansion` in `sema/stmt.rs`). The macro gets the value (or its address, if the parameter is a pointer), the loop body as `Code`, and `For_Flags`, which is a compile-time constant. It declares `it` and `it_index` with backticks and inserts the body {#macro.8}:
- `for :walk x, i: b` selects a named expansion `walk` {#macro.9}.
- Naming the index hides the macro's `it_index` from the body {#macro.10}.
- An expansion can forward its body to another one {#macro.11}, and overloads on different types coexist {#macro.12}.
