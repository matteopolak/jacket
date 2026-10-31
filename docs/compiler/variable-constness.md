# Variables are not constants

Only `::` and `: T :` declarations name constants. A `:=` or `: T =` variable never does, even when its initializer folds.

```jai
G := 4;
A :: G;          // error: constant declaration needs a constant value, but `G` is a variable
```

## How it works

- Contexts that need a constant (`#if` conditions, `::` values, array sizes, enum values, `$` arguments, `#insert` operands) go through `eval_const` in `sema/decls.rs`. Reading a global variable there is an error.
- Exempt on purpose: `#run` bodies, `#assert`, global initializers (`eval_const_or_run`), `type_of`/`size_of` operands, and address-of (`*G`, `*G.field`).

## How to change it

A new constant context calls `eval_const`, or `eval_const_as` with its own wording. A context that may read variables at compile time uses `eval_const_or_run`.

Test: `crates/jaic/tests/cleanroom_variable_constness.rs`.
