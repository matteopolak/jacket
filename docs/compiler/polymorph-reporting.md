# Polymorph reporting

## What it is

Generic procedure lenses show concrete specializations selected by calls and procedure values across the workspace. Clicking the lens lists all resolved bindings; only the preview is shortened.

## How it works

The compiler records selected instances before call-site merging and display budgets. Overload trials and editor sketches do not count. Identities include declaration locations, module parameters, nominal type origins and typed baked values, rather than printed type names. Equal display labels are disambiguated with source locations or module parameters. Summaries are frozen before lowered constant images are released.

The server discovers configured or inferred entry programs, checks closed callers too, and merges every compiler a build metaprogram creates. Declaration location does not limit the search: shared imports and stdlib participate. A built child is answered through its build metaprogram rather than checked independently without its generated declarations.

Lightweight per-program summaries survive eviction from the two-compiler analysis cache. Each summary keeps the paths its compile read (`semantic::Reads`) and a hash of the open texts among them. An edit makes a summary stale, never drops it: a buffer it read changed, closed or appeared, or a file it read changed on disk. Files created or deleted on disk make every summary stale; a `.toml` change drops them all, since it can change which programs exist.

A lens request for a file:

- compiles programs with no summary, and stale programs whose compile read the file;
- leaves stale programs that never read the file to `Session::background_work`, which the server runs while idle, one program at a time. Their old summaries have no rows for the file, but an edit may have made them load it, so the answer is labelled incomplete until they are done. The server then asks the client to refresh code lenses.

Summaries come from the editor's own compile when it matches the open texts (`Cache::cached`), else from a detached compile that does not evict the editor's (`Cache::compile_detached`). An incomplete analysis labels its counts as incomplete instead of claiming no instances exist.

## How to change it

Compiler selection hooks and portable keys live in `jaic::ide::polymorphs`; aggregate before per-site facts discard repeated selections. Keep nominal identity separate from display text. Server aggregation and dependency snapshots live in `jai-language-server::polymorphs` (`Session::polymorph_need` decides what a request compiles). Do not increase the full-compiler cache to retain counts, and do not compile programs that cannot contribute to the requested file on the request path. Regression coverage (`tests/polymorph_instances.rs`) includes more programs than that cache holds, and counts compiles to check that an edit recompiles only the programs that read it.

## Inference failure notes

When a call cannot infer `$T`, `sema/calls/poly_hints.rs` adds located notes to the error (`could not infer`, `conflicting types`, or a pattern mismatch such as `expected an array`). The message text is unchanged.

- Unbound: a note at the `$T` that declares it, a note at each argument passed for that parameter that gave nothing, and help `T = s64` unless `$T` is only an array count or a parameter has that name. The `name = type` spelling was compiled against `..$T`, `[$N] $T` and `[] $T` parameters.
- Conflict: `infer_pattern_param` records the first argument that bound each variable (`Inference::binders`); the error gets a note at both arguments with their types, and help to `cast` one. An explicit `T = ...` does not convert a conflicting argument, so it is not suggested there.
- Pattern mismatch: a note at the argument with its type and the pattern as written.

- Literal arguments: `arg_cost_for` (`calls/matching.rs`) adds notes to "argument cannot be inferred as parameter type" (a `.{...}`, `.[...]` or `.NAME` argument, which has no type of its own, next to a parameter that cannot take it): a note at the parameter with its type, a note at the argument saying what kind of literal it is, and help to pass a value of the parameter's type. `poly_hints::CANNOT_INFER` is the message prefix it recognizes. When a `.{...}` literal is the one that does not fit, the message ends with the reason (`LitMisfit`, from `struct_lit_fit`): an unknown member, a value that does not fit its member, too many values, or not enough (naming the members left without a value). The "no overload matches" notes print this message per candidate, so they carry the reason too.
- Array lengths: when `[N] T` follows a `[$N] T` parameter and the arguments' fixed lengths differ, the "does not match parameter type" error (`with_length_notes`) gets a note at the argument that bound `$N` with its length, a note at the failing argument with its length, and help to pass arrays of the same length. Element-type mismatches keep the plain error.

No edits are offered; the right type is the author's choice.

## Configuration

Workspace folders and saved `[project] entry` / `import_path` determine the programs searched. The current program targets, workspace trust and existing compile-time execution budget apply. No filesystem-wide search or speculative execution of other target branches is added.

## Dependencies

Uses compiler editor facts, module/type identity, project discovery, observed build workspaces, the semantic analysis cache and its read tracking, and `workspace/codeLens/refresh` when the client supports it. There are no additional external dependencies.
