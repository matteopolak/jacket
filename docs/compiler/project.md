# Project structure (`jaic::project`)

## What it is

The one place that knows how a Jai project is laid out for tools that do not compile all of it: the base folder (the `jacket.toml` itself is read by `crates/jacket-config`, [jacket](../tools/jacket.md)), the entry files, the `#load`/`#import` directives of a file, the files a program's `#load`s reach, and the root file a source file is compiled from. `jailsp` and `jailint` build on it; neither carries a parser or a load walk of its own.

## How it works

- **Settings.** Not here: `jacket-config` parses `jacket.toml` (serde and `toml`) and finds the nearest one through a `Files` trait (the editor counts unsaved buffers). `find_config_dir` remains for a caller that wants any one file name.
- **Base and entries.** `project_base`: the folder of the nearest config, else the innermost workspace folder holding the file. `jacket_config::entry_files`: `[project] entry`, else the `INFERRED_ENTRIES` that exist (`build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai`; `declared` says which); `first_entry` is the one `jacket` hands to jaic.
- **Import paths.** `import_paths(main_dir, configured, extra, stdlib)` is the one place the module search order is decided, and what `jaic` does for a real compile: `<main_dir>/modules`, then `configured` (`[project] import_path`, joined onto the `jacket.toml` folder by `jacket_config::Project::import_dirs`; `jacket` hands them to `jaic` as `-import_dir`), then `extra` (`-import_dir`/`-I` folders and whatever else a tool adds), then the stdlib. A folder listed twice keeps its first place; folders that do not exist stay (the compiler skips them; `existing_dirs(fs, dirs)` filters for a tool that needs only real ones). Parent folders are not searched. `jaic-cli`, `jailint` (`driver::program_of`), the playground (`jai-wasm`) and the language server's environments all call it; `add_import_dirs` is the same insertion for an environment that owns the base list. The compiler itself also looks in `modules/` next to the importing file first (`sema::modules::search_roots`, see [Module search order](#module-search-order)).
  Every tool sets a compile up the same way: `ProgramInputs::for_file(fs, main, default_stdlib)` finds the enclosing standard library (a checkout's `stdlib/` wins over the installed one, `jaic::enclosing_stdlib`), then `sema::Options::for_program(fs, main, &inputs)` builds the import paths from `import_paths` and calls `set_stdlib`, which fixes the prelude and its ordering. `Options::use_program(fs, &Program)` does the same for an already planned program and keeps a preload the caller supplied. `jaic`, `jailint`, the language server and the playground (`jai-wasm`, which has only `/stdlib`) all call these; none spells the stdlib directory or the order itself.
- **Programs.** `resolve_program(fs, file, &RootOptions, &ProgramInputs, cache)` is `resolve_root` plus the import folders that root is compiled with (`program_for`); a module root also searches the folder holding it. `ProjectFs` is the only file system it sees, so the editor's unsaved text is a parameter. `module_import` is how a program imports a module file. `module_root_stub(dir)` and `module_root_stub_text(import)` give the virtual one-line program (`.jacket-module-root.jai` in the import folder, so it counts as library code) that `Compiler::compile_module_root` loads to check a module as a root; `Root::module_stub()` produces it. The linter and the language server both compile a module root this way (`jaic::ide::LintCompile`).
- **Directives.** `scan_directives(tokens)` finds `#load` and `#import` (with `,file`/`,dir` flags; `,string` is skipped) from lexer tokens, so it works while the text does not parse. `directive_links(file, text)` also resolves `#load`/`,file`/`,dir` targets against the file's folder. A module import needs the import paths, so its `target` stays `None`.
- **Files.** `ProjectFs` is what a walk sees: `DiskFs` resolves links in its keys; `jailsp` puts open buffers and its scan cache over the disk (`auto_import.rs` `IndexFs`) and adds the files a build metaprogram compiles to a file's `edges`. `load_closure` and `reachable_loads` follow `edges`.
- **Roots.** `resolve_root(fs, file, &RootOptions, &mut LoadCache)` returns a `Root`: a module (the `module.jai` whose loads reach the file, or the entry of an import folder) or a program (the file that loads it, else the file itself). `RootOptions` carries the choices that differ between tools:
  - `RootOptions::linter(import_dirs)`: module detection on, loaders looked for in the file's folder and two above, `TieBreak::Outermost` (the loader no other loader loads; first path wins).
  - `RootOptions::language_server(folder)`: no module detection, loaders anywhere under `folder`, `TieBreak::ModuleFirstShallowest` (climb: `module.jai`, then shallowest path, then name).
  
  The tools agree on trees with a single loader chain and differ only when several unrelated files load one file; that difference is the tie-break option, not a silent divergence. `jailint` does not use the entry files; `jailsp` tries the project's entry files before calling the loader search.

## How to change it

`ProjectFs::loader_graph` lets a file system that keeps the loaders of a tree between calls (the language server's `IndexFs`) answer the `ModuleFirstShallowest` search without reading every `#load` again; the default returns `None` and `build_loader_graph` is used.

Add a setting by extending the schema in `crates/jacket-config/src/schema.rs`. Change root rules by adding a variant to `TieBreak` or `LoaderSearch` rather than editing the existing ones, and extend `linter_and_language_server_agree_on_the_same_tree` and `tie_breaks_differ_when_loaders_disagree`. `jai-language-server` `project.rs` has a test that the real `Session::project_root` equals the linter preset on a fixture tree.

## Configuration

`jacket.toml`; see [jacket](../tools/jacket.md), the [language server](language-server.md) and [jailint](../tools/jailint.md) docs for their keys.

## Dependencies

`jaic::lexer` (tokens), `jaic::paths` (normalisation, canonical paths), `jaic::ast::ImportSource`.

## Module search order

`#import "Name"` finds `Name.jai`, else `Name/module.jai`, in these roots, first match wins (`sema::modules::search_roots` builds the list, `find_module_in` walks it; `Compiler::find_module` adds the first row):

| # | Root | Set by |
| --- | --- | --- |
| 0 | the selected bootstrap entry, only for the name `Preload` | `Options::preload` (`Compiler::find_module`) |
| 1 | `modules/` next to the importing file | the importing file's folder |
| 2 | `<main_dir>/modules` | `import_paths` |
| 3 | `[project] import_path` folders | `jacket.toml`, handed to `jaic` as `-import_dir` |
| 4 | `-import_dir` / `-I` folders and tool additions | the command line, the tool |
| 5 | the standard library | `set_stdlib` |

If no root has a module named `Preload`, the prelude is looked up in the same roots in order (`find_preload`; `JAIC_PRELUDE` overrides the file). `Runtime_Support` is found through the same roots under its protocol name. There is no other rule: no `Basic` special case, and a file's own location, including a file under `stdlib/internal/`, does not change the list.

Consequences for the standard library:

- A file resolves a name exactly as the module that loads it does. A project folder that holds a module named like a library one (`Windows`, `Objective_C`, `Thread`) replaces it for the whole program, library modules included. The names `Objective_C/AppKit` and `Extensions/X` are module names that contain a `/`, joined onto each root.
- The library's own native code therefore does not import a module a project may replace. `stdlib/internal` reaches the platform bindings through private modules (`internal/bindings/windows-api`, `posix-api`, `objc-api`, `curl-api`), which load the same `declarations.jai` and `wrappers.jai` the public modules hold; the public `Windows`, `POSIX`, `Objective_C` and `Curl` re-export them (`using Declarations :: #import ...`), so without a replacement everything shares one set of types. Where a binding type would cross from the library into a public module that uses the project's module (a window handle, a context), the internal code passes `*void` (`internal/window/cocoa.jai`, `internal/bindings/nsgl`). Small needs are declared privately (`internal/bindings/inotify`, `internal/gamepad/linux-abi.jai`).
- `X11` is the exception: its globals (`x_global_display`) are part of its public API, so the window code shares the module the program imports.

**To change it:** the order is the body of `search_roots`; extend the table-driven test `search_roots_in_order` in `crates/jaic/src/sema/modules/filesystem.rs` with the new row, and `import_paths` in `project.rs` for roots outside the compiler. The fixture projects in `crates/jaic-cli/tests/module_replacement` (a minimal and a full replacement) are type-checked for the other OS by `crates/jaic-cli/tests/module_replacement.rs`; a new internal file that imports a replaceable binding module fails the minimal ones.
