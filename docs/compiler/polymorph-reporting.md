# Polymorph reporting

Generic procedure code lenses show the concrete specializations selected by calls and procedure values across the workspace. Clicking a lens lists every binding; only the preview is shortened.

## How it works

- The compiler records selected instances before call-site merging and display budgets. Overload trials and editor sketches don't count. Identity uses declaration locations, module parameters, nominal type origins and baked values, not printed type names; equal labels get a location or module parameter.
- The server finds configured or inferred entry programs, checks closed callers too, and merges every compiler a build metaprogram creates. Shared imports and stdlib participate.
- Per-program summaries outlive eviction from the two-compiler cache. Each keeps the paths its compile read and a hash of the open texts among them. An edit makes a summary stale, never drops it; a `.toml` change drops all.
- A lens request compiles programs with no summary and stale ones that read the file. Other stale programs go to `Session::background_work`; the answer is labelled incomplete until they finish, then the client is asked to refresh lenses.

## How to change it

- Compiler hooks and portable keys: `jaic::ide::polymorphs`. Aggregate before per-site facts discard repeats, and keep nominal identity apart from display text.
- Server aggregation and dependency snapshots: `jai-language-server::polymorphs` (`Session::polymorph_need`).
- Don't grow the compiler cache to keep counts, and don't compile programs that can't contribute to the requested file. `tests/polymorph_instances.rs` counts compiles to check this.

## Inference failure notes

When a call can't infer `$T`, `sema/calls/poly_hints.rs` adds located notes to the error; the message itself is unchanged. No edits are offered.

- Unbound: notes at the `$T` and at arguments that gave nothing, plus help `T = s64`.
- Conflict: notes at both arguments with their types, and help to `cast` one.
- Pattern mismatch: a note at the argument with its type and the pattern.
- Literal arguments (`arg_cost_for`, `calls/matching.rs`): notes on the parameter and the literal, and for `.{...}` the reason it doesn't fit (unknown member, wrong value, too many or too few).
- Array lengths: when `[N] T` follows `[$N] T` and lengths differ, notes at both arguments with their lengths.
