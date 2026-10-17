# Compiler builtins

## What it is

The `#compiler` primitives that stdlib calls to reach the Rust compiler (`__jaic_workspace_*`, `__jaic_syntax_*`). Anything they select, a workspace option or a syntax attribute, is an enum, not a string, so a misspelt name is a Jai type error.

## How it works

Each builtin receives the enum member's integer value. `crates/jaic/src/builtin_keys.rs` declares the same enums in Rust (`raw_enum!`), and `from_raw` rejects values outside them.

| Builtin | Selector enum (Jai) | Rust |
| --- | --- | --- |
| `__jaic_workspace_set_text(ws, option, text)` | `Workspace_Text` | `WorkspaceText` |
| `__jaic_workspace_set_flag(ws, option, bool)` | `Workspace_Flag` | `WorkspaceFlag` |
| `__jaic_workspace_set_number(ws, option, s64)` | `Workspace_Number` | `WorkspaceNumber` |
| `__jaic_workspace_clear_list` / `_append_list` | `Workspace_List` | `WorkspaceList` |
| `__jaic_workspace_destroy(ws)` | none | none |
| `__jaic_syntax_int/string/child(node, key, ...)` | `Syntax_Key` | `SyntaxKey` |

Number options carry a `Build_Options` enum by value (`output_type`, `os_target`, the check levels, the optimization settings...). Rust decodes them into typed enums (`OutputType`, `OsTag`, `CheckLevel`...) and stores the typed value in `BuildSettings`; nothing downstream compares names.

`set_build_options` in `stdlib/internal/compiler-api/workspaces.jai` rejects an `os_target` or `cpu_target` jaic cannot build for with ``jaic cannot build for `os_target = .PS5` ``.

Syntax attributes are record fields named `__syntax_<member>` (derived from `SyntaxKey`'s name by `Name::syntax`); `code_export.rs` writes them with `SyntaxKey`, and `syntax_int/string/child` in `stdlib/internal/program-print/attributes.jai` read them.

## How to change it

Add a member to both declarations, with the same numeric value:

- workspace option: `Workspace_*` in `stdlib/internal/compiler-api/primitives.jai`, the Rust enum in `builtin_keys.rs`, the handler in `Workspaces::set_*` (`crates/jaic/src/build.rs`), and the `set_build_options` forward.
- syntax attribute: `Syntax_Key` in `attributes.jai`, `SyntaxKey` in `builtin_keys.rs`, then use it in `code_export.rs` and the printer.

`crates/jaic/tests/typed_builtin_keys.rs` parses the stdlib declarations and fails when a Rust enum drifts from them (names and values).

## Configuration

None.

## Dependencies

`stdlib/internal/compiler-api`, `stdlib/internal/program-print`, `crates/jaic/src/records.rs` (record field names).
