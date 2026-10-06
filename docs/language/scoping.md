# Scoping: visibility, using and conditional declarations

## What it is

Which code can see a declaration (file, module, or everyone), how `using` pulls names in, and how top-level `#if` selects declarations and files.

## How it works

Visibility directives set the default for the declarations that follow them in a file:

```jai
answer :: 42;        // exported (the default)
#scope_module
module_only :: ...;  // every file of this module, not importers
#scope_file
secret :: 1;         // this file only
```

`G.module_only()` from another module fails with `module 'Greeter' has no exported member 'module_only'`. A `#import` under `#scope_file` is visible only in that file, but as a last resort a name unknown everywhere else is looked up in sibling files' file-scope imports (`lookup_sibling_file_imports`).

`using`:

- `using Color;` makes enum members unqualified; `using v;` on a struct local makes its fields plain names. See [using](using.md).
- `using S :: #import "M";` names the module and exposes its members. At export visibility this re-exports them to importers; `using global;` does the same for a global.
- `#import "Math";` inside a procedure body works, and its names are visible in the whole file (`hoist_body_imports` in `stmt.rs`).

Top-level conditionals:

```jai
#if OS == .MACOS { #load "other.jai"; KIND :: "mac"; } else { KIND :: "other"; }
#if DEBUG #load "sub/thing.jai";
#if #exists(from_other) { ... }
```

`#if` conditions that are plain constants are expanded first, before any `#run`, so a conditional `#load` (and any `#add_context` in it) lands before the Context type is laid out.

Two lookups that cross scopes:

- `#this` inside an `#expand` macro, including in a backtick `defer`, is the procedure the macro expanded into: `check_this` is retried from each macro frame's caller scope and from `backtick_scope`.
- `type_of(field)` in a procedure nested in a struct names the field's type, with no value needed (`type_field_type` in `sema/calls.rs` walks enclosing struct scopes).

## How to change it

Entity creation and visibility: `declare_stmt` in `sema/modules.rs` (`file_private`, `target_scope`). Lookup: `sema/scope.rs` (`Found::Using`, `module_lookup`). Conditional expansion: `expand_pending` and `expand_plain_ifs` in `scope.rs`. Keep `plain_condition` free of calls, or the early pass could run code before the program is ready.

Tests: `tests/stdlib/module-using-import-reexport.jai`, `module-using-global.jai`.

## Configuration

`OS` and `CPU` come from `Options::os` / `cpu`; `jaic check -os linux|windows|macos` overrides the OS.

## Dependencies

[Modules and imports](modules-and-imports.md); `consteval.rs` evaluates the conditions.
