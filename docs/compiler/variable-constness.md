# Variables are not constants

**What it is.** Only `::` and `: T :` declarations name constants. A `:=` or `: T =` variable never does, even when its initializer folds.

**How it works.** `eval_const` / `eval_const_as` (`sema/decls.rs`) check the expression in a `ConstInit` thunk whose `FnCtx.constant_only` holds the start of the error message. `check_ident` (`sema/expr.rs`) fails a global variable read there with `<what>, but `G` is a variable`:

- `#if` conditions: `#if condition must be a constant`
- `::` values (global and local): `constant declaration needs a constant value`
- array sizes, enum values, `$` arguments, `#insert` operands: `this must be a constant expression`

Exempt on purpose:

- `#run` bodies get their own `Run` thunk and may read and write globals.
- `#assert` and global variable initializers (`eval_const_or_run`) evaluate with variables allowed, since they execute code at compile time.
- `type_of` / `size_of` check their operand without emitting, so `type_of(G)` is fine.
- `*G` and `*G.field` take an address, which is constant (`names_variable_path` in `check_unary`).
- `using global;` resolves its target through `eval_const_or_run`.

**How to change it.** A new constant context should call `eval_const` (or `eval_const_as` with its own wording). A context that must read variables at compile time should use `eval_const_or_run`.

**Configuration / dependencies.** None. Tests: `crates/jaic/tests/cleanroom_variable_constness.rs`.
