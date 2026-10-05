# Scoping: visibility, using and conditional declarations

## What it is

Rules for which names a declaration is visible to (file, module or everyone), how `using` pulls names in, and how top-level `#if` selects declarations and files.

## How it works

Visibility directives switch the default for the declarations that follow them in a file:

```jai
answer :: 42;        // exported (default)
#scope_module
module_only :: ...;  // every file of this module, not importers
#scope_file
secret :: 1;         // this file only
```

Verified: `G.module_only()` from another module fails with `module 'Greeter' has no exported member 'module_only'`, while another file of `Greeter` calls it freely. A `#import` under `#scope_file` is visible only in that file; a name unknown elsewhere is finally looked up in sibling files' file-scope imports (`lookup_sibling_file_imports`).

`using`:
- `using Color;` (enum) makes members unqualified (`GREEN`); `using v;` on a local struct makes its fields plain names.
- `using S :: #import "M";` names the module and exposes its members. At export visibility this re-exports them to importers (`tests/stdlib/module-using-import-reexport.jai`); `using global;` does the same for a global (`tests/stdlib/module-using-global.jai`).
- `#import "Math";` inside a procedure body works and its names are visible in the whole file (`hoist_body_imports` in `stmt.rs`).

Top-level conditionals, all verified on macOS:

```jai
#if OS == .MACOS { #load "other.jai"; KIND :: "mac"; } else { KIND :: "other"; }
#if DEBUG #load "sub/thing.jai";
#if #exists(from_other) { ... }
```

`#if` conditions that are plain constants are expanded first, before any `#run`, so a conditional `#load` (and any `#add_context` in it) lands before the Context type is laid out.

- `#this` inside an `#expand` macro (including in a backtick `defer`) is the procedure the macro was expanded into: `E::This` retries `check_this` from each macro frame's caller scope and from `backtick_scope`.
- `type_of(field)` in a procedure nested in a struct names the field's type (`type_field_type` in `sema/calls.rs` walks enclosing struct scopes when the name is not otherwise visible); no value is needed.

## How to change it

Entity creation and visibility are in `declare_stmt` (`crates/jaic/src/sema/modules.rs`; `file_private`, `target_scope`). Lookup is in `crates/jaic/src/sema/scope.rs` (`Found::Using`, `module_lookup`). Conditional expansion is `expand_pending` / `expand_plain_ifs` in the same file; keep `plain_condition` free of calls so the early pass stays safe.

## Configuration

None. `OS` and `CPU` come from `Options::os` / `cpu`; `jaic check -os linux|windows|macos` overrides the OS.

## Dependencies

[modules-and-imports.md](modules-and-imports.md); `consteval.rs` evaluates the conditions.
