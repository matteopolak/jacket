# Modules and imports

A module is a set of Jai files compiled as one namespace. `#import` brings in a module, file, directory or source string; `#load` splices a file into the current module.

```jai
#import "Basic";                              // Basic.jai or Basic/module.jai
G :: #import "Greeter"(LOUD = true);          // named: G.name
using S :: #import "Plain";                   // named and unqualified
T :: #import,file "sub/thing.jai";            // relative to the importing file
D :: #import,dir "modules/Greeter";
Str :: #import,string "str_val :: 11;";       // fresh module each time, no caching
#load "other.jai";                            // same namespace
```

The comments hold for the plain {#import.2}, named {#import.3}, `using` {#import.4}, `,file` {#import.5}, `,dir` {#import.6} and `,string` {#import.7} forms and for `#load` {#import.8}; each form is verified with `jaic -interp` {#import.1}.

## How it works

- `#import "Name"` tries `Name.jai`, then `Name/module.jai`, in `<importing file's dir>/modules`, then the `-I` directories, then the stdlib {#import.9}. A miss reports ``module `X` not found``, with the directories searched and the closest module {#import.10}.
- A name may be a path: `#import "Extensions/Long_Double"` tries `Extensions/Long_Double.jai`, then `.../module.jai`. Importing an extension without its folder, or the folder itself, reports the import to write {#import.19}. See [stdlib extensions](../extensions/README.md).
- `#load` and `#import,file` resolve relative to the file that wrote them {#import.11}. Loading a file twice into one module is an error: `` `part.jai` is already loaded into this module `` {#import.12}.
- An instance is keyed by canonical entry path plus parameter values, so two plain `#import "M"` share one {#import.13}. See [module parameters](module-parameters.md).
- Imports resolve eagerly, so a module's top-level `#run`s and `#add_context`s are known before they matter {#import.14}.
- Qualified `M.name` sees module-scope and exported declarations (see [scoping](scoping.md)). File-scope ones don't: `P.secret` on a `#scope_file` name fails with `` module `Plain` has no exported member `secret` `` {#import.15}. Plain imports and `using` expose only exports.
- `using,only(a, b) M;` brings in just those names {#import.16}; `except` and `map` also exist.
- `using,only(a, b) #import "M";` filters an import: excluded names aren't found by fallback lookup {#import.17}, and excluded procedures don't join overload sets {#import.18}. A named alias still gives qualified access.
- `using,except(foo, bar) Dog :: #import "Dog";` keeps `Dog` as a qualified alias and imports the other exports. The excluded names stay reachable as `Dog.foo`.
- A module's top-level `using Alias;` (`I :: #import "I"; using I;`) makes I's names members of the module. At export scope they reach `M.name` and plain importers. At `#scope_module` they reach only `M.name`, which is how `String.tprint` resolves to Basic's `tprint` while `#import "String"` alone leaves `tprint` undeclared. `#scope_file` usings are private to their file.

## How to change it

`resolve_import`, `load_module`, `load_file` and `declare_stmt` in `sema/modules.rs`; lookup by `Compiler::find_module`. File access goes through the `FileSystem` trait (`NativeFs`, `VirtualFs`); use `self.fs`, never `std::fs`. `module_lookup` in `sema/scope/module_lookup.rs` is the one lookup path for imports and follows re-exports (keep its cycle guard); `module_member_lookup` handles `M.name`. Imported and qualified lookup have separate boundaries, so cache entries for one must never stand in for the other. Negative export tests must use a plain import or `using` with an unqualified name.

Config: `-I dir`/`-import_dir dir`, `JAIC_STDLIB`, `JAIC_PRELUDE` ([prelude selection](../compiler/prelude-selection.md)), `import_path` in build options.
