# Scoping: visibility, using and conditional declarations

## What it is

Which code can see a declaration (file, module, everyone), how `using` pulls names in, and how top-level `#if` selects declarations and files.

## How it works

```jai
answer :: 42;        // exported (the default)
#scope_module
module_only :: ...;  // every file of this module, not importers
#scope_file
secret :: 1;         // this file only

#if OS == .MACOS { #load "other.jai"; KIND :: "mac"; } else { KIND :: "other"; }
#if #exists(from_other) { ... }
```

- Module-private and file-private names fail importer lookup with a missing-export diagnostic, including qualified `M.name`. A `#import` under `#scope_file` is visible only in that file. Each `#load`ed file starts as exported.
- `using Color;` makes enum members unqualified; `using v;` on a struct local makes its fields plain ([using](using.md)). `using S :: #import "M";` names the module and exposes its members; at export visibility this re-exports them. An `#import` inside a procedure body joins the module's imports, like a top-level anonymous one, so it is visible in every file of the module (`hoist_body_imports`, `sema/stmt.rs`).
- A top-level `#if` selects declarations and `#load`s by a constant condition, with or without braces. Plain-constant conditions expand first, before any `#run`, so a conditional `#load` (and its `#add_context`) lands before the Context type is laid out.
- `#this` inside an `#expand` macro, including a backtick `defer`, is the procedure the macro expanded into (`check_this` retried per macro frame and `backtick_scope`). `type_of(field)` in a procedure nested in a struct names the field's type with no value (`type_field_type`, `sema/calls.rs`).

## How to change it

Entities and visibility: `declare_stmt` in `sema/modules.rs` (`file_private`, `target_scope`). Lookup: `sema/scope.rs` (`Found::Using`, `module_lookup`). Conditional expansion: `expand_pending`, `expand_plain_ifs`; keep `plain_condition` free of calls so the early pass cannot run code too soon. `consteval.rs` evaluates conditions. Tests: `tests/stdlib/module-using-import-reexport.jai`, `module-using-global.jai`.

Configuration: `OS` and `CPU` come from `Options::os` / `cpu`; `jaic -check -os linux|windows|macos` overrides the OS. See [modules and imports](modules-and-imports.md).

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Declarations are exported by default {#scope.1}; after `#scope_module` they are visible to every file of the module but unavailable to importers, including qualified `M.name` lookup {#scope.2}; after `#scope_file` only to the declaring file {#scope.3}.
- Both module-private and file-private names fail importer lookup with a missing-export diagnostic. A `#import` under `#scope_file` is visible only in that file {#scope.4}; sibling files cannot use its names. Each `#load`ed file starts with export visibility independently of its caller's active directive.
- `using Color;` makes enum members unqualified {#scope.5}; `using v;` on a struct local makes its fields plain names {#scope.6}. See [using](using.md).
- `using S :: #import "M";` names the module and exposes its members {#scope.7}. At export visibility this re-exports them to importers {#scope.8}; `using global;` does the same for a global {#scope.9}.
- `#import "Math";` inside a procedure body works, and its names are visible in every file of the module, not just the one that wrote it (`hoist_body_imports` in `stmt.rs`) {#scope.10}.
- A top-level `#if` selects declarations and `#load`s by a constant condition {#scope.11}, with or without braces and with an optional `else` {#scope.12}; `#exists(name)` tests whether a name is declared {#scope.13}.
- `#if` conditions that are plain constants are expanded first, before any `#run`, so a conditional `#load` (and any `#add_context` in it) lands before the Context type is laid out {#scope.14}.
- `#this` inside an `#expand` macro, including in a backtick `defer`, is the procedure the macro expanded into {#scope.15}: `check_this` is retried from each macro frame's caller scope and from `backtick_scope`.
- `type_of(field)` in a procedure nested in a struct names the field's type, with no value needed (`type_field_type` in `sema/calls.rs` walks enclosing struct scopes) {#scope.16}.
