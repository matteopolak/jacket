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

Declarations are exported by default {#scope.1}; after `#scope_module` they are visible to every file of the module but not to importers {#scope.2}; after `#scope_file` only to the declaring file {#scope.3}.

`G.module_only()` from another module fails with `` module `Greeter` has no exported member `module_only` ``. A `#import` under `#scope_file` is visible only in that file {#scope.4}, but as a last resort a name unknown everywhere else is looked up in sibling files' file-scope imports (`lookup_sibling_file_imports`).

`using`:

- `using Color;` makes enum members unqualified {#scope.5}; `using v;` on a struct local makes its fields plain names {#scope.6}. See [using](using.md).
- `using S :: #import "M";` names the module and exposes its members {#scope.7}. At export visibility this re-exports them to importers {#scope.8}; `using global;` does the same for a global {#scope.9}.
- `#import "Math";` inside a procedure body works, and its names are visible in the whole file (`hoist_body_imports` in `stmt.rs`) {#scope.10}.

Top-level conditionals:

```jai
#if OS == .MACOS { #load "other.jai"; KIND :: "mac"; } else { KIND :: "other"; }
#if DEBUG #load "sub/thing.jai";
#if #exists(from_other) { ... }
```

A top-level `#if` selects declarations and `#load`s by a constant condition {#scope.11}, with or without braces and with an optional `else` {#scope.12}; `#exists(name)` tests whether a name is declared {#scope.13}.

`#if` conditions that are plain constants are expanded first, before any `#run`, so a conditional `#load` (and any `#add_context` in it) lands before the Context type is laid out {#scope.14}.

Two lookups that cross scopes:

- `#this` inside an `#expand` macro, including in a backtick `defer`, is the procedure the macro expanded into {#scope.15}: `check_this` is retried from each macro frame's caller scope and from `backtick_scope`.
- `type_of(field)` in a procedure nested in a struct names the field's type, with no value needed (`type_field_type` in `sema/calls.rs` walks enclosing struct scopes) {#scope.16}.

## How to change it

Entity creation and visibility: `declare_stmt` in `sema/modules.rs` (`file_private`, `target_scope`). Lookup: `sema/scope.rs` (`Found::Using`, `module_lookup`). Conditional expansion: `expand_pending` and `expand_plain_ifs` in `scope.rs`. Keep `plain_condition` free of calls, or the early pass could run code before the program is ready.

Tests: `tests/stdlib/module-using-import-reexport.jai`, `module-using-global.jai`.

## Configuration

`OS` and `CPU` come from `Options::os` / `cpu`; `jaic check -os linux|windows|macos` overrides the OS.

## Dependencies

[Modules and imports](modules-and-imports.md); `consteval.rs` evaluates the conditions.
