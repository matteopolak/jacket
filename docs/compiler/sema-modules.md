# Sema: module loading and top-level expansion

## What it is

How `crates/jaic/src/sema` loads modules, binds module parameters, and expands top-level `#if` / `#insert` /
`#run` items: mostly `modules.rs` (loading, `expand_all`) and `scope.rs` (lookup, `expand_pending`).

## How it works

- **Loading** (`load_module`): a module is keyed by entry path and its parameter values. The first
  `#module_parameters (...)` list holds module parameters (part of the instance key); a second list holds
  **program parameters** (`Module.program_params`), shared by every instance in the program.
- **Program parameters**: `#import "Basic"()(MEMORY_DEBUGGER = DEBUG)` records a setting in
  `Compiler::program_param_settings` (entry path, name, value expression, importing scope). `apply_program_params`
  patches the parameter's declaration value as soon as both the module instance and the setting exist, before
  anything reads it. Setting it twice to different values is an error.
- **Typed module parameters**: a scalar argument for `$X: T` becomes the declaration's literal value with its
  declared type kept.
- **Exported `using`**: a module-level `using global;` outside `#scope_file` is recorded in
  `Module.exported_usings`, so importers find the members (`Found::Using`). An exported
  `using X :: #import "Y";` re-exports `Y`'s names (`Module.exported_using_imports`, followed by
  `module_exports` with a cycle guard).
- **Top-level expansion** (`expand_all`): first a pass over every scope expands `#if` items whose condition is a
  plain constant (`expand_plain_ifs`, no calls or `#run`), and imports. Then lookups expand pending items lazily
  (`expand_pending`). This ordering makes a module's `#if FLAG #load "x.jai"` (and an `#add_context` in it) land
  before any `#run` lays out the Context.
- **Settled names**: a lookup skips `expand_pending` when the scope already binds the name to a non-overloadable,
  non-placeholder entity, so a `#if` condition reading `DEBUG` does not run every `#insert` of the scope.
- **Sibling file imports**: an unknown name is finally looked up in the `#scope_file` imports of the module's other
  files (`lookup_sibling_file_imports`).
- **`#add_context`** arriving after the Context type was created but before it is laid out is appended to its
  struct (`struct_asts.extra`); after layout it is an error.
- **Deferred items**: an item that fails while a procedure body is mid-lowering is retried later (see
  [sema-polymorphism-and-declarations.md](sema-polymorphism-and-declarations.md)). If the program fails before the retry, the error carries the deferred
  item's own error as a note (`with_deferred_errors` in `driver.rs`), so a missing `#import` shows up instead of
  only "unknown identifier".

## How to change it

New per-module state goes on `Module` (`mod.rs`). Anything that must be visible before compile-time code runs
belongs in the plain pass of `expand_all`; keep that pass free of calls, since it runs before most declarations
resolve.

## Configuration

Import directories are `Options::import_paths`: `<main file dir>/modules`, then CLI `-I dir`s, then the stdlib (`crates/jaic-cli/src/main.rs`); `find_module` also tries `<importing file dir>/modules` first. User-facing view: [modules-and-imports](../language/modules-and-imports.md).

## Dependencies

`parser` (module files), `consteval.rs` (conditions and `#run`), `structs.rs` (Context layout).
