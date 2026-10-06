# Macros and custom iteration

## What it is

`#expand` procedures are expanded inline at the call site. They can reach into the caller with backtick names, `` `return `` and `` `defer ``, take `Code` arguments to `#insert`, and implement `for` over user types with `for_expansion`.

## How it works

`expand_macro` in `crates/jaic/src/sema/calls.rs` inlines the body; `macro_code_args` decides which arguments are passed as `Code` rather than evaluated. Macro frames are tracked by `MacroFrame` in `sema/lower.rs`.

```jai
macro_ret :: (v: int) #expand { if v > 2 { `return v * 10; } }
outer :: () -> int { macro_ret(5); return 1; }        // outer() == 50

swap_in :: (a: *int, b: *int) #expand { t := a.*; a.* = b.*; b.* = t; }
reads_caller :: () #expand { `counter += 1; }          // bumps the caller's local `counter`
with_code :: (c: Code) #expand { #insert c; }
with_code(print("inserted\n"));                        // also accepts { ... } blocks
```

A backtick `defer` schedules cleanup in the caller's block, and a backtick declaration is visible to the caller afterward (`tests/stdlib/backtick-defer-names.jai`: `` `thing := New(int) `` then `thing.* = 9` in the caller, freed at the end of the caller's block). A `Code` variable passed to a `Code` parameter passes its value, not its name (`tests/stdlib/macro-code-variable-arg.jai`).

`for x: value` over a struct looks up a `for_expansion` macro visible for the operand type (`for_expansion_procs`, `check_for_expansion` in `sema/stmt.rs`). The macro receives the value (or its address when declared with a pointer parameter), the loop body as `Code`, and `For_Flags`; it defines `it` and `it_index` with backtick and inserts the body:

```jai
Bag :: struct { items: [3] int = .[1, 2, 3]; }
for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.items.count-1 { `it := bag.items[i]; `it_index := i; #insert body; }
}
for b { s += it * 10 + it_index; }    // s == 63
```

A named expansion is selected with `for :walk x, i: b ...` (`walk` has the same shape; with a reversed inner loop and `* 10`, the output was `30 2 20 1 10 0`). Naming the index hides the macro's `it_index` from the body (`for-expansion-renamed-index.jai`); an expansion may forward its body to another (`for-expansion-forwarded-body.jai`); several overloads on different types coexist (`for-expansion-mixed-overloads.jai`).

Backticks may be per name in a multi-declaration: `` status, `it := next(); `` declares `it` in the caller and `status` in the macro (`Decl::backtick_names`). Inside a macro, `` #if #exists(`name) `` asks the caller's scope (`body_static_condition` in `sema/stmt.rs`), so a macro called twice can declare a caller variable once and assign it afterwards (Epic_Fail's `assert`). `code_of(proc)` gives the `Code` of a procedure, header included, for macros that rewrite it (yield-jai).

## How to change it

- `break`/`continue` in an inserted for body leave through the expansion's innermost loop and run the defers the expansion registered inside it (`defer i += 1;` before `#insert body;`), like a `continue` written in that loop (`insert_for_body` in `sema/stmt.rs`, `tests/stdlib/for-expansion-defer-continue.jai`).
- Loop-body plumbing (`break`/`continue`/`remove` as inserted bindings) is in `sema/lower.rs` (`ForBody`, around the "`#insert (break=..., continue=..., remove=...)`" comment).
- For `for_expansion`, `flags` is a compile-time constant (`sema/calls.rs`, "A for_expansion's `flags`").
- Tests go in `tests/stdlib/*.jai` with a header comment stating the rule; keep each to one behavior.

## Configuration

None.

## Dependencies

`Basic` (`print`, `New`) and the preload `Code`/`For_Flags` definitions.
