# Polymorph reporting

## What it is

Generic procedure lenses show concrete specializations selected by calls and procedure values across the workspace. Clicking the lens lists all resolved bindings; only the preview is shortened.

## How it works

The compiler records selected instances before call-site merging and display budgets. Overload trials and editor sketches do not count. Identities include declaration locations, module parameters, nominal type origins and typed baked values, rather than printed type names. Equal display labels are disambiguated with source locations or module parameters. Summaries are frozen before lowered constant images are released.

The server discovers configured or inferred entry programs, checks closed callers too, and merges every compiler a build metaprogram creates. Declaration location does not limit the search: shared imports and stdlib participate. A built child is answered through its build metaprogram rather than checked independently without its generated declarations.

Lightweight per-program summaries survive eviction from the two-compiler analysis cache. Source dependencies are validated on request; source edits invalidate readers, while configuration and topology changes invalidate discovery. An incomplete analysis labels its counts as incomplete instead of claiming no instances exist.

## How to change it

Compiler selection hooks and portable keys live in `jaic::ide::polymorphs`; aggregate before per-site facts discard repeated selections. Keep nominal identity separate from display text. Server aggregation and dependency snapshots live in `jai-language-server::polymorphs`; do not increase the full-compiler cache to retain counts. Regression coverage includes more programs than that cache holds.

## Configuration

Workspace folders and saved `[project] entry` / `import_path` determine the programs searched. The current program targets, workspace trust and existing compile-time execution budget apply. No filesystem-wide search or speculative execution of other target branches is added.

## Dependencies

Uses compiler editor facts, module/type identity, project discovery, observed build workspaces and the existing semantic analysis cache. There are no additional external dependencies.
