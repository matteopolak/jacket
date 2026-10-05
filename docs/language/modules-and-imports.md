# Modules and imports

## What it is

A module is a set of Jai files compiled as one namespace. `#import` brings a module (or a single file, directory or string) into scope; `#load` splices another file into the current module.

## How it works

Forms of `#import` (each verified with `jaic run`):

```jai
#import "Basic";                              // Basic.jai or Basic/module.jai on the search path
G :: #import "Greeter"(LOUD = true);          // named: members via G.name
using S :: #import "Plain";                   // named, and members also unqualified
T :: #import,file "sub/thing.jai";            // a file, relative to the importing file
D :: #import,dir "modules/Greeter";           // a directory containing module.jai
Str :: #import,string "str_val :: 11;";       // source text as a module
#load "other.jai";                            // same module, same namespace
```

- `#import "Name"` looks for `Name.jai`, then `Name/module.jai`, in: `<importing file dir>/modules`, then the `-I`/`-import_dir` directories, then the stdlib. The CLI builds this list in `crates/jaic-cli/src/main.rs` (`options.import_paths`); the lookup is `Compiler::find_module` in `crates/jaic/src/sema/modules.rs`. A miss reports `module 'X' not found (searched N import directories)`.
- `#load` and `#import,file` resolve relative to the file that wrote them. Loading the same file twice into one module is a no-op.
- A module is keyed by canonical entry path plus its parameter values, so two plain `#import "M"` share one instance. See [module-parameters.md](module-parameters.md).
- Imports are resolved eagerly during `expand_all`, not on first use, so a module's top-level `#run`s and `#add_context` are known before they matter.
- Only export-visibility names are reachable through the namespace (see [scoping.md](scoping.md)). `P.secret` on a `#scope_file` name fails with `module 'Plain' has no exported member 'secret'`.
- `using,only(a, b) M;` filters what a `using` brings in; verified: `using,only(answer) M;` exposes just `answer`. The AST also has `except` and `map` filters (`ast::UsingFilter`), which I did not exercise here.
- `using,only(a, b) #import "M";` imports with a filter (`Import::using`). The listed names resolve like any import; M's other names are still found as a last resort when nothing else binds them (`lookup_sibling_file_imports`), since libraries such as Epic_Fail list only some of the names they use. The filter's point is to keep e.g. Basic's `assert` from competing with the module's own (`tests/stdlib/using-only-import.jai`).

## How to change it

Import loading is `Compiler::resolve_import` / `load_module` / `load_file` in `crates/jaic/src/sema/modules.rs`; `declare_stmt` there turns each top-level `Import`, `Load` and `Using` into scope entries. File access goes through the `FileSystem` trait (`NativeFs` for the CLI, `VirtualFs` for the browser and tests), so new reads must use `self.fs`, never `std::fs`.

Gotchas: `#import,string` makes a fresh module each time (no caching). Re-exports through `using` chains are followed by `module_lookup` in `scope.rs`, the one lookup every access path uses; keep its cycle guard.

## Configuration

`-I dir` / `-import_dir dir` on `jaic run|check|build` adds an import directory. `JAIC_STDLIB` overrides the stdlib directory. Metaprograms can set `import_path` through the build options (`crates/jaic/src/build.rs`).

## Dependencies

`parser` for file parsing, `FileSystem` for I/O, and [sema-modules.md](../compiler/sema-modules.md) for the expansion order.
