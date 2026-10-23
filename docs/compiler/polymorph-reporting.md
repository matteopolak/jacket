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

## Configuration

Workspace folders and saved `[project] entry` / `import_path` determine the programs searched. The current program targets, workspace trust and existing compile-time execution budget apply. No filesystem-wide search or speculative execution of other target branches is added.

## Dependencies

Uses compiler editor facts, module/type identity, project discovery, observed build workspaces, the semantic analysis cache and its read tracking, and `workspace/codeLens/refresh` when the client supports it. There are no additional external dependencies.
