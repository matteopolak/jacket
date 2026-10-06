# Modules and imports

## What it is

A module is a set of Jai files compiled as one namespace. `#import` brings a module, file, directory or source string into scope; `#load` splices another file into the current module.

## How it works

Forms of `#import` (each verified with `jaic run`) {#import.1}:

```jai
#import "Basic";                              // Basic.jai or Basic/module.jai on the search path
G :: #import "Greeter"(LOUD = true);          // named: members via G.name
using S :: #import "Plain";                   // named, and members also unqualified
T :: #import,file "sub/thing.jai";            // a file, relative to the importing file
D :: #import,dir "modules/Greeter";           // a directory containing module.jai
Str :: #import,string "str_val :: 11;";       // source text as a module
#load "other.jai";                            // same module, same namespace
```

The comments above hold for the plain {#import.2}, named {#import.3}, `using` {#import.4}, `,file` {#import.5}, `,dir` {#import.6} and `,string` {#import.7} forms and for `#load` {#import.8}.

- `#import "Name"` tries `Name.jai`, then `Name/module.jai`, in `<importing file's dir>/modules`, then the `-I` directories, then the stdlib {#import.9}. `Compiler::find_module` in `sema/modules.rs` does the lookup; a miss reports ``module `X` not found``, with a note listing the directories searched and a help naming the closest module found there {#import.10}.
- `#load` and `#import,file` resolve relative to the file that wrote them {#import.11}. Loading a file twice into one module is a no-op {#import.12}.
- A module instance is keyed by canonical entry path plus parameter values, so two plain `#import "M"` share one {#import.13}. See [module parameters](module-parameters.md).
- Imports resolve eagerly during `expand_all`, so a module's top-level `#run`s and `#add_context`s are known before they matter {#import.14}.
- Only exported names are reachable through the namespace (see [scoping](scoping.md)): `P.secret` on a `#scope_file` name fails with `` module `Plain` has no exported member `secret` `` {#import.15}.

### Filters

`using,only(a, b) M;` brings in just those names {#import.16}; `except` and `map` filters also exist (`ast::UsingFilter`).

`using,only(a, b) #import "M";` filters an import (`Import::using`). The listed names resolve normally, but M's other names are still found as a last resort when nothing else binds them (`lookup_sibling_file_imports`), because libraries like Epic_Fail list only some of the names they use {#import.17}. The filter's real job is to stop, for example, Basic's `assert` competing with the module's own (`tests/stdlib/using-only-import.jai`) {#import.18}.

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
