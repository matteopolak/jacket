# Modules and imports

## What it is

A module is a set of Jai files compiled as one namespace. `#import` brings in a module, file, directory or source string; `#load` splices a file into the current module.

## How it works

```jai
#import "Basic";                              // Basic.jai or Basic/module.jai
G :: #import "Greeter"(LOUD = true);          // named: G.name
using S :: #import "Plain";                   // named and unqualified
T :: #import,file "sub/thing.jai";            // relative to the importing file
D :: #import,dir "modules/Greeter";
Str :: #import,string "str_val :: 11;";       // fresh module each time, no caching
#load "other.jai";                            // same namespace
```

- `#import "Name"` tries `Name.jai`, then `Name/module.jai`, in `<importing dir>/modules`, then `-I` directories, then the stdlib (`Compiler::find_module`, `sema/modules.rs`). A miss names the directories searched and the closest module. A path name (`Extensions/Long_Double`) is searched the same way; see [stdlib extensions](../extensions/README.md).
- `#load` and `#import,file` resolve relative to the writing file; loading twice is a no-op.
- Instances are keyed by entry path plus parameter values ([module parameters](module-parameters.md)). Imports resolve eagerly during `expand_all`, so top-level `#run` and `#add_context` are known early.
- Only exported names are reachable (`P.secret` on a `#scope_file` name is an error); see [scoping](scoping.md).
- `using,only(a, b) #import "M";` filters (`except` and `map` also exist, `ast::UsingFilter`); excluded names are not found by fallback lookup and do not join overload sets, but a named alias still gives qualified access.

## How to change it

`resolve_import`, `load_module`, `load_file` and `declare_stmt` in `sema/modules.rs`. File access goes through the `FileSystem` trait (`NativeFs`, `VirtualFs` for browser and tests); use `self.fs`, never `std::fs`. `module_lookup` in `scope.rs` follows re-exports and is the one lookup path; keep its cycle guard.

Configuration: `-I dir` / `-import_dir dir`, `JAIC_STDLIB`, and `import_path` in build options (`crates/jaic/src/build.rs`). Expansion order: [sema: module loading](../compiler/sema.md).

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Forms of `#import` (each verified with `jaic -interp`) {#import.1}:
- The comments above hold for the plain {#import.2}, named {#import.3}, `using` {#import.4}, `,file` {#import.5}, `,dir` {#import.6} and `,string` {#import.7} forms and for `#load` {#import.8}.
- `#import "Name"` tries `Name.jai`, then `Name/module.jai`, in `<importing file's dir>/modules`, then the `-I` directories, then the stdlib {#import.9}. `Compiler::find_module` in `sema/modules.rs` does the lookup; a miss reports ``module `X` not found``, with a note listing the directories searched and a help naming the closest module found there {#import.10}.
- A name may be a path: `#import "Folder/Name"` tries `Folder/Name.jai`, then `Folder/Name/module.jai`, in the same directories. jaic's own modules use this: they live in `stdlib/Extensions/` and are imported as `#import "Extensions/Long_Double";`, which the stdlib folder alone does not offer as `#import "Long_Double";`. Importing one without the `Extensions/` path, or the folder itself, reports the import to write instead of the search note {#import.19}. See [stdlib extensions](../extensions/README.md).
- `#load` and `#import,file` resolve relative to the file that wrote them {#import.11}. Loading a file twice into one module is a no-op {#import.12}.
- A module instance is keyed by canonical entry path plus parameter values, so two plain `#import "M"` share one {#import.13}. See [module parameters](module-parameters.md).
- Imports resolve eagerly during `expand_all`, so a module's top-level `#run`s and `#add_context`s are known before they matter {#import.14}.
- Only exported names are reachable through the namespace (see [scoping](scoping.md)): `P.secret` on a `#scope_file` name fails with `` module `Plain` has no exported member `secret` `` {#import.15}.
- `using,only(a, b) M;` brings in just those names {#import.16}; `except` and `map` filters also exist (`ast::UsingFilter`).
- `using,only(a, b) #import "M";` filters an import (`Import::using`): excluded names do not become available through fallback lookup {#import.17}. A named alias remains available for qualified lookup of the module's exported names. Filters also prevent excluded procedures from joining an overload set {#import.18}.
