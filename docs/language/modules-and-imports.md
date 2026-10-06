# Modules and imports

## What it is

A module is a set of Jai files compiled as one namespace. `#import` brings a module, file, directory or source string into scope; `#load` splices another file into the current module.

## How it works

```jai
#import "Basic";                              // Basic.jai or Basic/module.jai on the search path
G :: #import "Greeter"(LOUD = true);          // named: members via G.name
using S :: #import "Plain";                   // named, and members also unqualified
T :: #import,file "sub/thing.jai";            // a file, relative to the importing file
D :: #import,dir "modules/Greeter";           // a directory containing module.jai
Str :: #import,string "str_val :: 11;";       // source text as a module
#load "other.jai";                            // same module, same namespace
```

- `#import "Name"` tries `Name.jai`, then `Name/module.jai`, in `<importing file's dir>/modules`, then the `-I` directories, then the stdlib. `Compiler::find_module` in `sema/modules.rs` does the lookup; a miss reports `module 'X' not found (searched N import directories)`.
- `#load` and `#import,file` resolve relative to the file that wrote them. Loading a file twice into one module is a no-op.
- A module instance is keyed by canonical entry path plus parameter values, so two plain `#import "M"` share one. See [module parameters](module-parameters.md).
- Imports resolve eagerly during `expand_all`, so a module's top-level `#run`s and `#add_context`s are known before they matter.
- Only exported names are reachable through the namespace (see [scoping](scoping.md)): `P.secret` on a `#scope_file` name fails with `module 'Plain' has no exported member 'secret'`.

### Filters

`using,only(a, b) M;` brings in just those names; `except` and `map` filters also exist (`ast::UsingFilter`).

`using,only(a, b) #import "M";` filters an import (`Import::using`). The listed names resolve normally, but M's other names are still found as a last resort when nothing else binds them (`lookup_sibling_file_imports`), because libraries like Epic_Fail list only some of the names they use. The filter's real job is to stop, for example, Basic's `assert` competing with the module's own (`tests/stdlib/using-only-import.jai`).

## How to change it

Loading is `resolve_import`, `load_module` and `load_file` in `sema/modules.rs`; `declare_stmt` turns top-level `Import`, `Load` and `Using` into scope entries. All file access goes through the `FileSystem` trait (`NativeFs` for the CLI, `VirtualFs` for the browser and tests), so use `self.fs`, never `std::fs`.

Gotchas:

- `#import,string` makes a fresh module every time; there is no caching.
- `module_lookup` in `scope.rs` follows re-exports through `using` chains and is the single lookup every access path uses. Keep its cycle guard.

## Configuration

- `-I dir` / `-import_dir dir` on `jaic run|check|build` adds an import directory (`options.import_paths`, built in `crates/jaic-cli/src/main.rs`).
- `JAIC_STDLIB` overrides the stdlib directory.
- Metaprograms set `import_path` through the build options (`crates/jaic/src/build.rs`).

## Dependencies

The parser, the `FileSystem` trait, and the expansion order in [sema: module loading](../compiler/sema-modules.md).
