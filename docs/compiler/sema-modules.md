# Sema: module loading and top-level expansion

## What it is

How `crates/jaic/src/sema` loads modules, binds module parameters, and expands top-level `#if`, `#insert` and `#run` items. Mostly `modules.rs` (loading, `expand_all`) and `scope.rs` (lookup, `expand_pending`). The user-facing view is [modules and imports](../language/modules-and-imports.md).

## How it works

### Loading and parameters

`load_module` keys a module by entry path and parameter values. The first `#module_parameters (...)` list is part of the instance key; the second holds program parameters (`Module.program_params`), shared by every instance.

`#import "Basic"()(MEMORY_DEBUGGER = DEBUG)` records a setting in `Compiler::program_param_settings` (entry path, name, value expression, importing scope). `apply_program_params` patches the parameter's declaration value as soon as both the instance and the setting exist, before anything reads it. Two different settings are an error. A scalar argument for a typed parameter becomes the declaration's literal value, keeping the declared type. Details: [module parameters](../language/module-parameters.md).

### Member lookup

Every way of reaching a module's names goes through `module_lookup` in `scope.rs`: a plain `#import`, `M.name`, `using M;`, re-exporting modules, sibling-file imports and IDE completion. It returns declarations, re-exports, exported `using` members (`Found::Using`) and module parameters.

This is deliberate. Separate per-path lookups once diverged: `GL.glViewport` failed while a plain `glViewport` worked, because only one path knew about GL's `using gl;` procedure table. `tests/stdlib/module-member-resolution.jai` checks that the paths agree.

- A module-level `using global;` outside `#scope_file` is recorded in `Module.exported_usings`. An exported `using X :: #import "Y";` re-exports `Y`'s names (`Module.exported_using_imports`, followed with a cycle guard).
- `module_declarations` sees only the module's own declarations and re-exports. Use it only for fixed compiler names (`Context`, `__arithmetic_overflow`), never for user-written names.
- As a last resort an unknown name is looked up in the `#scope_file` imports of the module's other files (`lookup_sibling_file_imports`).

### Top-level expansion

`expand_all` first makes a pass over every scope that expands imports and `#if` items whose condition is a plain constant (`expand_plain_ifs`: no calls, no `#run`). After that, lookups expand pending items lazily (`expand_pending`).

`expand_all` runs again after each source a metaprogram adds, so it visits only the module and file scopes in `Compiler::unsettled`, in scope order. `new_scope` adds every module and file scope there, `push_pending` and `push_import` add the scope they queue work for, and `expand_all` drops a scope once every item in it is `Done` and every import is loaded (`settled`). Named imports (`X :: #import`) are resolved from `named_imports_done` onward. Without this, a metaprogram that adds code at every `TYPECHECKED_ALL_WE_CAN` paid for all scopes and entities so far on each round, quadratic overall.

The early pass exists so that a module's `#if FLAG #load "x.jai"`, and any `#add_context` in it, lands before a `#run` lays out the Context. A static `#if X == { case ...; }` counts as plain when its value and every case are plain (ui_builder picks a backend this way). While checking a condition, the pass sets `lookup_without_expansion` so a lookup never starts `expand_pending` and runs compile-time code early; a condition that doesn't resolve yet (`plain_condition_resolves`) is left for the lazy pass.

A lookup skips `expand_pending` when the scope already binds the name to a non-overloadable, non-placeholder entity, so an `#if` reading `DEBUG` doesn't run every `#insert` in the scope.

An `#add_context` that arrives after the Context type exists but before layout is appended to its struct (`struct_asts.extra`); after layout it is an error.

An item that fails while a procedure body is mid-lowering is retried later (see [sema: polymorphism and declarations](sema-polymorphism-and-declarations.md)). If the program fails before the retry, the error carries the deferred item's own error as a note (`with_deferred_errors` in `driver.rs`), so a missing `#import` shows up instead of a bare "unknown identifier".

## How to change it

- Per-module state goes on `Module` (`mod.rs`).
- Anything that must be visible before compile-time code runs belongs in the plain pass of `expand_all`. Keep that pass free of calls; most declarations aren't resolved yet.
- Queue top-level items and imports with `push_pending` / `push_import`, never by pushing onto `Scope::pending` or `Scope::imports` directly: a scope `expand_all` has already retired would never see them.
- A new kind of module member goes into `module_lookup`, never into one caller, with a line in `tests/stdlib/module-member-resolution.jai`.

Test: `tests/stdlib/static-switch-add-context.jai`.

## Configuration

`Options::import_paths`: `<main file dir>/modules`, then `-I` directories, then the stdlib (built in `crates/jaic-cli/src/main.rs`). `find_module` tries `<importing file dir>/modules` before those.

## Dependencies

The parser, `consteval.rs` (conditions and `#run`), `structs.rs` (Context layout).
