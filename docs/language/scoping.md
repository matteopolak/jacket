# Scoping: visibility, using and conditional declarations

Which code can see a declaration (file, module, everyone), how `using` pulls names in, and how top-level `#if` selects declarations and files.

```jai
answer :: 42;        // exported (the default)
#scope_module
module_only :: ...;  // every file of this module, and qualified M.module_only
#scope_file
secret :: 1;         // this file only

#if OS == .MACOS { #load "other.jai"; KIND :: "mac"; } else { KIND :: "other"; }
#if #exists(from_other) { ... }
```

## Visibility

- Declarations are exported by default {#scope.1}. After `#scope_module` they're visible to every file of the module and through qualified `M.name`, but don't enter importers' unqualified scope {#scope.2}. After `#scope_file` only the declaring file sees them {#scope.3}.
- File-private names fail importer lookup with a missing-export error. A `#import` under `#scope_file` is visible only in that file {#scope.4}. Each `#load`ed file starts as exported, whatever its caller's directive.
- `#import "Math";` inside a procedure body works, and its names are visible in every file of the module {#scope.10}.
- A name that isn't a procedure, declared in a module, wins over what one of its files imports, though the file is nearer: the module's own `errno` variable isn't hidden by an imported `errno` procedure {#scope.18}. Procedures from both sides form one overload set, where the calling module's is preferred ([procedures](procedures.md)). The official compiler rejects the clash in the program's own files, so only those warn (`declaration-hides-import`, [leniencies](leniencies.md)).
- A procedure declared inside a procedure hides module-level and imported procedures of its name; used as a value (`apply(sin, 1.0)`) it is chosen without ambiguity {#scope.20}.
- A constant, procedure or type declared in a body is visible only below its declaration: using it above is `` error: `x` is used above its declaration `` (`use-before-declaration`, in modules too) {#scope.19}. This covers two local procedures that call each other, type positions, a constant using a later one, `#run`, `#assert`, `#if` and `#insert` text. A use inside a procedure nested in another local procedure isn't checked.
- Struct bodies are declarative: field types, constant defaults and generated members may refer to later enclosing constants or generators. A generic struct's parameter types and defaults follow declaration order at the instantiation site. Method bodies keep their own local order, and runtime locals cannot become compile-time field bounds.

## using

- `using Color;` makes enum members unqualified {#scope.5}; `using v;` on a struct local makes its fields plain names {#scope.6}. See [using](using.md).
- `using S :: #import "M";` names the module and exposes its members {#scope.7}. At export visibility this re-exports them to importers {#scope.8}; `using global;` does the same for a global {#scope.9}.

## Conditional declarations

- A top-level `#if` selects declarations and `#load`s by a constant condition {#scope.11}, with or without braces and with an optional `else` {#scope.12}. `#exists(name)` tests whether a name is declared {#scope.13}.
- `#exists(query, other)` answers for `query` alone; `other` must itself be valid, and a bad one is an error, not `false` {#scope.17}.
- Plain-constant conditions expand first, before any `#run`, so a conditional `#load` (and its `#add_context`) lands before the Context type is laid out {#scope.14}.
- `#this` inside an `#expand` macro, including a backtick `defer`, is the procedure the macro expanded into {#scope.15}.
- `type_of(field)` in a procedure nested in a struct names the field's type with no value needed {#scope.16}.

## How to change it

Visibility is `declare_stmt` in `sema/modules.rs` (`file_private`, `target_scope`). Lookup is `sema/scope.rs` (`Found::Using`, `module_lookup`). Conditional expansion is `expand_pending`/`expand_plain_ifs`; keep `plain_condition` free of calls so the early pass can't run code too soon. `OS` and `CPU` come from `Options::os`/`cpu`; `jaic -check -os linux|windows|macos` overrides the OS. See [modules and imports](modules-and-imports.md).

Deferred generic headers store their instantiation span on the parameter scope. `note_use_before_declaration` in `sema/calls/lenient.rs` follows scope boundaries: struct bodies stop the enclosing statement-order check, and parameter scopes substitute their instantiation site. Change these rules together with the local struct tests in `use_before_declaration.rs`, and keep lexical lookup and runtime-capture checks intact.
